use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{RepoPath, VerificationKind};

/// The precedence layer that introduced a policy fragment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PolicyLayer {
    /// Hardcoded platform invariants.
    Core,
    /// Organization-wide policy.
    Org,
    /// Repository-level policy.
    Repo,
    /// Path-scoped policy.
    Path,
    /// Mode-specific policy.
    Mode,
    /// Task-local constraints.
    Task,
}

/// The trust classification attached to inputs and observations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Built-in platform guidance.
    System,
    /// Repository policy files.
    RepoPolicy,
    /// Source code and manifests from the workspace.
    RepoCode,
    /// The direct user request for the active task.
    UserTask,
    /// Tool output captured during execution.
    ToolOutput,
    /// External text such as issue descriptions or web pages.
    ExternalText,
}

impl TrustLevel {
    /// Returns whether this trust level is allowed to define or widen policy.
    #[must_use]
    pub fn can_define_policy(self) -> bool {
        matches!(self, Self::System | Self::RepoPolicy | Self::UserTask)
    }
}

/// A capability that can be granted, denied, or approval-gated.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Capability {
    /// Read files within the workspace.
    ReadWorkspace,
    /// Edit files within the workspace.
    EditWorkspace,
    /// Run verification commands such as `cargo check`.
    RunChecks,
    /// Run ad hoc commands outside the verification plan.
    RunArbitraryCommand,
    /// Delete files from the workspace.
    DeleteFiles,
    /// Reach the network.
    NetworkAccess,
    /// Call an MCP server by identifier.
    UseMcpServer(String),
    /// Transition between execution modes.
    SwitchMode,
}

/// The approval policy attached to a capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalRule {
    /// The capability can be used without approval.
    Allow,
    /// The capability requires explicit approval.
    Ask,
    /// The capability requires approval when the action is risky.
    AskIfRisky,
    /// The capability is not allowed.
    Deny,
}

/// A repository-relative path restriction.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PathPattern(pub RepoPath);

/// The repository stance on dependency changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DependencyPolicy {
    /// Dependency additions are denied unless a policy layer approves them.
    Frozen,
    /// Dependency additions are allowed with explicit approval.
    #[default]
    AllowApproved,
    /// Dependency additions are broadly allowed.
    Flexible,
}

/// Reporting rules that constrain what the runtime may claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReportingPolicy {
    /// Whether the runtime may claim a fix without test evidence.
    pub may_claim_fix_without_tests: bool,
}

/// Minimum verification requirements imposed by policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ValidationMinimums {
    /// Verification steps that must run before the report is complete.
    #[serde(default)]
    pub must_run: BTreeSet<VerificationKind>,
}

/// The fully resolved policy used by a run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectivePolicy {
    /// Capabilities the current mode is allowed to attempt.
    pub allowed_capabilities: BTreeSet<Capability>,
    /// Approval rules attached to individual capabilities.
    pub approval_rules: BTreeMap<Capability, ApprovalRule>,
    /// Workspace paths that must remain untouched.
    pub forbidden_paths: Vec<PathPattern>,
    /// Dependency modification policy.
    pub dependency_policy: DependencyPolicy,
    /// Reporting constraints for the final report.
    pub reporting_policy: ReportingPolicy,
    /// Verification minimums enforced by policy.
    pub validation_minimums: ValidationMinimums,
}
