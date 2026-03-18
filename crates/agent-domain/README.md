# ferrify-domain

`ferrify-domain` defines the core vocabulary for Ferrify.

This crate exists so the rest of the workspace can talk about policy, scope,
trust, provenance, plans, and reports using explicit Rust types instead of
meaning-bearing raw primitives. If Ferrify's control plane has a language, this
crate is that language.

## What This Crate Owns

- validated identifiers such as `RepoPath`, `ModeSlug`, and `ApprovalProfileSlug`
- policy and trust types such as `PolicyLayer`, `TrustLevel`, and `Capability`
- intake and planning types such as `ChangeIntent`, `ChangePlan`, and `PatchPlan`
- provenance labels such as `InputRole` and `ClassifiedInput`
- reporting types such as `FinalChangeReport` and `ValidationReceipt`

## Why It Exists

Ferrify is built around explicit governance. That only works if the most
important invariants are encoded as values that can be validated early and
carried across crate boundaries without ambiguity.

For example:

- a repository path must not escape the workspace
- a mode name must be a stable slug, not arbitrary text
- a final report must be able to point to concrete verification receipts

`ferrify-domain` makes those constraints structural.

## Example

Add the package:

```toml
[dependencies]
ferrify-domain = "0.1.1"
```

Use the validated value objects:

```rust
use ferrify_domain::{ModeSlug, RepoPath, TrustLevel};

fn main() -> Result<(), ferrify_domain::DomainTypeError> {
    let target = RepoPath::new("crates/agent-cli/src/main.rs")?;
    let mode = ModeSlug::new("architect")?;

    assert_eq!(target.as_str(), "crates/agent-cli/src/main.rs");
    assert_eq!(mode.as_str(), "architect");
    assert!(TrustLevel::RepoPolicy.can_define_policy());

    Ok(())
}
```

## Design Notes

- Prefer types that explain intent over generic strings and booleans.
- Keep serialization straightforward so plans and reports remain easy to emit as
  JSON.
- Treat policy, trust, and evidence as first-class data rather than prompt
  conventions.

## Relationship To The Workspace

Every other Ferrify crate depends on this crate either directly or indirectly.
If a type belongs to the shared control-plane model, it should usually live
here rather than in an orchestration or infrastructure crate.
