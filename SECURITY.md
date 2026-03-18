# Security Policy

## Reporting a Vulnerability

If you believe you have found a security issue in Ferrify, do not open a public issue with exploit details.

Instead:

1. Use GitHub's private security reporting flow if it is available for the repository.
2. Otherwise, share a private report with reproduction steps, impact, and affected files.
2. Include the Ferrify version or commit if known.
3. Describe any required configuration or approval mode needed to trigger the issue.

## Scope

Security reports are especially useful for issues involving:

- policy bypass
- approval bypass
- prompt or tool-output injection impact
- sandbox boundary assumptions
- unsafe path handling
- destructive command or file access escalation

## Response Expectations

Reports should receive acknowledgement and triage as quickly as practical. Public disclosure should wait until the issue has been understood and a fix or mitigation is ready.

## What Helps a Good Report

- the exact command you ran
- whether `--auto-approve` or another approval flag was used
- whether the issue depends on `.agent/` repository policy
- whether the issue came from tool output, path handling, or verification logic
