# ferrify-policy

`ferrify-policy` loads and resolves repository policy for Ferrify.

It is responsible for turning declarative configuration from `.agent/` into the
effective policy used during a run. That includes mode definitions, approval
profiles, capability rules, reporting constraints, and the rule that widening
authority requires explicit approval.

## What This Crate Owns

- `ModeSpec`
- `ApprovalProfile`
- `PolicyRepository`
- `PolicyEngine`
- `ResolvedMode`

## Why It Exists

Ferrify treats repository-local policy as executable configuration, not as
documentation that humans are supposed to remember. This crate keeps that logic
separate from the application layer so policy loading and enforcement stay
reviewable, testable, and versioned with the repository.

## Example

Add the package:

```toml
[dependencies]
ferrify-policy = "0.1.1"
ferrify-domain = "0.1.1"
```

Load repository policy and resolve a mode:

```rust,no_run
use ferrify_domain::ApprovalProfileSlug;
use ferrify_policy::{PolicyEngine, PolicyRepository};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repository = PolicyRepository::load_from_root(std::path::Path::new("."))?;
    let engine = PolicyEngine::new(repository);
    let resolved = engine.resolve("architect", &ApprovalProfileSlug::new("default")?)?;

    assert!(resolved
        .effective_policy
        .allowed_capabilities
        .contains(&ferrify_domain::Capability::ReadWorkspace));

    Ok(())
}
```

## Design Notes

- Default rules are conservative.
- Modes describe what can be attempted.
- Approval profiles describe what requires consent.
- A transition to a broader authority set must be explicitly authorized.

## Relationship To The Workspace

`ferrify-policy` is consumed by `ferrify-application` during orchestration, but
it has no dependency on repository modeling or verification execution. That
separation is intentional: policy should remain declarative and independent of
runtime side effects.
