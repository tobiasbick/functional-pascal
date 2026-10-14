# Task handles

The code fragments use `uses Std.Tasks as Tasks;`.

A retained task handle has one fixed type determined by the spawned routine:

- `task` is a procedure-task handle with no result.
- `task of T` is a function-task handle whose result has type T.

At runtime both are opaque task identities. Bare `task` never infers a
function result and is not an erased handle. Use the same annotation for
bindings, parameters, fields, aliases, containers, and exported signatures.

```pascal
scope
  const Calculation: task of integer := go Compute(100);
  const Worker: task := go RunWorker();
  const Value: integer := Tasks.Wait(Calculation);
  Tasks.Wait(Worker);
end scope;
```

## Typed task handles

The spawned call and calls supplying its arguments must have known result
types before a handle is created. Inline callback bodies follow their own
ordinary routine checks.
If an argument such as `None` leaves that type unknown, first bind the argument
to an explicit type. A receiving task annotation does not specialize the call.
Generic parameters appearing in the spawned result must be inferred from
arguments or bound through an explicit callable type. An enclosing routine's
own type parameter remains valid when passed as an argument.

In the fragment above, Compute is an integer-returning function and RunWorker
is a procedure. A function-task handle cannot be assigned to `task`, and a
procedure-task handle cannot supply a function result. `Wait` on `task of T`
returns T. The ordinary type checks preserve the result type; no implicit
conversion occurs through the handle. `task` is reserved and cannot name a
variable, field, or unit segment.

```pascal
function Doubled(Job: task of integer): integer;
begin
  return Tasks.Wait(Job) * 2;
end function;

procedure Join(Job: task);
begin
  Tasks.Wait(Job);
end procedure;

scope
  const Jobs: array of task of result of (boolean, string) := [go Connect(), go Serve()];
  Tasks.WaitAll(Jobs);
end scope;
```

Generic parameters may use `task of T` for function tasks; they cannot infer
an internal empty result from a procedure task. A generic over the whole
handle, such as `Identity<T>(Value: T): T`, may still accept `task` as T.
Option, Result, records, enums, and unit interfaces preserve the same
distinction. `Std.Tasks.StartTaskInGroup`
and `StartSupervisedTask` return `task of T` for a function worker and `task`
for a procedure worker. Group ownership is independent of later handle use;
bind the returned handle when its result is required, or explicitly discard it.
[`discard`](../functions/discard.md#task-handles-and-aggregates) accepts handles
and values containing them; it does not cancel, detach, join, or consume results.
Retrieve required data before the owning scope/group releases it.
See [Task groups](../../std/concurrency/task.md#task-groups).

## Waiting for a task

`Std.Tasks.Wait` waits until the task completes and consumes its result of type **`T`**. Child waits
save their continuation and release the pool thread; the main task waits for scheduler notification
while pool workers run queued work. See [Waiting and execution](../../std/concurrency/task.md#waiting-and-execution).

```pascal
scope
  const T: task of integer := go Compute(100);
  const TaskValue: integer := Tasks.Wait(T);
end scope;
```

For a procedure task, Wait is a procedure operation: write `Tasks.Wait(Job);` as
a statement. It cannot supply a value, including inside Some, Ok, an array,
or another call's arguments. No public unit/void value type is needed.

## Waiting for multiple tasks

WaitAll and WaitAny, including controlled WaitAny variants, accept homogeneous
arrays: either `array of task` or `array of task of T` with one result type.
They observe completion without consuming results. Mixed task arrays are
rejected. WaitAll uses the same cooperative child-wait mechanism as Wait:

```pascal
Tasks.WaitAll([T1, T2, T3]);
```

`WaitAll` is a barrier only: it does not consume return values. You may still `Wait` each handle afterward. See [`Std.Tasks`](../../std/concurrency/task.md).

If a retained task fails, `Wait` and `WaitAll` propagate its original runtime or internal diagnostic. A handle of a child owned by a [task scope](task-scopes.md#handle-escapes) cannot leave that scope. If the main task ends while a retained task is still suspended in `Std.Time.Sleep`, teardown cancels that task with a shutdown diagnostic. Wait for required tasks before the main task returns or halts.

## `Std.Tasks`

Per-symbol reference (parameters, edge cases, `Wait` vs `WaitAll`, runtime errors): [`Std.Tasks`](../../std/concurrency/task.md).

| Function | Signature | Description |
|----------|-----------|-------------|
| `Wait` | `(Handle: task of T): T` | Consume a function task's result |
| `Wait` | `(Handle: task)` | Complete a procedure task without a value |
| `WaitAll` | `(Tasks: array of task of T)` or `(Tasks: array of task)` | Observe all homogeneous task completions |

Here T is the function's return type. The procedure form has no value result.

For non-consuming completion observation, use [WaitAny and its controlled variants](../../std/concurrency/task.md#waiting-for-any-task).
For differently typed task results, use [TaskCase and Select](../../std/concurrency/task.md#mixed-source-selection)
and keep the original typed handles for result retrieval. A task scope joins
all its children even when they return different types; outside a shared
scope, wait on the separately typed handles. Observation cannot bypass scope
ownership or expose an erased task result.

## See also

- [`go`](go.md)
- [Task scopes](task-scopes.md)
- [Fork-join](fork-join.md)
- [Scheduling](scheduling.md)
