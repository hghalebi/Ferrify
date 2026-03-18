# Ferrify

Ferrify is a governed Rust software-change platform with typed policy, bounded planning, and evidence-backed reporting.

## Status

Ferrify is early-stage and intentionally opinionated. The current runtime focuses on governed planning, policy enforcement, verification, and traceable reporting. It does not yet apply source edits automatically.

## Overview

- Typed control planes for policy, context, planning, and reporting
- Approval-gated mode transitions and capability checks
- Structural-first repository modeling
- Evidence-backed verification and trace grading

## Workspace

- `crates/agent-domain`: domain types for policy, planning, provenance, and reports
- `crates/agent-policy`: mode specs, approval profiles, and policy enforcement
- `crates/agent-context`: repository scanning and context selection
- `crates/agent-application`: intake, orchestration, and final report generation
- `crates/agent-syntax`: patch planning and edit-budget enforcement
- `crates/agent-infra`: verification backend, sandbox profile selection, and tool broker types
- `crates/agent-evals`: trace graders and evaluation helpers
- `crates/agent-cli`: the `ferrify` binary

## Quick Start

```bash
cargo run -p ferrify -- \
  --goal "tighten CLI reporting surface" \
  --task-kind cli-enhancement \
  --in-scope crates/agent-cli/src/main.rs \
  --auto-approve \
  --json
```

## Verification

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

## Community

- [Contributing](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Security Policy](SECURITY.md)

## GitHub Remote

```bash
git remote add origin git@github.com:hghalebi/Ferrify.git
git branch -M main
git push -u origin main
```

## License

Ferrify is dual-licensed under MIT or Apache-2.0.
See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
