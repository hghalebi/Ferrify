use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{ModeSlug, RepoPath};

/// The kind of change the operator is requesting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskKind {
    /// Fix an incorrect behavior.
    BugFix,
    /// Add new user-visible behavior.
    FeatureAdd,
    /// Restructure code without changing behavior.
    Refactor,
    /// Extend or adjust CLI behavior.
    CliEnhancement,
    /// Add, remove, or update a dependency.
    DependencyChange,
    /// Increase verification coverage.
    TestHardening,
    /// Improve failure handling or safety margins.
    ReliabilityHardening,
    /// Create an initial project or subsystem skeleton.
    Scaffold,
}

impl TaskKind {
    /// Maps a task kind to the semantic concern used by patch planning.
    #[must_use]
    pub fn concern(self) -> SemanticConcern {
        match self {
            Self::BugFix => SemanticConcern::BugFix,
            Self::FeatureAdd | Self::CliEnhancement => SemanticConcern::FeatureAdd,
            Self::Refactor => SemanticConcern::Refactor,
            Self::DependencyChange => SemanticConcern::BuildChange,
            Self::TestHardening | Self::ReliabilityHardening => SemanticConcern::TestHardening,
            Self::Scaffold => SemanticConcern::FeatureAdd,
        }
    }
}

/// A repository item used to bound scope.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ScopeItem(pub RepoPath);

/// The maximum acceptable blast radius for a change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlastRadius {
    /// The change should stay tightly scoped.
    Small,
    /// The change may span a small subsystem.
    Medium,
    /// The change may touch multiple subsystems.
    Large,
}

/// The desired outcome stated for the task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutcomeSpec {
    /// A short human-readable outcome description.
    pub summary: String,
}

/// A verification requirement requested by the operator or planner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRequirement {
    /// The verification step that should produce the evidence.
    pub kind: VerificationKind,
    /// Why this evidence matters for the task.
    pub detail: String,
}

/// The severity attached to a residual risk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// The risk is worth noting but not blocking.
    Low,
    /// The risk could affect correctness or scope.
    Medium,
    /// The risk could invalidate the outcome.
    High,
}

/// A risk item preserved across planning and reporting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskItem {
    /// The severity of the risk.
    pub level: RiskLevel,
    /// A concise explanation of the risk.
    pub summary: String,
}

/// The explicit scope boundary for a task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeBoundary {
    /// Items the change is allowed to touch.
    pub in_scope: Vec<ScopeItem>,
    /// Items the change must not touch.
    pub out_of_scope: Vec<ScopeItem>,
    /// The allowed blast radius for the change.
    pub blast_radius_limit: BlastRadius,
}

/// The intake object that describes the requested software change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeIntent {
    /// The broad category of work.
    pub task_kind: TaskKind,
    /// The operator's natural-language goal.
    pub goal: String,
    /// The outcome that the run is trying to produce.
    pub desired_outcome: OutcomeSpec,
    /// Explicit scope constraints for the run.
    pub scope_boundary: ScopeBoundary,
    /// Evidence required before the run can claim success.
    pub success_evidence: Vec<EvidenceRequirement>,
    /// Risks that must survive into the final report.
    pub primary_risks: Vec<RiskItem>,
}

/// The semantic concern for a patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticConcern {
    /// Correct a defect.
    BugFix,
    /// Add new behavior.
    FeatureAdd,
    /// Rearrange existing code.
    Refactor,
    /// Improve or extend tests.
    TestHardening,
    /// Change manifests or dependency settings.
    BuildChange,
    /// Limit the patch to documentation.
    DocsOnly,
}

/// The expected API impact for the change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiImpact {
    /// No API impact is expected.
    None,
    /// Only internal APIs should change.
    InternalOnly,
    /// Public APIs may change compatibly.
    PublicCompatible,
    /// The change is expected to break public APIs.
    PublicBreaking,
}

/// The patch budget used to keep edits bounded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PatchBudget {
    /// Maximum number of files the patch may touch.
    pub max_files: u16,
    /// Maximum number of changed lines.
    pub max_changed_lines: u32,
    /// Whether manifest changes are allowed.
    pub allow_manifest_changes: bool,
}

/// The verification steps supported by Ferrify.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum VerificationKind {
    /// Run `cargo fmt --check`.
    CargoFmtCheck,
    /// Run `cargo check`.
    CargoCheck,
    /// Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
    CargoClippy,
    /// Run the currently configured test command.
    TargetedTests,
}

/// The verification steps required for a plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct VerificationPlan {
    /// Verification steps that should run before the report is emitted.
    #[serde(default)]
    pub required: BTreeSet<VerificationKind>,
}

/// The architect-stage change plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangePlan {
    /// The original intake object.
    pub intent: ChangeIntent,
    /// The semantic concern chosen for the run.
    pub concern: SemanticConcern,
    /// Repository-relative files selected for the change.
    pub target_files: BTreeSet<RepoPath>,
    /// The mode that produced the plan.
    pub selected_mode: ModeSlug,
    /// The expected API impact.
    pub api_impact: ApiImpact,
    /// The patch budget inherited from policy.
    pub patch_budget: PatchBudget,
    /// Verification required for the task.
    pub verification_plan: VerificationPlan,
    /// Planner notes that justify the chosen scope.
    pub notes: Vec<String>,
}

/// A file-level anchor used by patch planning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchAnchor {
    /// The anchored file.
    pub file: RepoPath,
    /// Why this file was selected.
    pub reason: String,
}

/// The implementer-stage patch plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchPlan {
    /// The semantic concern carried into patching.
    pub concern: SemanticConcern,
    /// The files the patch is allowed to touch.
    pub target_files: BTreeSet<RepoPath>,
    /// File-level anchors that explain the scope.
    pub anchors: Vec<PatchAnchor>,
    /// The patch budget enforced during implementation.
    pub budget: PatchBudget,
    /// The expected API impact of the patch.
    pub api_impact: ApiImpact,
    /// Verification required after patching.
    pub required_validation: VerificationPlan,
}
