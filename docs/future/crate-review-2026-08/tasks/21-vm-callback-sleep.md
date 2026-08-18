# Task 21 — Hosted callbacks must not inherit the owner task id for Sleep/Yield

Status: open
Severity: P1
Difficulty: hard
Language gate: no
Depends on: none

## Goal

`go` + `Std.Array.Map(..., procedure that Sleeps)` is a valid program. It must not emit `Root register execution suspended unexpectedly` or enqueue a ghost task under the owner’s id.

## Spec

[`docs/pascal/std/host/time.md`](../../../pascal/std/host/time.md): Sleep in spawned tasks is cooperative. Callbacks (`Std.Array.Map`, graph `OnPaint`) are synchronous nested work, not a second spawn with the same id.

## Bug

`crates/fpas-vm/src/vm/callback_call.rs`: synchronous callbacks copy `parent.task_id` onto a nested worker and `run_in_place`. If `parent.task_id != 0`, `Sleep`/`Yield` call `take_task_state` / `suspend_and_enqueue` on the **callback** worker (same id as the still-running owner), then `run_in_place` errors.

## Fix

Give the nested callback worker a **distinct** id of 0 (in-place, non-schedulable) **or** a nested id that Sleep/Yield treat as “block the callback, do not steal the owner”. Sleep inside a callback on a spawned task should block that callback like main-thread Sleep (or cooperatively wait **without** detaching the owner). Pick the option that matches existing main-task Sleep, and document it in a short `//` comment.

Do not change Map’s callback semantics (still synchronous). Do not start a real extra pool task per Map element unless that is already how callbacks work.

## Tests

Compiler/VM or `tests/` program: `go` a procedure that `Std.Array.Map` over `[1]` with a callback that `Sleep(1)` then returns; `Wait` that task; program succeeds.

If Map callback types make that awkward, a smaller hook: any `run_in_place` callback path already tested — add Sleep there.

## Verify

```text
cargo test -p fpas-vm
cargo test -p fpas-compiler
cargo fmt
```

Plus the FPAS file if you added one.

## Done when

- Sleep/Yield in a callback on a spawned task does not ICE/internal-error.
- Owner task still completes.
- Docs unchanged unless you chose “Sleep in callbacks is always blocking” — then one sentence on the Time or Array Map page.
