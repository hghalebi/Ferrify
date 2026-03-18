# Ferrify User Guide

## What Ferrify Does

Ferrify helps you make safer, more predictable software changes in a Rust project.

Instead of jumping straight into edits, Ferrify:

- understands the repository first
- keeps changes within a clear scope
- checks what is allowed before doing risky work
- runs verification steps
- gives you a report you can review

Today, Ferrify is best described as a governed planning and verification assistant.
It does not automatically rewrite your source files yet.

## What You Get Back

When Ferrify runs well, you get:

- a clear summary of what it understood
- a list of files it selected
- the checks it ran
- a final status such as `Verified` or `PartiallyVerified`
- any leftover risks you should still review

## Why You Might Use It

Ferrify is useful when you want to:

- plan a change without touching unrelated files
- verify that a change idea fits the repo’s rules
- see what would be affected before editing
- keep approvals and risk decisions visible
- get a structured report instead of vague agent output

## Before You Start

You need:

- a Rust project
- a terminal
- Ferrify in this workspace

If your repository includes Ferrify policy files under `.agent/`, Ferrify will use them automatically.

## The Simplest Way To Think About Ferrify

Ferrify does three practical things:

1. It reads the repo and figures out the shape of the work.
2. It checks what is allowed before moving forward.
3. It verifies what it can prove and tells you what still needs judgment.

## Your First Run

This example asks Ferrify to inspect a CLI change and return a structured report:

```bash
cargo run -p ferrify -- \
  --goal "tighten CLI reporting surface" \
  --task-kind cli-enhancement \
  --in-scope crates/agent-cli/src/main.rs \
  --auto-approve \
  --json
```

## What the Main Options Mean

- `--goal`: what you want done, in plain language
- `--task-kind`: the kind of work, such as bug fix or refactor
- `--in-scope`: the files or modules Ferrify is allowed to focus on
- `--out-of-scope`: areas Ferrify should avoid
- `--auto-approve`: approve approval-gated capabilities for the run
- `--json`: return the full machine-readable report

## Choosing a Task Kind

- `bug-fix`: when something behaves incorrectly
- `feature-add`: when you want new behavior
- `refactor`: when you want structure changes without changing behavior
- `cli-enhancement`: when the CLI is the focus
- `dependency-change`: when manifests or dependency posture matter
- `test-hardening`: when the goal is stronger verification
- `reliability-hardening`: when failure handling or safety margins matter
- `scaffold`: when you want a starter structure or initial plan

## Common Examples

### Check a planned CLI change

```bash
cargo run -p ferrify -- \
  --goal "tighten CLI reporting surface" \
  --task-kind cli-enhancement \
  --in-scope crates/agent-cli/src/main.rs \
  --auto-approve
```

### Review dependency-related scope

```bash
cargo run -p ferrify -- \
  --goal "evaluate dependency posture" \
  --task-kind dependency-change \
  --auto-approve \
  --json
```

### Test that unsafe authority requests are blocked

```bash
cargo run -p ferrify -- --run-adversarial-policy-eval --json
```

### Ask Ferrify to stay out of an area

```bash
cargo run -p ferrify -- \
  --goal "review CLI behavior without touching policy code" \
  --task-kind cli-enhancement \
  --in-scope crates/agent-cli/src/main.rs \
  --out-of-scope crates/agent-policy \
  --auto-approve
```

## How to Read the Result

Ferrify reports a few things clearly:

- `outcome`: whether the run was planned, verified, partial, or failed
- `touched_areas`: the files Ferrify selected for the task
- `validations`: the checks it ran and whether they passed
- `residual_risks`: anything you should still review carefully

If you use `--json`, you also get the repo model, policy resolution, trace, and scorecards.

## What `Verified` Really Means

`Verified` means Ferrify has receipts for the checks it ran and those checks passed.
It does not mean Ferrify rewrote your code or that human review is unnecessary.

## Approvals, in Plain English

Some actions are more sensitive than others.

Ferrify can require approval before it:

- edits files
- uses risky commands
- deletes files
- accesses the network

If you do not grant the needed approval, Ferrify will stop and tell you why.

## Common Problems

### "requires approval"

Ferrify hit an action that the current mode is not allowed to do without approval.
Either grant the needed approval or run a narrower task.

### "must not contain parent-directory traversal"

One of your scope paths tried to escape the repository, for example `../outside`.
Use a repository-relative path instead.

### "invalid CLI input"

One of your arguments did not pass Ferrify's input validation.
Check spelling, slug format, and path format.

## What Ferrify Will Not Do

Ferrify is intentionally strict.

It will not:

- let untrusted text silently change its authority
- accept scope paths that try to escape the repository
- claim success without verification evidence
- hide when a plan had to infer scope or trim work

## FAQ

### Does Ferrify change my code automatically?

Not yet. The current version plans, scopes, verifies, and reports. Automatic source editing is not the current behavior.

### Is Ferrify only for Rust?

The current workspace and control logic are Rust-first.
The product itself is aimed at governed software-change workflows, but the current implementation is designed around Rust repositories.

### Do I need to understand the whole repo first?

No. Ferrify is designed to help you explore a repo safely before making changes.

### What if I only want a narrow change?

Use `--in-scope` to point Ferrify at the exact file or module you care about.

### What if I want strict boundaries?

Use both `--in-scope` and `--out-of-scope`. Ferrify will preserve that boundary through planning.

### What if a run fails?

Ferrify will tell you whether the failure came from approval rules, invalid input, or a verification step.

### Do I need JSON output?

No. Human-readable output is enough for quick review. Use `--json` when you want the full structured report.

## Tips for Better Results

- write goals in plain language
- keep scope small when possible
- use approval flags deliberately, not by habit
- read the final report before acting on it
- treat residual risks as real review items, not boilerplate
- start with small, explicit scope before asking for broad work
