# ferrify

`ferrify` is the operator-facing CLI package for the Ferrify workspace.

Install this package when you want to run governed software-change planning,
verification, and reporting from a terminal. The binary reads repository-local
policy, models the current workspace, produces a bounded plan, runs
verification, and prints either structured JSON or a concise human-readable
summary.

## What The CLI Does

- loads `.agent/` policy and approval profiles
- classifies task input and untrusted text
- plans work within an explicit scope boundary
- runs verification commands
- emits evidence-backed reports

## Current Product Boundary

The CLI currently plans, verifies, and reports.
It does not yet apply source edits automatically.

## Installation

```bash
cargo install ferrify --version 0.1.1
```

## Quick Start

Run a scoped planning pass:

```bash
ferrify \
  --goal "tighten CLI reporting surface" \
  --task-kind cli-enhancement \
  --in-scope crates/agent-cli/src/main.rs \
  --auto-approve \
  --json
```

Run the built-in adversarial policy check:

```bash
ferrify --run-adversarial-policy-eval --json
```

## Output Modes

Without `--json`, the CLI prints:

- outcome headline
- design reason
- touched areas
- validation receipts
- scorecards

With `--json`, it also includes:

- classified inputs
- repository model
- working set and context snapshot
- effective policies
- change and patch plans
- full final report
- execution trace

## Validation

Ferrify's standard verification path uses:

```text
cargo fmt --check
cargo check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

## Relationship To The Workspace

This package is the public entry point. The underlying control-plane crates are
published separately so the workspace can be versioned and released
consistently, but `ferrify` is the package most users should start with.
