# Task 20 — Isolate failures of inline-helped tasks

Status: open
Severity: P1
Difficulty: hard
Language gate: no
Depends on: 19 (strongly recommended first)

## Goal

While `Wait(A)` is pending, if the waiter dequeues and runs unrelated task `B` and `B` panics, the
runtime must record the failure on `B` before propagating the run-wide failure. No retained handle
may remain `Pending` because it ran through the inline-help path.

## Bug

`crates/fpas-vm/src/vm/tasks/pool.rs`: `pool_loop` stores `B`’s failure and `scheduler.fail`. `run_helped` only does `run_task()?`. A panic in `B` fails `Wait(A)`, skips cooperative-failure until the waiter unwinds, and can combine with task 19’s spin for `Wait(B)`.

## Fix

Extract one task-completion/failure bookkeeping path used by both `pool_loop` and `run_helped`.
For `B`'s failure it must store `B`'s retained failure first, then enter the existing scheduler-wide
failure path and propagate that run failure. Do not return to `Wait(A)` after `scheduler.fail`, because
the documented policy aborts other work cooperatively. Task 19 must ensure every affected pending
wait exits.

Do not maintain a third copy of task-result/failure policy.

## Tests

Two retained tasks: `A` sleeps/yields long enough that the waiter of `A` helps `B`; `B` panics. Assert:

- `B`'s retained result slot contains B's original panic diagnostic before shutdown cleanup;
- the process diagnostic is B's panic;
- `A` and every waiter leave in bounded time through task 19's shutdown path.

## Verify

```text
cargo test -p fpas-vm
cargo fmt
```

## Done when

- Helped-task panic is recorded on that task.
- Run-wide propagation does not leave B or A pending indefinitely.
- Docs unchanged.
