# Task 26 — Decide the default `fpas test` timeout

Status: decision required (default only)
Severity: policy decision
Difficulty: medium after decision
Language gate: no (CLI policy), but current docs must change
Depends on: none

## Questions

Does `fpas test` use a default timeout when the flag is omitted?

## Why this is blocked

`TestCliConfig.timeout` remains `None` when `--timeout` is omitted. Introducing a default changes
the CLI policy and still requires an explicit decision. Startup coverage is no longer blocked: the
review-remediation request approved treating an explicit timeout as the whole isolated-worker
budget, and the implementation and current docs now enforce that policy.

## Recommended policy for user agreement

- Omitted timeout defaults to 300 seconds, shown in `fpas test --help` and CLI/Test docs.

## Tests after decision

- Argument parsing exposes the selected default.
- Help and CLI/Test documentation expose the selected default.
- Existing explicit-timeout, `--jobs`, and structured report tests retain their behavior.

## Decision record

- Startup included: approved and implemented on 27 August 2026
- Default timeout: pending
- Approved by user: startup policy only
