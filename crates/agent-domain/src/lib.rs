//! Core domain types for Ferrify, a governed software-change platform.

mod change;
mod policy;
mod provenance;
mod report;
mod types;

pub use change::{
    ApiImpact, BlastRadius, ChangeIntent, ChangePlan, EvidenceRequirement, OutcomeSpec,
    PatchAnchor, PatchBudget, PatchPlan, RiskItem, RiskLevel, ScopeBoundary, ScopeItem,
    SemanticConcern, TaskKind, VerificationKind, VerificationPlan,
};
pub use policy::{
    ApprovalRule, Capability, DependencyPolicy, EffectivePolicy, PathPattern, PolicyLayer,
    ReportingPolicy, TrustLevel, ValidationMinimums,
};
pub use provenance::{ClassifiedInput, InputRole};
pub use report::{
    ArtifactRef, Assumption, ChangeStatus, ChangeSummary, FinalChangeReport, TouchedArea,
    ValidationReceipt, VerificationStatus,
};
pub use types::{ApprovalProfileSlug, DomainTypeError, ModeSlug, RepoPath};
