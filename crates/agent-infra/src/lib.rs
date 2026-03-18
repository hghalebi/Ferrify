//! Runtime primitives for verification, sandbox selection, and tool brokering.
//!
//! `ferrify-infra` defines the boundary between Ferrify's control-plane types and
//! the outside world. This includes sandbox selection, tool-broker contracts,
//! and the verification backend that shells out to Cargo.
//!
//! The crate stays intentionally small in the starter implementation. The goal
//! is to make operational boundaries explicit before adding richer runtimes or
//! external integrations.
//!
//! # Examples
//!
//! ```
//! # use agent_domain as ferrify_domain;
//! use ferrify_domain::ModeSlug;
//! use ferrify_infra::{SandboxManager, SandboxProfile};
//!
//! let mode = ModeSlug::new("verifier").expect("verifier is a valid mode slug");
//! assert_eq!(
//!     SandboxManager::profile_for_mode(&mode),
//!     SandboxProfile::ReadOnlyWorkspace
//! );
//! ```

use std::{
    path::Path,
    process::{Command, Output},
};

use agent_domain::{
    ArtifactRef, Capability, EffectivePolicy, InputRole, ModeSlug, TrustLevel, ValidationReceipt,
    VerificationKind, VerificationPlan, VerificationStatus,
};
use thiserror::Error;

/// The sandbox profile attached to a stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxProfile {
    /// Read-only access to the workspace.
    ReadOnlyWorkspace,
    /// Workspace writes without network access.
    WorkspaceWriteNoNetwork,
    /// Workspace writes with allowlisted network access.
    WorkspaceWriteAllowlistedNetwork,
    /// Broad ephemeral autonomy reserved for isolated environments.
    EphemeralFullAuto,
}

/// Chooses runtime sandbox profiles for modes.
#[derive(Debug, Default)]
pub struct SandboxManager;

impl SandboxManager {
    /// Returns the recommended sandbox profile for a mode.
    ///
    /// Ferrify keeps verifier stages read-only and uses a write-without-network
    /// profile for implementer stages. Unknown modes currently default to the
    /// conservative read-only profile.
    #[must_use]
    pub fn profile_for_mode(mode_slug: &ModeSlug) -> SandboxProfile {
        match mode_slug.as_str() {
            "implementer" => SandboxProfile::WorkspaceWriteNoNetwork,
            "verifier" => SandboxProfile::ReadOnlyWorkspace,
            _ => SandboxProfile::ReadOnlyWorkspace,
        }
    }
}

/// A raw tool request that must pass through the broker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolRequest {
    /// The tool identifier.
    pub tool: String,
    /// The serialized input payload.
    pub input: String,
    /// The mode making the request.
    pub requested_by_mode: ModeSlug,
    /// The capability associated with the tool call.
    pub capability: Capability,
}

/// A normalized tool output blob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolOutputBlob {
    /// The mime type for the payload.
    pub mime_type: String,
    /// The textual content captured from the tool.
    pub content: String,
}

/// A fact observed from tool output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedFact {
    /// The fact subject.
    pub subject: String,
    /// The fact detail.
    pub detail: String,
    /// The operational role attached to the fact source.
    pub input_role: InputRole,
    /// The trust classification attached to the fact.
    pub trust_level: TrustLevel,
}

/// An audit record for a tool invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditRecord {
    /// The mode that initiated the call.
    pub mode: ModeSlug,
    /// The capability that gated the call.
    pub capability: Capability,
}

/// The normalized result returned by a brokered tool call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolReceipt {
    /// The synthetic tool call identifier.
    pub tool_call_id: String,
    /// The operational role assigned to the raw output.
    pub input_role: InputRole,
    /// The trust level of the raw output.
    pub trust_level: TrustLevel,
    /// The raw output blob.
    pub raw_output: ToolOutputBlob,
    /// Normalized facts extracted from the output.
    pub normalized_facts: Vec<ObservedFact>,
    /// Audit data attached to the call.
    pub audit: AuditRecord,
}

/// A broker that mediates tool execution under policy.
pub trait ToolBroker {
    /// Executes a tool request under the current effective policy.
    ///
    /// # Errors
    ///
    /// Implementations should return [`ToolError`] when the policy forbids the
    /// capability, when the request cannot be normalized, or when the tool
    /// backend itself is unavailable.
    fn call(
        &self,
        request: ToolRequest,
        policy: &EffectivePolicy,
    ) -> Result<ToolReceipt, ToolError>;
}

