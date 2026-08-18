# Task 26 — `fpas test` timeouts must bound worker startup (and have a default)

Status: open
Severity: P1 (startup) / P2 (no default)
Difficulty: medium
Language gate: no
Depends on: none

## Goal

1. `--timeout` applies to isolated-worker startup (`wait_until_ready`), not only after `start`.
2. `fpas test` without `--timeout` still cannot hang forever on an infinite-loop test. Use a documented default (debug already uses 300s). If a default would break long integration tests in this repo, pick a large default (300s) and document it — do not leave `None`.

## Spec / CLI for agents

Bounded CI, `--timeout` means wall clock for the whole wait. [`docs/pascal/program-structure/cli.md`](../../../pascal/program-structure/cli.md) and test-runner help.

## Bug

- `crates/fpas-cli/src/cli_test/process/mod.rs`: `wait_until_ready` loops until `ready` or exit; deadline is armed only after the parent writes `start`. Hang in image decode / `apply_test_script` ignores `--timeout`.
- `crates/fpas-cli/src/cli_input/mod.rs`: `timeout` stays `None` unless the flag is passed → in-process runner, infinite-loop `*_test.fpas` hangs.

## Fix

Arm the deadline before waiting for `ready` (include worker spawn). On timeout, kill the worker the same way as test-body timeout.

Set a default timeout consistent with `fpas debug` (300s) unless help text already promises “no default”. Update `--help` and cli.md to state the default. Isolated workers (`--jobs`) must use the same default.

Do not change exit-code mapping except timeout → the existing timeout code.

## Tests

- Timeout tests already cover infinite loops after the body starts — keep them.
- Add: worker never writes `ready` (mock or a stub binary if the test harness allows injecting a fake worker). If that is too heavy, unit-test `wait_until_ready` with a reader that never sends `ready` and a 50ms timeout.
- Default: parse `fpas test` args and assert `timeout` is `Some(300s)` or whatever you chose.

## Verify

```text
cargo test -p fpas-cli
cargo fmt
```

## Done when

- Startup is covered by the deadline.
- Help/docs show the default.
- Existing timeout tests pass.
