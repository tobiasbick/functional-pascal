# Task 22 — Restore state on IndexSet OOB and Application.Run errors

Status: open
Severity: P2
Difficulty: medium
Language gate: no
Depends on: none

## Goal

1. Out-of-bounds `IndexSet` must not leave the destination register as `Unit`.
2. A failing `Application.Run` must clear `run_active` and release the session (`close_graph` or equivalent).

## Bug

- `crates/fpas-vm/src/vm/execute/aggregates.rs`: `IndexSet` `take`s the destination, then `Err` on OOB without writing the array back. `run()` hides this; the debugger keeps the worker for inspection.
- `crates/fpas-vm/src/vm/hosted/graph/host.rs`: `run_active` is set before the loop; `?` from redraw/dispatch/`OnPaint` returns without `close_graph`.

Existing `array_index_updates_reject_out_of_bounds_indexes` only checks the error code. Graph tests cover successful `Application.Run` only.

## Fix

IndexSet: on error, put the original collection back (or use a method that does not `take` until success). `try`/`finally` style around the slot.

Graph: `defer`/scope-guard: on any return from `run_active = true`, set it false and close the session. Match the success-path cleanup.

## Tests

- IndexSet OOB: if a debug/inspect test exists for panic leftover state, mirror it; otherwise after catching the error in a `try` (if the language can), read the array and assert it is unchanged. If `try` cannot catch this runtime error, a debugger post-failure inspect test is enough.
- Graph: force `OnPaint` to fail (existing error injection?) and assert a second `Application.Run` is allowed or the error is “no active run” is **not** caused by a stale flag. If headless graph tests can call run twice, do that.

If graph tests are too heavy, ship IndexSet with a test and add a graph unit test that only checks `run_active` via a crate-visible test hook **only if one already exists**. Do not add a new public API for the test.

## Verify

```text
cargo test -p fpas-vm
cargo fmt
```

## Done when

- OOB IndexSet restores the collection.
- Failed Run does not leave `run_active` true.
- Docs unchanged.
