# AGENTS.md

Ferrify is a governed Rust software-change platform. The codebase prefers typed policy, explicit trust boundaries, deterministic planning, and evidence-backed reporting over prompt-only behavior.

## Mission

- Keep policy, mode, and evidence concerns explicit in Rust types.
- Prefer repo-local configuration under `.agent/` over hardcoded defaults when both exist.
- Treat code, compiler output, and tool output as distinct inputs with distinct trust levels.
- Classify inputs by role; only task goals and repository policy may influence authority decisions.

## Dependency posture

- Add dependencies only when they materially reduce complexity in the control plane.
- Keep the public API small and typed.
- Prefer serde-backed config formats over ad hoc parsing.

## Verification minimums

- `cargo fmt --check`
- `cargo check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`

## Reporting contract

- Do not claim a fix unless a verification receipt exists or the claim is labeled as inference.
- Preserve residual risks when checks fail or when the plan had to infer target files.
- Keep scope and patch-budget decisions visible in the final report.
