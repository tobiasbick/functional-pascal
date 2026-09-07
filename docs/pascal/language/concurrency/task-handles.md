# Task handles

The `task` type represents a handle to a running task. Assign the result of a **`go` expression** to capture it. For type checking, the handle carries the spawned call’s result type **`T`** (for a procedure spawn, **`T`** is the empty / unit result); at runtime the value is an opaque task id.

```pascal
var T: task := go ComputeSomething(Data);
```

`Std.Task.StartTaskInGroup` and `StartSupervisedTask` also return typed task handles. Their explicit
group retains ownership even when the caller discards a handle; see [Task groups](../../std/concurrency/task.md#task-groups).

## Waiting for a task

`Std.Task.Wait` waits until the task completes and consumes its result of type **`T`**. Child waits
save their continuation and release the pool thread; the main task may help queued work or wait
for scheduler notification. See [Waiting and execution](../../std/concurrency/task.md#waiting-and-execution).

```pascal
var T: task := go Compute(100);
var TaskValue: integer := Wait(T);
```

For a **procedure** task, `Wait` completes when the procedure finishes; **`T`** is the unit type in the type system.

## Waiting for multiple tasks

`Std.Task.WaitAll` waits until all tasks in the array complete, using the same cooperative
child-wait mechanism as `Wait`:

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

For non-consuming completion observation, use [WaitAny and its controlled variants](../../std/concurrency/task.md#function-waitanytasks-array-of-task-integer).
For one event across tasks, channels, timers, and cancellation, use [mixed-source selection](../../std/concurrency/task.md#mixed-source-selection).

## See also

- [`go`](go.md)
- [Fork-join](fork-join.md)
- [Scheduling](scheduling.md)
