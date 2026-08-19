# Task 32 — Clean up graph state after failed Application.Run

Status: open
Severity: P2
Difficulty: medium
Language gate: no
Depends on: none

## Goal

Every exit from `Application.Run` clears `run_active` and closes/releases the graph session, including
redraw, event-dispatch, and callback failures.

## Verified cause

`crates/fpas-vm/src/vm/hosted/graph/host.rs` sets `run_active` before entering the run loop. Several
`?` exits can bypass the success-path cleanup, leaving later calls to observe a stale active run.

## Fix

Centralize cleanup in one scope guard or one closure/result epilogue. Cleanup must run exactly once
on success and error and must not replace the original runtime diagnostic with a cleanup error.
Reuse existing `close_graph` behavior; do not add a public test hook.

## Tests

- Force an existing callback or backend error during Run and assert session state is released.
- A second graph acquire/run is not rejected because of stale `run_active`.
- Success and window-close paths still clean up once.

## Verify

```text
cargo test -p fpas-vm
cargo build
cargo test --workspace
cargo fmt
```

## Done when

- Every post-activation return path executes the same cleanup.
- The original failure remains the reported diagnostic.
- Docs unchanged.
