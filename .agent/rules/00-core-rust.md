# Core Rust Rules

- Keep policy and planning logic in typed Rust structures rather than free-form strings.
- Use small enums and newtypes for authority, trust, and verification concepts.
- Prefer repository evidence over remembered conventions.
- Tag inputs by role so policy-bearing content stays separate from untrusted evidence.
- Comments should explain non-obvious rationale, not narrate the code.
