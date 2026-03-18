//! Final-report and verification-receipt types.
//!
//! These types are what make Ferrify's reporting contract concrete. A final
//! status is paired with individual [`ValidationReceipt`] values and preserved
//! risks so the runtime can say exactly what it observed instead of implying
//! more certainty than the evidence supports.

use serde::{Deserialize, Serialize};

use crate::{RepoPath, RiskItem, VerificationKind};

/// The result of an individual verification step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// The verification step succeeded.
    Succeeded,
    /// The verification step failed.
    Failed,
    /// The verification step did not run.
    Skipped,
}

/// A reference to evidence captured during a verification step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRef {
    /// A short label for the artifact.
    pub label: String,
    /// Where the artifact can be found.
    pub location: String,
}

/// A receipt proving which verification command ran and how it ended.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReceipt {
    /// The verification step that produced this receipt.
    pub step: VerificationKind,
    /// The command that was executed.
    pub command: String,
    /// The observed result.
    pub status: VerificationStatus,
    /// Artifacts captured during execution.
    pub artifacts: Vec<ArtifactRef>,
}

/// The overall status for the final report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeStatus {
    /// The run produced a plan but no verification evidence.
    Planned,
    /// The run produced mixed or incomplete verification evidence.
    PartiallyVerified,
    /// The run produced the expected verification evidence.
    Verified,
    /// The run failed to produce acceptable evidence.
    Failed,
}

/// The high-level change outcome shown to the operator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeSummary {
    /// The status of the run.
    pub status: ChangeStatus,
    /// A short human-readable summary.
    pub headline: String,
}

/// A touched area that should appear in the final report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TouchedArea {
    /// The repository-relative path for the touched area.
    pub path: RepoPath,
    /// Why the area was touched.
    pub reason: String,
}

/// An assumption that influenced the run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assumption {
    /// The assumption preserved for the operator.
    pub summary: String,
}

/// The evidence-backed final report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalChangeReport {
    /// The overall outcome summary.
    pub outcome: ChangeSummary,
    /// The design rationale used for planning.
    pub design_reason: String,
    /// Areas the plan or patch selected.
    pub touched_areas: Vec<TouchedArea>,
    /// Verification receipts collected by the runtime.
    pub validations: Vec<ValidationReceipt>,
    /// Assumptions that still matter for review.
    pub assumptions: Vec<Assumption>,
    /// Risks that remain after the run.
    pub residual_risks: Vec<RiskItem>,
}
