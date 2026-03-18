# ferrify-syntax

`ferrify-syntax` turns a broad change plan into a bounded patch plan.

The crate currently focuses on patch planning and budget enforcement. It does
not yet apply edits to source files. That boundary is important: the workspace
can already reason about target files, anchors, budgets, and required
verification without pretending that AST-level editing exists before it is
implemented.

## What This Crate Owns

- `PatchPlanner`

The rich patch data types themselves live in `ferrify-domain`; this crate
provides the policy-aware transformation from a `ChangePlan` to a `PatchPlan`.

## Example

Add the packages:

```toml
[dependencies]
ferrify-domain = "0.1.1"
ferrify-syntax = "0.1.1"
```

Build a bounded patch plan:

```rust
use std::collections::BTreeSet;

use ferrify_domain::{
    ApiImpact, BlastRadius, ChangeIntent, ChangePlan, OutcomeSpec, PatchBudget, RepoPath,
    ScopeBoundary, SemanticConcern, TaskKind, VerificationKind, VerificationPlan,
};
use ferrify_syntax::PatchPlanner;

fn main() -> Result<(), ferrify_domain::DomainTypeError> {
    let mut target_files = BTreeSet::new();
    target_files.insert(RepoPath::new("crates/agent-cli/src/main.rs")?);

    let mut required = BTreeSet::new();
    required.insert(VerificationKind::CargoCheck);

    let change_plan = ChangePlan {
        intent: ChangeIntent {
            task_kind: TaskKind::CliEnhancement,
            goal: "tighten CLI reporting".to_owned(),
            desired_outcome: OutcomeSpec {
                summary: "narrow the CLI plan".to_owned(),
            },
            scope_boundary: ScopeBoundary {
                in_scope: Vec::new(),
                out_of_scope: Vec::new(),
                blast_radius_limit: BlastRadius::Small,
            },
            success_evidence: Vec::new(),
            primary_risks: Vec::new(),
        },
        concern: SemanticConcern::FeatureAdd,
        target_files,
        selected_mode: "implementer".parse()?,
        api_impact: ApiImpact::InternalOnly,
        patch_budget: PatchBudget {
            max_files: 1,
            max_changed_lines: 40,
            allow_manifest_changes: false,
        },
        verification_plan: VerificationPlan { required },
        notes: vec!["limit the edit to the CLI entrypoint".to_owned()],
    };

    let patch_plan = PatchPlanner::build(&change_plan);
    assert_eq!(patch_plan.target_files.len(), 1);

    Ok(())
}
```

## Relationship To The Workspace

This crate sits between the architect-stage plan in `ferrify-application` and
the eventual future edit engine. For now, it is the place where budget and
patch radius become concrete.