/// A deny-by-default broker used by Ferrify.
#[derive(Debug, Default)]
pub struct DenyByDefaultToolBroker;

impl ToolBroker for DenyByDefaultToolBroker {
    fn call(
        &self,
        request: ToolRequest,
        policy: &EffectivePolicy,
    ) -> Result<ToolReceipt, ToolError> {
        if !policy.allowed_capabilities.contains(&request.capability) {
            return Err(ToolError::Unauthorized(request.capability));
        }

        Err(ToolError::Unavailable(request.tool))
    }
}

/// Runs verification commands and returns receipts.
pub trait VerificationBackend {
    /// Executes the verification plan at the repository root.
    ///
    /// # Errors
    ///
    /// Returns [`InfraError`] when the backend cannot launch the required
    /// command or cannot collect its result.
    fn run(
        &self,
        root: &Path,
        plan: &VerificationPlan,
    ) -> Result<Vec<ValidationReceipt>, InfraError>;
}

/// A process-based verification backend that shells out to Cargo.
#[derive(Debug, Default)]
pub struct ProcessVerificationBackend;

impl VerificationBackend for ProcessVerificationBackend {
    fn run(
        &self,
        root: &Path,
        plan: &VerificationPlan,
    ) -> Result<Vec<ValidationReceipt>, InfraError> {
        let mut receipts = Vec::new();
        for step in &plan.required {
            let (program, args) = command_for(*step);
            let output = Command::new(program)
                .args(args)
                .current_dir(root)
                .output()?;
            receipts.push(receipt_for(*step, program, args, &output));
        }

        Ok(receipts)
    }
}

/// Errors produced by the infrastructure layer.
#[derive(Debug, Error)]
pub enum InfraError {
    /// Process execution failed.
    #[error("failed to execute verification command: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors produced by tool brokering.
#[derive(Debug, Error)]
pub enum ToolError {
    /// The requested capability is not allowed.
    #[error("tool capability `{0:?}` is not allowed")]
    Unauthorized(Capability),
    /// The tool is not implemented by the starter broker.
    #[error("tool `{0}` is unavailable in Ferrify")]
    Unavailable(String),
}

fn command_for(step: VerificationKind) -> (&'static str, &'static [&'static str]) {
    match step {
        VerificationKind::CargoFmtCheck => ("cargo", &["fmt", "--check"]),
        VerificationKind::CargoCheck => ("cargo", &["check"]),
        VerificationKind::CargoClippy => (
            "cargo",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ],
        ),
        VerificationKind::TargetedTests => ("cargo", &["test", "--workspace"]),
    }
}

fn receipt_for(
    step: VerificationKind,
    program: &str,
    args: &[&str],
    output: &Output,
) -> ValidationReceipt {
    let command = if args.is_empty() {
        program.to_owned()
    } else {
        format!("{program} {}", args.join(" "))
    };

    let status_code = output.status.code().unwrap_or(-1);
    let stdout_tail = tail_snippet(&output.stdout);
    let stderr_tail = tail_snippet(&output.stderr);
    ValidationReceipt {
        step,
        command,
        status: if output.status.success() {
            VerificationStatus::Succeeded
        } else {
            VerificationStatus::Failed
        },
        artifacts: vec![
            ArtifactRef {
                label: "exit-code".to_owned(),
                location: status_code.to_string(),
            },
            ArtifactRef {
                label: "stdout-tail".to_owned(),
                location: stdout_tail,
            },
            ArtifactRef {
                label: "stderr-tail".to_owned(),
                location: stderr_tail,
            },
        ],
    }
}

fn tail_snippet(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let tail = text.trim();
    if tail.is_empty() {
        return "<empty>".to_owned();
    }

    const MAX_CHARS: usize = 240;
    let char_count = tail.chars().count();
    if char_count <= MAX_CHARS {
        return tail.to_owned();
    }

    let suffix = tail
        .chars()
        .skip(char_count.saturating_sub(MAX_CHARS))
        .collect::<String>();
    format!("...{suffix}")
}

#[cfg(test)]
mod tests {
    use agent_domain::ModeSlug;

    use super::{SandboxManager, SandboxProfile};

    #[test]
    fn verifier_mode_uses_read_only_profile() {
        let verifier_mode = ModeSlug::new("verifier")
            .unwrap_or_else(|error| panic!("verifier should be a valid mode slug: {error}"));
        assert_eq!(
            SandboxManager::profile_for_mode(&verifier_mode),
            SandboxProfile::ReadOnlyWorkspace
        );
    }
}
