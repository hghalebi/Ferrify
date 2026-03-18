# Releasing Ferrify

This repository is a Cargo workspace with publishable internal crates. The
binary package `ferrify` depends on those internal crates, so publishing must
follow dependency order.

## Pre-flight checks

Run the full verification suite first:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
cargo test --workspace --doc
```

Then confirm packaging locally:

```bash
cargo publish --dry-run --workspace --allow-dirty
```

## Publish order

Publish the internal crates first:

1. `ferrify-domain`
2. `ferrify-policy`
3. `ferrify-context`
4. `ferrify-evals`
5. `ferrify-infra`
6. `ferrify-syntax`
7. `ferrify-application`
8. `ferrify`

Example:

```bash
cargo publish -p ferrify-domain
cargo publish -p ferrify-policy
cargo publish -p ferrify-context
cargo publish -p ferrify-evals
cargo publish -p ferrify-infra
cargo publish -p ferrify-syntax
cargo publish -p ferrify-application
cargo publish -p ferrify
```

## Notes

- `cargo publish` strips local `path` dependencies when publishing. That is why
  every internal dependency in this workspace carries an explicit version.
- The package names are prefixed with `ferrify-` to avoid generic crates.io
  naming collisions while preserving the existing crate import names inside the
  codebase.
- If crates.io index propagation lags between publishes, wait briefly before
  publishing the next dependent crate.
