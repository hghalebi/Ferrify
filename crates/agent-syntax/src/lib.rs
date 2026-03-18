//! Patch planning and budget enforcement.

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
