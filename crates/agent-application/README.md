# ferrify-application

`ferrify-application` orchestrates a governed Ferrify run.

This crate is the layer that wires repository modeling, policy resolution,
patch planning, verification, trace grading, and final reporting into one
coherent flow. It does not own repository parsing or process execution itself.
Instead, it coordinates the surrounding crates and returns a structured
`RunResult`.

## What This Crate Owns

- `RunRequest`
- `RunResult`
- `GovernedAgent`
- `ApplicationError`

## Execution Stages

A typical run goes through these stages:

1. classify task input and untrusted text
2. build a repository model
3. resolve mode and approval policy
4. create a bounded change plan
5. derive a patch plan
6. run verification
7. build an evidence-backed final report

## Example

Add the packages:

```toml
[dependencies]
ferrify-application = "0.1.1"
ferrify-domain = "0.1.1"
ferrify-infra = "0.1.1"
ferrify-policy = "0.1.1"
```

Run the orchestrator:

```rust,no_run
use std::collections::BTreeSet;
use std::path::PathBuf;

use ferrify_application::{GovernedAgent, RunRequest};
use ferrify_domain::{ApprovalProfileSlug, Capability, TaskKind};
use ferrify_infra::ProcessVerificationBackend;
use ferrify_policy::{PolicyEngine, PolicyRepository};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repository = PolicyRepository::load_from_root(std::path::Path::new("."))?;
    let engine = PolicyEngine::new(repository);
    let agent = GovernedAgent::new(engine, ProcessVerificationBackend);

    let result = agent.run(RunRequest {
        root: PathBuf::from("."),
        goal: "tighten CLI reporting surface".to_owned(),
        task_kind: TaskKind::CliEnhancement,
        in_scope: vec!["crates/agent-cli/src/main.rs".to_owned()],
        out_of_scope: Vec::new(),
        approval_profile: ApprovalProfileSlug::new("default")?,
        approval_grants: [Capability::EditWorkspace].into_iter().collect::<BTreeSet<_>>(),
        untrusted_texts: Vec::new(),
    })?;

    println!("{}", result.final_report.outcome.headline);
    Ok(())
}
```

## Design Notes

- The application layer should stay orchestration-focused.
- Policy and trust decisions must remain visible in returned data.
- A successful run is a verified plan unless the runtime actually performs
  edits.

## Relationship To The Workspace

If `ferrify-domain` defines the language of a run, `ferrify-application`
defines the sequence in which that language is used.
