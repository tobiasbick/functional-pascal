# Task 26 — Decide `fpas test` timeout policy

Status: decision required
Severity: policy decision
Difficulty: medium after decision
Language gate: no (CLI policy), but current docs must change
Depends on: none

## Questions

1. Does `--timeout` cover worker startup/preparation, or only VM execution after the worker reports
   ready?
2. Does `fpas test` use a default timeout when the flag is omitted?

## Why this is blocked

`cli_test/process/mod.rs::wait_until_ready` has no deadline, and `TestCliConfig.timeout` remains
`None` when omitted. That is unfriendly to unattended agents, but the current
[`Std.Test`](../../../pascal/std/testing/test.md) documentation explicitly says the timeout starts
only when the worker is ready, and CLI docs do not promise a default. The original plan incorrectly
classified both changes as bugfixes against the current spec.

## Recommended policy for user agreement

- `--timeout` is a wall-clock budget beginning immediately after successful worker spawn and covers
  readiness, hooks, and the test body.
- Omitted timeout defaults to 300 seconds, shown in `fpas test --help` and CLI/Test docs.
- Timeout kills the existing process tree and uses the existing timeout outcome/exit mapping.

## Tests after decision

- Argument parsing exposes the selected default.
- A worker that never writes `ready` times out and is terminated.
- Existing body/hook timeout, `--jobs`, help, and structured report tests use the same policy.

## Decision record

- Startup included: pending
- Default timeout: pending
- Approved by user: pending
