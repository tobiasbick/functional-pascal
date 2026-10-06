# AP04.2: Require consumed function results

Package: [AP04: Discarded function values](README.md)

## Scope

Make an unused function result in statement position an error and migrate all
existing statement-position function calls.

## Prerequisites

- AP04.1 (`discard` exists).

## Implementation

- Sema: a function call used as a statement, including the final call of a
  postfix chain, is an error unless its result is consumed.
- The diagnostic shows `discard`; for `Result` values it also shows `case`
  and `try`.
- Procedures and `go` statements are unaffected.

## Affected areas

- `crates/fpas-sema/src/check/stmt/calls.rs` and postfix statement checking.
- `crates/fpas-diagnostics/src/codes.rs`.

## Migration

- Add `discard` (or proper handling) to every statement-position function call
  in `lib/`, `apps/`, `examples/`, `tests/`, Rust-embedded fixtures, templates,
  and documentation examples. Use semantic results to find them.
- Where a discarded `Result` hides a real error path, record it in the pull
  request instead of silently discarding.

## Documentation

- `docs/pascal/language/functions/postfix-chaining.md` (final calls are no
  longer discarded implicitly), routine declarations, error-handling pages.

## Verification

- Tests: unused ordinary value, unused `Result`, unused `Option`, postfix chain
  ending in a function, procedures still valid, hint text.
- Full FPAS suite and example/app checks pass after migration.

## Result

Unused direct, member, intrinsic, callable-variable, and final postfix function
results are rejected with `FP3022`. Hints share AP04.1's task-freedom proofs,
so task-containing values and unverified captures receive consumption guidance
without an invalid discard suggestion. Procedures and `go` retain their behavior.

Repository callers use explicit discard or bind results that cannot be discarded.
Existing ignored cleanup and channel `Result` outcomes retain their behavior;
this migration makes their deliberate omission visible. Task-group starts bind
their returned handles while the existing groups supervise and join the work.

All 53 example/app checks pass, including 22 project checks.
[`interactive_demo.fpas`](../../../../examples/pascal/tui/interactive_demo.fpas)
now explicitly handles `TuiMsg.Started` and `TuiMsg.BackgroundFailed`, completing
its framework-message case. Both arms preserve the model because the demo does
not start background work.
