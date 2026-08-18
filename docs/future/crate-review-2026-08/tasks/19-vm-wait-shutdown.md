# Task 19 — `Wait` / `WaitAll` must return when the runtime aborts

Status: open
Severity: P1
Difficulty: hard
Language gate: no
Depends on: none

## Goal

`Std.Task.Wait` / `WaitAll` must not hot-spin when `shutdown` is set and the handle is still `Pending`. A sibling panic or `ShutdownHandle` during `Wait` must surface a diagnostic and let `Vm::run` leave `thread::scope`.

## Spec

[`docs/pascal/std/concurrency/task.md`](../../../pascal/std/concurrency/task.md) and [`docs/pascal/language/concurrency/scheduling.md`](../../../pascal/language/concurrency/scheduling.md): Wait does not spin; worker panic is the run’s diagnostic; teardown completes retained tasks with a shutdown error.

## Bug

`crates/fpas-vm/src/vm/tasks/mod.rs` Wait loop only returns on `Available` / `Failed` / `Consumed` / `Unknown`. `Pending` calls `help_or_wait_result`. `wait_for_result` in `scheduler.rs` returns as soon as `shutdown` is set even if the slot is still `Pending`. `run_task` treats abort as `Ok(None)` and does not store a result. The Wait loop then spins.

Existing test `shutdown_handle_cancels_register_execution_before_dispatch` in `crates/fpas-vm/src/vm/tests/runtime.rs` cancels **before** `run()`, not during `Wait`.

## Fix

When shutdown/abort is visible and the handle is still `Pending`, return `Failed` with the standard shutdown/worker diagnostic (already used for teardown sleep). Store that failure in the result slot so a second Wait is `Consumed`/`Failed` consistently — match existing “wait once” rules.

Do not busy-loop. Do not change successful Wait.

If helper-task isolation (task 20) is still open, still make this loop **exit**. Attribution can stay imperfect until 20.

## Tests

VM test: spawn a task that `Wait`s another handle; abort/shutdown while still pending; `Vm::run` returns an error in bounded time (use a timeout thread or the existing shutdown handle). Do not hang the test suite.

Also: sibling `panic` while main `Wait`s that sibling or another task — run completes with the panic diagnostic.

## Verify

```text
cargo test -p fpas-vm
cargo fmt
```

If a test hangs, stop and fix; do not ship a sleep-and-hope test.

## Done when

- Wait returns on shutdown/abort.
- No hot-spin (CPU-bound loop without blocking).
- Docs unchanged unless Wait’s shutdown behavior was unspecified — then one sentence on the task page.
