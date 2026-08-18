# Task 20 — Isolate failures of inline-helped tasks

Status: open
Severity: P1
Difficulty: hard
Language gate: no
Depends on: 19 (strongly recommended first)

## Goal

While `Wait(A)` is pending, if the waiter dequeues and runs unrelated task `B` and `B` panics, `Wait(A)` must not fail with `B`’s diagnostic while leaving `B`’s result `Pending`.

## Bug

`crates/fpas-vm/src/vm/tasks/pool.rs`: `pool_loop` stores `B`’s failure and `scheduler.fail`. `run_helped` only does `run_task()?`. A panic in `B` fails `Wait(A)`, skips cooperative-failure until the waiter unwinds, and can combine with task 19’s spin for `Wait(B)`.

## Fix

`run_helped` must treat `B`’s failure the same as `pool_loop`: store result/failure for `B`, `scheduler.fail` if that is the policy for worker panics, and **return to the Wait(A) loop** unless the runtime is shutting down because of that failure. If the documented policy is “any worker panic aborts the whole run”, then Wait(A) should get the **run** diagnostic after `B`’s slot is filled — not a Pending `B` plus a mis-attributed `Wait(A)`.

Read pool_loop and match it. Do not duplicate a third failure path.

## Tests

Two retained tasks: `A` sleeps/yields long enough that the waiter of `A` helps `B`; `B` panics. Assert:

- `B`’s handle is failed (Wait(B) sees B’s panic), and/or
- the process diagnostic is B’s panic,
- `Wait(A)` is not reported as B’s error **while A is still running**, unless the whole VM is aborting **and** A’s wait returns (task 19).

## Verify

```text
cargo test -p fpas-vm
cargo fmt
```

## Done when

- Helped-task panic is recorded on that task.
- Wait(A) does not steal B’s error as its own result value.
- Docs unchanged.
