# ferrify-evals

`ferrify-evals` grades Ferrify runs.

The crate provides small, explicit types for execution traces and scorecards,
plus the first built-in grader: an honesty check that penalizes reports claiming
more certainty than the recorded evidence supports.

## What This Crate Owns

- `TraceStage`
- `TraceEvent`
- `TraceRecord`
- `Scorecard`
- `TraceGrader`
- `HonestyGrader`

## Why It Exists

An agentic runtime should be judged by its behavior, not just by whether it
produced output. `ferrify-evals` makes that measurable.

The current crate is intentionally small, but it establishes the contract for:

- trace-based evaluation
- honesty grading
- broader golden and adversarial task grading over time

## Example

Add the packages:

```toml
[dependencies]
ferrify-domain = "0.1.1"
ferrify-evals = "0.1.1"
```

Grade a verified report:

```rust
use ferrify_domain::{
    ChangeStatus, ChangeSummary, FinalChangeReport, ValidationReceipt,
    VerificationKind, VerificationStatus,
};
use ferrify_evals::{HonestyGrader, TraceGrader, TraceRecord, TraceStage};

let mut trace = TraceRecord::default();
trace.push(TraceStage::Verify, "verification completed");

let report = FinalChangeReport {
    outcome: ChangeSummary {
        status: ChangeStatus::Verified,
        headline: "verified".to_owned(),
    },
    design_reason: "example".to_owned(),
    touched_areas: Vec::new(),
    validations: vec![ValidationReceipt {
        step: VerificationKind::CargoCheck,
        command: "cargo check".to_owned(),
        status: VerificationStatus::Succeeded,
        artifacts: Vec::new(),
    }],
    assumptions: Vec::new(),
    residual_risks: Vec::new(),
};

let scorecard = HonestyGrader.grade(&trace, &report);
assert_eq!(scorecard.score, 100);
```

## Relationship To The Workspace

This crate is consumed by `ferrify-application`, but it stays pure and
side-effect free. That makes it easy to reuse for regression harnesses or
future evaluation tooling.
