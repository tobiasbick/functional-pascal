# `Std.Task`

Blocking helpers for **`task`** handles produced by the `go` expression. For the `go` keyword, the `task` type, threading model, and fork-join patterns, see [Concurrency](../../language/concurrency/README.md).

```pascal
program Example;
uses Std.Console, Std.Task;
function N(): integer;
begin
  return 7
end;
begin
  var T: task := go N();
  WriteLn(Wait(T))
end.
```


## Importing and names

After `uses Std.Task;` use short names (`Wait`, `WaitAll`) or qualified (`Std.Task.Wait`, …).

---

## Quick reference

`T` below is the result type of the call that was spawned with `go`.

| Kind | Name | Notes |
|------|------|--------|
| function | `Wait(Handle: task): T` | blocks until the task finishes; **consumes** the handle’s result once |
| procedure | `WaitAll(Tasks: array of task)` | blocks until every task has completed; does **not** consume results — you may still `Wait` each handle afterward |

---

## `function Wait(Handle: task): T`

Blocks until the spawned call completes, then returns its value. The task result is **consumed**: calling `Wait` again on the same logical completion is a runtime error.

```pascal
var T: task := go Square(6);
WriteLn(Wait(T))
```

**Hint:** If you need the result only once, assign `Wait(T)` to a variable and reuse that value.

---

## `procedure WaitAll(Tasks: array of task)`

Blocks until every task in the array has finished. This is a **barrier only**; it does not pop return values. Typical use: synchronize before reading results with `Wait`, or when you only need to know that all work finished.

```pascal
var Ta: task := go Work(1);
var Tb: task := go Work(2);
WaitAll([Ta, Tb]);
// still valid:
Wait(Ta);
Wait(Tb)
```

An empty array completes immediately.

---

## Runtime errors

- **`Wait` after the result was already taken:** wait each task handle at most once for its return value (see VM hint: do not double-await the same completion).
- **Unknown or detached task handle:** `Wait` and `WaitAll` accept only handles returned by retained `go` expressions in the current VM. Forged handles and statement-form detached tasks produce an invalid-task diagnostic instead of waiting indefinitely.
- **Task failure:** `Wait` and `WaitAll` propagate the spawned task's original diagnostic, including its code and source location. The runtime also enters its **failure** path so other spawned work can stop cooperatively. Fix the reported fault in the spawned task.
- **Main-task teardown:** retained tasks still suspended in `Std.Time.Sleep` are completed with a shutdown diagnostic when the main task finishes. Wait for every required result before leaving the main task. See [Scheduling](../../language/concurrency/scheduling.md).

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Registration | [`loaded/channel_task.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/channel_task.rs), [`builtins/channel_task.rs`](../../../../crates/fpas-sema/src/std_registry/builtins/channel_task.rs) |
| Compiler | [`lowering/concurrency.rs`](../../../../crates/fpas-compiler/src/lowering/concurrency.rs), [`lowering/stmt.rs`](../../../../crates/fpas-compiler/src/lowering/stmt.rs), [`bytecode/selection.rs`](../../../../crates/fpas-compiler/src/bytecode/selection.rs) |
| Bytecode | [`instruction.rs`](../../../../crates/fpas-bytecode/src/instruction.rs), [`intrinsic/task.rs`](../../../../crates/fpas-bytecode/src/intrinsic/task.rs) |
| VM | [`tasks/mod.rs`](../../../../crates/fpas-vm/src/vm/tasks/mod.rs), [`tasks/scheduler.rs`](../../../../crates/fpas-vm/src/vm/tasks/scheduler.rs), [`tasks/state.rs`](../../../../crates/fpas-vm/src/vm/tasks/state.rs), [`shared/task_results.rs`](../../../../crates/fpas-vm/src/vm/shared/task_results.rs) |

## See also

- [Concurrency index](README.md)
- [Standard library index](../README.md)
