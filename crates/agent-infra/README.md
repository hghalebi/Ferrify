# ferrify-infra

`ferrify-infra` defines Ferrify's runtime boundary.

It contains the abstractions and starter implementations for sandbox profile
selection, verification execution, and brokered tool access. If another crate
needs to cross from pure control-plane logic into processes or external tools,
it should usually do so through the interfaces defined here.

## What This Crate Owns

- `SandboxProfile`
- `SandboxManager`
- `ToolRequest`
- `ToolReceipt`
- `ToolBroker`
- `VerificationBackend`
- `ProcessVerificationBackend`

## Why It Exists

Ferrify separates policy from execution on purpose. A system cannot make
credible claims about safety or verification if the authority model and the
runtime boundary are intertwined.

`ferrify-infra` keeps those edges explicit.

## Example

Add the packages:

```toml
[dependencies]
ferrify-domain = "0.1.1"
ferrify-infra = "0.1.1"
```

Select the runtime profile for a mode:

```rust
use ferrify_domain::ModeSlug;
use ferrify_infra::{SandboxManager, SandboxProfile};

let mode = ModeSlug::new("verifier").expect("valid mode");

assert_eq!(
    SandboxManager::profile_for_mode(&mode),
    SandboxProfile::ReadOnlyWorkspace
);
```

## Design Notes

- Unknown modes default to the conservative read-only profile.
- Tool access is brokered rather than invoked directly.
- Verification returns receipts, not just pass or fail booleans.

## Relationship To The Workspace

This crate is intentionally operational but minimal. Richer sandboxing and tool
execution can grow here later without forcing the policy or domain crates to
depend on process management details.
