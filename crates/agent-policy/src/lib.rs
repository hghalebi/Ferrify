//! Policy loading, resolution, and authorization.
//!
//! `agent-policy` is the governance core for Ferrify. It loads declarative mode
//! and approval-profile files from `.agent/`, merges them into an
//! [`EffectivePolicy`], and decides whether a capability or mode transition is
//! allowed for the current run.
//!
//! The crate deliberately separates repository configuration from application
//! orchestration. That keeps policy versionable, reviewable, and testable
//! without hardwiring repository-specific rules into the runtime itself.
//!
//! # Examples
//!
//! ```no_run
//! use agent_domain::ApprovalProfileSlug;
//! use agent_policy::{PolicyEngine, PolicyRepository};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let repository = PolicyRepository::load_from_root(std::path::Path::new("."))?;
//! let engine = PolicyEngine::new(repository);
//! let resolved = engine.resolve("architect", &ApprovalProfileSlug::new("default")?)?;
//!
//! assert!(resolved
//!     .effective_policy
//!     .allowed_capabilities
//!     .contains(&agent_domain::Capability::ReadWorkspace));
//! # Ok(())
//! # }
//! ```

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use agent_domain::{
    ApprovalProfileSlug, ApprovalRule, Capability, DependencyPolicy, EffectivePolicy, ModeSlug,
    PatchBudget, PathPattern, ReportingPolicy, ValidationMinimums,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A declarative execution mode loaded from `.agent/modes/*.yaml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeSpec {
    /// Stable mode identifier.
    pub slug: ModeSlug,
    /// The purpose of the mode.
    pub purpose: String,
    /// Capabilities the mode is allowed to request.
    #[serde(default)]
    pub allowed_capabilities: BTreeSet<Capability>,
    /// Mode-specific approval overrides.
    #[serde(default)]
    pub approval_rules: BTreeMap<Capability, ApprovalRule>,
    /// Verification rules that must hold for the mode.
    #[serde(default)]
    pub validation_minimums: ValidationMinimums,
    /// Reporting constraints attached to the mode.
    #[serde(default)]
    pub reporting: ReportingPolicy,
    /// The default patch budget for this mode.
    #[serde(default)]
    pub patch_budget: PatchBudget,
}

/// A named approval profile loaded from `.agent/approvals/*.yaml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalProfile {
    /// Stable profile identifier.
    pub slug: ApprovalProfileSlug,
    /// Base approval rules for the repository.
    #[serde(default)]
    pub approval_rules: BTreeMap<Capability, ApprovalRule>,
    /// Paths that the runtime should never edit.
    #[serde(default)]
    pub forbidden_paths: Vec<PathPattern>,
    /// The repository stance on dependency changes.
    #[serde(default)]
    pub dependency_policy: DependencyPolicy,
    /// Reporting rules shared across modes.
    #[serde(default)]
    pub reporting: ReportingPolicy,
}

/// In-memory policy data loaded from the repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyRepository {
    modes: BTreeMap<ModeSlug, ModeSpec>,
    approval_profiles: BTreeMap<ApprovalProfileSlug, ApprovalProfile>,
}

impl PolicyRepository {
    /// Loads `.agent/modes` and `.agent/approvals` from the repository root.
    ///
    /// # Errors
    ///
    /// Returns [`PolicyError`] when the directories cannot be read or when a
    /// YAML file fails to deserialize into the expected policy type.
    pub fn load_from_root(root: &Path) -> Result<Self, PolicyError> {
        let modes_dir = root.join(".agent").join("modes");
        let approvals_dir = root.join(".agent").join("approvals");

        let modes = load_yaml_directory::<ModeSpec>(&modes_dir)?
            .into_iter()
            .map(|mode| (mode.slug.clone(), mode))
            .collect();
        let approval_profiles = load_yaml_directory::<ApprovalProfile>(&approvals_dir)?
            .into_iter()
            .map(|profile| (profile.slug.clone(), profile))
            .collect();

        Ok(Self {
            modes,
            approval_profiles,
        })
    }

    /// Returns a mode by slug.
    ///
    /// # Errors
    ///
    /// Returns [`PolicyError::MissingMode`] when the requested mode is not
    /// present in the loaded repository policy.
    pub fn mode(&self, slug: &str) -> Result<&ModeSpec, PolicyError> {
        self.modes
            .get(slug)
            .ok_or_else(|| PolicyError::MissingMode(slug.to_owned()))
    }

    /// Returns an approval profile by slug.
    ///
    /// # Errors
    ///
    /// Returns [`PolicyError::MissingApprovalProfile`] when the requested
    /// profile was not loaded from `.agent/approvals`.
    pub fn approval_profile(
        &self,
        slug: &ApprovalProfileSlug,
    ) -> Result<&ApprovalProfile, PolicyError> {
        self.approval_profiles
            .get(slug)
            .ok_or_else(|| PolicyError::MissingApprovalProfile(slug.to_string()))
    }
}

/// Resolves declarative policy into an effective policy and enforces approvals.
#[derive(Debug, Clone)]
pub struct PolicyEngine {
    repository: PolicyRepository,
}

impl PolicyEngine {
    /// Creates a policy engine from the loaded repository data.
    #[must_use]
    pub fn new(repository: PolicyRepository) -> Self {
        Self { repository }
    }

    /// Returns the loaded repository data.
    #[must_use]
    pub fn repository(&self) -> &PolicyRepository {
        &self.repository
    }

    /// Resolves the effective policy for a mode and approval profile.
    ///
    /// The result is the policy Ferrify actually executes with after mode
    /// defaults and approval-profile overrides have been merged.
    ///
    /// # Errors
    ///
    /// Returns [`PolicyError`] when either the mode or approval profile is
    /// missing from the loaded repository data.
    pub fn resolve(
        &self,
        mode_slug: &str,
        approval_profile_slug: &ApprovalProfileSlug,
    ) -> Result<ResolvedMode, PolicyError> {
        let mode = self.repository.mode(mode_slug)?.clone();
        let approval_profile = self
            .repository
            .approval_profile(approval_profile_slug)?
            .clone();

        let mut approval_rules = default_approval_rules();
        approval_rules.extend(approval_profile.approval_rules.clone());
        approval_rules.extend(mode.approval_rules.clone());

        let effective = EffectivePolicy {
            allowed_capabilities: mode.allowed_capabilities.clone(),
            approval_rules,
            forbidden_paths: approval_profile.forbidden_paths.clone(),
            dependency_policy: approval_profile.dependency_policy,
            reporting_policy: mode.reporting.clone(),
            validation_minimums: mode.validation_minimums.clone(),
        };

        Ok(ResolvedMode {
            spec: mode,
            effective_policy: effective,
        })
    }

    /// Checks whether a capability can be used with the provided approvals.
    ///
    /// # Errors
    ///
    /// Returns [`PolicyError`] when the capability is not allowed by the active
    /// mode, when the capability is denied outright, or when the capability
    /// requires approval and the caller did not supply it.
    pub fn authorize(
        &self,
        policy: &EffectivePolicy,
        capability: &Capability,
        approvals: &BTreeSet<Capability>,
    ) -> Result<(), PolicyError> {
        if !policy.allowed_capabilities.contains(capability) {
            return Err(PolicyError::CapabilityNotAllowed(capability.clone()));
        }

        let rule = policy
            .approval_rules
            .get(capability)
            .copied()
            .unwrap_or(ApprovalRule::Deny);

        match rule {
            ApprovalRule::Allow => Ok(()),
            ApprovalRule::Ask | ApprovalRule::AskIfRisky if approvals.contains(capability) => {
                Ok(())
            }
            ApprovalRule::Ask | ApprovalRule::AskIfRisky => {
                Err(PolicyError::ApprovalRequired(capability.clone()))
            }
            ApprovalRule::Deny => Err(PolicyError::CapabilityDenied(capability.clone())),
        }
    }

    /// Enforces the rule that widening a mode's authority requires approval.
    ///
    /// # Errors
    ///
    /// Returns [`PolicyError`] when the target mode introduces a capability
    /// that is either disallowed or not explicitly approved for the transition.
    pub fn authorize_transition(
        &self,
        from: &EffectivePolicy,
        to: &EffectivePolicy,
        approvals: &BTreeSet<Capability>,
    ) -> Result<(), PolicyError> {
        for capability in to
            .allowed_capabilities
            .difference(&from.allowed_capabilities)
        {
            self.authorize(to, capability, approvals)?;
        }

        Ok(())
    }
}

/// A resolved mode paired with its effective policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedMode {
    /// The mode specification loaded from the repository.
    pub spec: ModeSpec,
    /// The effective policy derived from the mode and approval profile.
    pub effective_policy: EffectivePolicy,
}

/// Errors produced while loading or enforcing policy.
#[derive(Debug, Error)]
pub enum PolicyError {
    /// Filesystem access failed.
    #[error("failed to access policy files: {0}")]
    Io(#[from] std::io::Error),
    /// YAML parsing failed.
    #[error("failed to parse policy file {path}: {source}")]
    Yaml {
        /// The file that failed to parse.
        path: PathBuf,
        /// The underlying parse error.
        source: serde_yaml::Error,
    },
    /// The requested mode was not present.
    #[error("missing mode `{0}`")]
    MissingMode(String),
    /// The requested approval profile was not present.
    #[error("missing approval profile `{0}`")]
    MissingApprovalProfile(String),
    /// The capability is not allowed in the current mode.
    #[error("capability `{0:?}` is not allowed in this mode")]
    CapabilityNotAllowed(Capability),
    /// The capability requires explicit approval.
    #[error("capability `{0:?}` requires approval")]
    ApprovalRequired(Capability),
    /// The capability is denied outright.
    #[error("capability `{0:?}` is denied")]
    CapabilityDenied(Capability),
}

fn load_yaml_directory<T>(directory: &Path) -> Result<Vec<T>, PolicyError>
where
    T: for<'de> Deserialize<'de>,
{
    let mut entries = fs::read_dir(directory)?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    entries.sort();

    entries
        .into_iter()
        .filter(|path| matches!(path.extension().and_then(|ext| ext.to_str()), Some("yaml")))
        .map(|path| {
            let raw = fs::read_to_string(&path)?;
            serde_yaml::from_str(&raw).map_err(|source| PolicyError::Yaml {
                path: path.clone(),
                source,
            })
        })
        .collect()
}

fn default_approval_rules() -> BTreeMap<Capability, ApprovalRule> {
    BTreeMap::from([
        (Capability::ReadWorkspace, ApprovalRule::Allow),
        (Capability::RunChecks, ApprovalRule::Allow),
        (Capability::EditWorkspace, ApprovalRule::Ask),
        (Capability::RunArbitraryCommand, ApprovalRule::AskIfRisky),
        (Capability::DeleteFiles, ApprovalRule::AskIfRisky),
        (Capability::NetworkAccess, ApprovalRule::Deny),
        (Capability::SwitchMode, ApprovalRule::Allow),
    ])
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, fs};

    use agent_domain::{ApprovalProfileSlug, Capability};
    use tempfile::tempdir;

    use super::{PolicyEngine, PolicyRepository};

    fn approval_profile_slug(value: &str) -> ApprovalProfileSlug {
        match ApprovalProfileSlug::new(value) {
            Ok(slug) => slug,
            Err(error) => panic!("approval profile slug should be valid in test: {error}"),
        }
    }

    #[test]
    fn policy_engine_requires_approval_for_widening_transition() {
        let tempdir = tempdir().expect("tempdir should be created for policy test");
        let root = tempdir.path();

        fs::create_dir_all(root.join(".agent").join("modes"))
            .expect("mode directory should be created for policy test");
        fs::create_dir_all(root.join(".agent").join("approvals"))
            .expect("approval directory should be created for policy test");

        fs::write(
            root.join(".agent").join("modes").join("architect.yaml"),
            "slug: architect\npurpose: read only\nallowed_capabilities:\n  - ReadWorkspace\n  - SwitchMode\napproval_rules:\n  SwitchMode: Allow\n",
        )
        .expect("architect mode should be written for policy test");
        fs::write(
            root.join(".agent").join("modes").join("implementer.yaml"),
            "slug: implementer\npurpose: edits\nallowed_capabilities:\n  - ReadWorkspace\n  - EditWorkspace\n  - SwitchMode\napproval_rules:\n  EditWorkspace: Ask\n  SwitchMode: Allow\n",
        )
        .expect("implementer mode should be written for policy test");
        fs::write(
            root.join(".agent").join("approvals").join("default.yaml"),
            "slug: default\napproval_rules:\n  EditWorkspace: Ask\n  SwitchMode: Allow\n",
        )
        .expect("approval profile should be written for policy test");

        let repository =
            PolicyRepository::load_from_root(root).expect("policy repository should load");
        let engine = PolicyEngine::new(repository);
        let default_profile = approval_profile_slug("default");
        let architect = engine
            .resolve("architect", &default_profile)
            .expect("architect policy should resolve");
        let implementer = engine
            .resolve("implementer", &default_profile)
            .expect("implementer policy should resolve");

        let denied = engine.authorize_transition(
            &architect.effective_policy,
            &implementer.effective_policy,
            &BTreeSet::new(),
        );
        assert!(denied.is_err());

        let mut approvals = BTreeSet::new();
        approvals.insert(Capability::EditWorkspace);
        let allowed = engine.authorize_transition(
            &architect.effective_policy,
            &implementer.effective_policy,
            &approvals,
        );
        assert!(allowed.is_ok());
    }
}
