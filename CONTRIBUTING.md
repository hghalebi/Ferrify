# Contributing to Ferrify

## Scope

Ferrify is a governed Rust software-change platform. Contributions should preserve its core constraints:

- typed policy and trust boundaries
- bounded planning and patch scope
- evidence-backed reporting
- explicit approval and verification behavior

## Before You Start

Before opening a large change, make sure the change matches the project direction:

- Ferrify is governance-first, not autonomy-first
- repository evidence should beat remembered conventions
- user-facing claims should be backed by receipts or clearly labeled as inference
- major architectural shifts should be discussed before implementation

## Development Workflow

1. Create a focused branch.
2. Keep changes scoped to one semantic concern.
3. Add or update tests when behavior changes.
4. Run the full verification suite before opening a pull request.
5. Explain any policy or reporting implications in the PR description.

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
- Keep user-facing output honest, concise, and evidence-backed.
- Avoid architectural sprawl when a bounded change will do.

## Pull Requests

- Explain the user-facing or operator-facing outcome.
- Call out any policy, verification, or reporting changes.
- List residual risks or known limitations honestly.
- Keep PRs reviewable; split unrelated work into separate submissions.

## Documentation Changes

If your change affects behavior, update the relevant documentation in the same PR.
For most user-visible changes, that means at least one of:

- `README.md`
- `USER_GUIDE.md`
- `AGENTS.md`
- `.agent/rules/*`
