# Domain Path Rules

- `crates/agent-domain/**` stays free of filesystem and process execution logic.
- Public types in this path need doc comments because other crates depend on them as contracts.
- Domain types should preserve Ferrify's governance model without leaking boundary primitives.
