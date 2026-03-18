# Test Path Rules

- Tests should favor small fixtures and temp workspaces.
- Avoid tests that recursively invoke workspace-wide checks from inside `cargo test`.
- Prefer example-shaped regression tests for user-facing CLI behavior.
