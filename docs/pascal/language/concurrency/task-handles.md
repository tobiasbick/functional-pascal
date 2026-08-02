# Task handles

The `task` type represents a handle to a running task. Assign the result of a **`go` expression** to capture it. For type checking, the handle carries the spawned call’s result type **`T`** (for a procedure spawn, **`T`** is the empty / unit result); at runtime the value is an opaque task id.

```pascal
var T: task := go ComputeSomething(Data);
```

## Waiting for a task

`Std.Task.Wait` blocks until the task completes and returns its result type **`T`** (the runtime waits on the same shared condition variable as the task queue — it does not hot-spin):

```pascal
var T: task := go Compute(100);
var TaskValue: integer := Wait(T);
```

For a **procedure** task, `Wait` completes when the procedure finishes; **`T`** is the unit type in the type system.

## Waiting for multiple tasks

`Std.Task.WaitAll` blocks until all tasks in the array complete (same condvar-based blocking as `Wait`):

```pascal
WaitAll([T1, T2, T3]);
```

`WaitAll` is a barrier only: it does not consume return values. You may still `Wait` each handle afterward. See [`Std.Task`](../../std/concurrency/task.md).

If a retained task fails, `Wait` and `WaitAll` propagate its original runtime or internal diagnostic. If the main task ends while a retained task is still suspended in `Std.Time.Sleep`, teardown cancels that task with a shutdown diagnostic. Wait for required tasks before the main task returns or halts.

## `Std.Task`

Per-symbol reference (parameters, edge cases, `Wait` vs `WaitAll`, runtime errors): [`Std.Task`](../../std/concurrency/task.md).

| Function | Signature | Description |
|----------|-----------|-------------|
| `Wait` | `(Handle: task): T` | Wait for a task and return its result |
| `WaitAll` | `(Tasks: array of task)` | Wait for all tasks to complete |

Here, **`T`** is the return type of the spawned call (unit for a procedure).

## See also

- [`go`](go.md)
- [Fork-join](fork-join.md)
- [Scheduling](scheduling.md)
