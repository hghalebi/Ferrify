# ferrify-context

`ferrify-context` models a repository and produces the bounded working context
used by Ferrify planning stages.

The crate follows a structural-first scan order. It starts with workspace and
toolchain roots, then reads policy artifacts, and only then expands into crate
manifests and nearby source context. That ordering keeps Ferrify grounded in
current repository evidence instead of remembered conventions.

## What This Crate Owns

- `RepoModel`
- `RepoModeler`
- `WorkingSet`
- `ContextBudget`
- `ContextSnapshot`

## Why It Exists

Repository-aware tools often fail because they read too little or remember too
much. `ferrify-context` exists to make context selection deliberate:

- preserve structural facts
- cap the active working set
- keep open questions visible
- avoid carrying irrelevant repo chatter into later stages

## Example

Add the package:

```toml
[dependencies]
ferrify-context = "0.1.1"
```

Scan a repository:

```rust,no_run
use ferrify_context::RepoModeler;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = RepoModeler::scan(Path::new("."))?;
    println!("workspace kind: {:?}", model.workspace_kind);
    println!("crates discovered: {}", model.crates.len());
    Ok(())
}
```

## Design Notes

- The crate is read-only by design.
- Structural roots are more important than distant source files.
- Context compaction should preserve facts, decisions, and active failures, not
  raw exploration noise.

## Relationship To The Workspace

`ferrify-application` consumes the `RepoModel` and `WorkingSet` from this crate
before planning. The policy and syntax crates stay separate so context
selection does not implicitly change authority or patch scope.
