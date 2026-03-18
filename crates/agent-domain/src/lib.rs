//! Core domain types for Ferrify.
//!
//! `agent-domain` is the vocabulary crate for the rest of the workspace. It
//! defines the value objects, planning records, policy types, provenance
//! labels, and reporting structures that let Ferrify describe a governed
//! software-change run without reaching into filesystem or process concerns.
//!
//! The main design goal is to keep meaning-bearing concepts explicit. A path
//! that must stay inside the repository is represented by [`RepoPath`], not a
//! raw `String`. A mode identifier is a validated [`ModeSlug`], not a free-form
//! label. The result is a control plane that can encode authority and scope in
//! the type system before any command is run.
//!
//! # Core Concepts
//!
//! - [`PolicyLayer`], [`TrustLevel`], and [`Capability`] describe who may do
//!   what, and why.
//! - [`ChangeIntent`], [`ChangePlan`], and [`PatchPlan`] carry work from intake
//!   into a bounded implementation strategy.
//! - [`InputRole`] and [`ClassifiedInput`] explain how Ferrify separates
//!   operator goals, repository policy, code, evidence, and untrusted text.
//! - [`FinalChangeReport`] and [`ValidationReceipt`] make reporting
//!   evidence-backed instead of speculative.
//!
//! # Examples
//!
//! ```
//! use agent_domain::{ModeSlug, RepoPath, TrustLevel};
//!
//! # fn main() -> Result<(), agent_domain::DomainTypeError> {
//! let target = RepoPath::new("crates/agent-cli/src/main.rs")?;
//! let mode = ModeSlug::new("architect")?;
//!
//! assert_eq!(target.as_str(), "crates/agent-cli/src/main.rs");
//! assert_eq!(mode.as_str(), "architect");
//! assert!(TrustLevel::RepoPolicy.can_define_policy());
//! # Ok(())
//! # }
//! ```

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
