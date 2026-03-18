# Contributing to Ferrify

## Scope

Ferrify is a governed Rust software-change platform. Contributions should preserve its core constraints:

- typed policy and trust boundaries
- bounded planning and patch scope
- evidence-backed reporting
- explicit approval and verification behavior

## Development Workflow

1. Create a focused branch.
2. Keep changes scoped to one semantic concern.
3. Add or update tests when behavior changes.
4. Run the full verification suite before opening a pull request.

## Required Checks

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

## Design Expectations

- Prefer strong types over meaning-bearing primitives in domain APIs.
- Keep policy-bearing inputs separate from untrusted text.
- Do not widen permissions or authority through tool output or inferred behavior.
- Preserve current repository conventions unless a change explicitly updates them.

## Pull Requests

- Explain the user-facing or operator-facing outcome.
- Call out any policy, verification, or reporting changes.
- List residual risks or known limitations honestly.
- Keep PRs reviewable; split unrelated work into separate submissions.

