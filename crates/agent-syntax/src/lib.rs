//! Patch planning and budget enforcement.
//!
//! `agent-syntax` is where Ferrify turns a broad change plan into a narrower
//! patch plan. In the current starter implementation it does not rewrite source
//! files. Instead, it enforces the patch budget and produces explicit anchors
//! that explain why each file was selected.
//!
//! That distinction matters: the rest of the workspace can already reason
//! about bounded implementation, verification, and reporting without pretending
//! that AST-level edits exist before they do.
//!
//! # Examples
//!
//! ```
//! use std::collections::BTreeSet;
//!
//! use agent_domain::{
//!     ApiImpact, BlastRadius, ChangeIntent, ChangePlan, OutcomeSpec, PatchBudget, RepoPath,
//!     ScopeBoundary, SemanticConcern, TaskKind, VerificationKind, VerificationPlan,
//! };
//! use agent_syntax::PatchPlanner;
//!
//! # fn main() -> Result<(), agent_domain::DomainTypeError> {
//! let mut target_files = BTreeSet::new();
//! target_files.insert(RepoPath::new("crates/agent-cli/src/main.rs")?);
//!
//! let mut required = BTreeSet::new();
//! required.insert(VerificationKind::CargoCheck);
//!
//! let change_plan = ChangePlan {
//!     intent: ChangeIntent {
//!         task_kind: TaskKind::CliEnhancement,
//!         goal: "tighten CLI reporting".to_owned(),
//!         desired_outcome: OutcomeSpec {
//!             summary: "narrow the CLI plan".to_owned(),
//!         },
//!         scope_boundary: ScopeBoundary {
//!             in_scope: Vec::new(),
//!             out_of_scope: Vec::new(),
//!             blast_radius_limit: BlastRadius::Small,
//!         },
//!         success_evidence: Vec::new(),
//!         primary_risks: Vec::new(),
//!     },
//!     concern: SemanticConcern::FeatureAdd,
//!     target_files,
//!     selected_mode: "implementer".parse()?,
//!     api_impact: ApiImpact::InternalOnly,
//!     patch_budget: PatchBudget {
//!         max_files: 1,
//!         max_changed_lines: 40,
//!         allow_manifest_changes: false,
//!     },
//!     verification_plan: VerificationPlan { required },
//!     notes: vec!["limit the edit to the CLI entrypoint".to_owned()],
//! };
//!
//! let patch_plan = PatchPlanner::build(&change_plan);
//! assert_eq!(patch_plan.target_files.len(), 1);
//! # Ok(())
//! # }
//! ```

use std::collections::BTreeSet;

use agent_domain::{ChangePlan, PatchAnchor, PatchPlan};

/// Builds bounded patch plans from architect-stage change plans.
#[derive(Debug, Default)]
pub struct PatchPlanner;

impl PatchPlanner {
    /// Converts a change plan into a patch plan while enforcing the file budget.
    #[must_use]
    pub fn build(change_plan: &ChangePlan) -> PatchPlan {
        let max_files = usize::from(change_plan.patch_budget.max_files);
        let target_files = if max_files == 0 {
            BTreeSet::new()
        } else {
            change_plan
                .target_files
                .iter()
                .take(max_files)
                .cloned()
                .collect::<BTreeSet<_>>()
        };

        let anchors = target_files
            .iter()
            .map(|file| PatchAnchor {
                file: file.clone(),
                reason: format!(
                    "Selected during planning for {:?} within the active patch budget.",
                    change_plan.concern
                ),
            })
            .collect();

        PatchPlan {
            concern: change_plan.concern,
            target_files,
            anchors,
            budget: change_plan.patch_budget.clone(),
            api_impact: change_plan.api_impact,
            required_validation: change_plan.verification_plan.clone(),
        }
    }
}
