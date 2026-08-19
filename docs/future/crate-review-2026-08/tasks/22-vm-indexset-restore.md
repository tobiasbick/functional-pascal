# Task 22 — Restore the destination after failed IndexSet

Status: open
Severity: P2
Difficulty: medium
Language gate: no
Depends on: none

## Goal

An out-of-bounds `IndexSet` leaves the original aggregate in its destination register. Debugger
post-failure inspection must not observe `Unit` merely because execution used `take` before erroring.

## Verified cause

`crates/fpas-vm/src/vm/execute/aggregates.rs` takes the destination collection before validating the
index and returns an error without restoring it. Ordinary `run()` hides the worker afterward, but a
debug session retains failed state for inspection.

## Fix

Validate without taking ownership, or restore the exact original value on every error path. Reuse a
single helper if array/dictionary index mutation shares the same take/restore pattern. Do not clone
large aggregates merely to cover one branch if mutation can be committed after validation.

## Tests

- Debug-session or worker-level test: execute an out-of-bounds `IndexSet`, inspect the destination,
  and assert the original collection remains intact.
- Existing runtime-error code/location test remains unchanged.

## Verify

```text
cargo test -p fpas-vm
cargo build
cargo test --workspace
cargo fmt
```

## Done when

- Every failed `IndexSet` path preserves the destination value.
- The regression proves post-failure state, not only the diagnostic code.
- Docs unchanged.
