# `Std.Task`

Blocking helpers for **`task`** handles produced by the `go` expression, cooperative cancellation,
and typed bounded channels shared by tasks. For the `go` keyword, the `task` type, threading model,
and fork-join patterns, see [Concurrency](../../language/concurrency/README.md).

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

After `uses Std.Task;` use short names (`Wait`, `Cancel`, …) or qualified (`Std.Task.Wait`, …).

---

## Quick reference

`T` below is the result type of the call that was spawned with `go`.

| Kind | Name | Notes |
|------|------|--------|
| type | `CancellationSource` | opaque VM-owned cancellation source |
| type | `CancellationToken` | opaque clonable view of one source's state |
| function | `CreateCancellationSource(): CancellationSource` | creates an active source |
| function | `GetCancellationToken(Source: CancellationSource): CancellationToken` | returns a token linked to the source |
| function | `Cancel(Source: CancellationSource): boolean` | requests cancellation; true only for the first request |
| function | `IsCancellationRequested(Token: CancellationToken): boolean` | reads the shared cancellation state |
| function | `CreateChannel(Capacity: integer): channel of T` | creates a VM-owned bounded channel; capacity is `1..=1048576` |
| function | `Send(Queue: channel of T; Value: T): result of boolean, string` | blocks while full; returns an error after close |
| function | `TrySend(Queue: channel of T; Value: T): result of boolean, string` | sends immediately; `Ok(false)` means the open channel is full |
| function | `SendWithCancellation(Queue: channel of T; Value: T; Token: CancellationToken): result of boolean, string` | send that also observes cancellation |
| function | `SendWithTimeout(Queue: channel of T; Value: T; TimeoutMillis: integer): result of boolean, string` | sends before a relative monotonic deadline |
| function | `Receive(Queue: channel of T): result of T, string` | blocks while empty and open |
| function | `TryReceive(Queue: channel of T): result of option of T, string` | receives immediately; `Ok(None)` means the open channel is empty |
| function | `ReceiveWithCancellation(Queue: channel of T; Token: CancellationToken): result of T, string` | receive that also observes cancellation |
| function | `ReceiveWithTimeout(Queue: channel of T; TimeoutMillis: integer): result of T, string` | receives before a relative monotonic deadline |
| function | `CloseChannel(Queue: channel of T): boolean` | closes and wakes waiters; true only for the first close |
| function | `Wait(Handle: task): T` | blocks until the task finishes; **consumes** the handle’s result once |
| procedure | `WaitAll(Tasks: array of task)` | blocks until every task has completed; does **not** consume results — you may still `Wait` each handle afterward |
| function | `WaitAny(Tasks: array of task): integer` | returns the lowest completed input index without consuming results |
| function | `WaitAnyWithTimeout(Tasks: array of task; TimeoutMillis: integer): result of integer, string` | completion index or a distinct timeout error |
| function | `WaitAnyWithCancellation(Tasks: array of task; Token: CancellationToken): result of integer, string` | completion index or a distinct cancellation error |

---

## Cooperative cancellation

`CreateCancellationSource` creates one active state. `GetCancellationToken` returns an opaque token
that may be copied and passed to worker tasks. `Cancel` atomically changes the shared state and
returns `true`; later calls return `false`. `IsCancellationRequested` is non-blocking.

Cancellation is a request, not forced task termination. A task or hosted blocking operation stops
only when it checks the token. The cancellation-aware `Std.Net` connect, accept, read, and write
operations observe tokens while their interruptible network phases are pending.

```pascal
var Source: CancellationSource := CreateCancellationSource();
var Token: CancellationToken := GetCancellationToken(Source);
Cancel(Source);
if IsCancellationRequested(Token) then
begin
  WriteLn('stopping')
end
```

Sources and tokens belong to the VM that created them. Their storage is released when that VM ends.

---

## Bounded channels

`CreateChannel` creates a FIFO queue with fixed capacity. The declaration supplies the element type
because capacity alone cannot infer `T`:

```pascal
var Messages: channel of string := CreateChannel(16)
```

`Send` waits until space is available. `Receive` waits until a value is available. Successful sends
return `Ok(true)`; successful receives return `Ok(Value)`. Values are received in send order.

```pascal
case Send(Messages, 'ready') of
  Ok(_): begin end;
  Error(Message): panic(Message)
end;

case Receive(Messages) of
  Ok(Message): WriteLn(Message);
  Error(Message): panic(Message)
end
```

`CloseChannel` is idempotent: the first close returns `true`, and later closes return `false`.
Buffered values remain receivable after close. Once drained, `Receive` returns
`Error('Channel is closed')`; `Send` returns that error immediately. Closing or VM shutdown wakes
blocked senders and receivers.

The cancellable variants additionally observe a `CancellationToken`. They return
`Error('Channel send was cancelled')` or `Error('Channel receive was cancelled')`. Cancellation
does not close the channel. If cancellation is already requested when an operation starts, the
cancellation result takes precedence.

`TrySend` and `TryReceive` never wait. `TrySend` returns `Ok(false)` when the channel is open but
full. `TryReceive` returns `Ok(None)` when it is open but empty and `Ok(Some(Value))` after receiving
a value. A closed channel still returns `Error('Channel is closed')`, so closure is distinct from a
temporary full or empty state.

`SendWithTimeout` and `ReceiveWithTimeout` take a relative, non-negative millisecond duration. The
runtime converts it to a monotonic deadline once, so wakeups and scheduler work do not restart the
timeout. A zero timeout performs one immediate attempt. An available slot or buffered value wins
that attempt; otherwise the operation returns `Error('Channel send timed out')` or
`Error('Channel receive timed out')`. A timeout does not close or otherwise change the channel.

Channel handles belong to their creating VM. A channel accepts only its declared element type, and
task-bound values with mutable captures cannot cross the channel boundary. See
[Channel types](../../language/types/channels.md).

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

## `function WaitAny(Tasks: array of task): integer`

Waits for at least one successful task completion and returns its zero-based input position.
The array must contain between 1 and 1,048,576 retained task handles. It follows the same
task-array typing rules as `WaitAll`; duplicate handles are allowed.

The runtime validates every identity before selecting a result. Invalid identities take precedence
over task failures; visible failures take precedence over successful completion. Among failures,
the first in input order is propagated with its original diagnostic. Among successful completions,
the lowest input position wins. This ordering describes one synchronized observation, not the
physical order in which workers finished, and does not promise fairness.

`WaitAny` does not consume results or cancel losing tasks. A successful result already consumed
by `Wait` still counts as complete, as with `WaitAll`; waiting for its value again remains an error.
Existing runtime-wide worker-failure handling remains active.

```pascal
var First: integer := WaitAny([Ta, Tb]);
// Both results still belong to their task handles.
WaitAll([Ta, Tb]);
Wait(Ta);
Wait(Tb)
```

The worker can help queued tasks while waiting; helping may delay its next completion observation.
There is no per-input helper thread and no busy polling. Debugger execution suspends cooperatively.
VM shutdown releases pending waits through the existing task-failure path.

## Controlled task-completion waits

`WaitAnyWithTimeout` and `WaitAnyWithCancellation` use the same non-empty bounded task list,
input ordering, and non-consuming completion policy as `WaitAny`. Success returns `Ok(Index)`.
Timeout returns `Error('Task wait timed out')`; cancellation returns
`Error('Task wait was cancelled')`. Neither outcome cancels tasks or consumes their results.

- Timeout milliseconds must be non-negative. One monotonic budget starts after argument validation;
  wakeups and scheduler helping do not reset it. Zero performs one immediate completion observation.
- In that initial observation, a ready task wins over timeout. In subsequent observations, an
  expired budget wins over a successful completion, even if it became available while the worker
  was busy helping another task. Completions are not timestamped.
- A cancelled token wins over successful completion, including on entry. Invalid task identities
  and task failures are checked first; they remain runtime diagnostics, not ordinary Result errors.
  Invalid tokens and timeouts also produce runtime diagnostics.
- Pending scheduler waits park for at most 10 ms between checks, shortened to the remaining timeout.
  This is cooperative, not a hard wall-clock bound: scheduler helping can execute task code that
  blocks or runs for a long time. Control checks resume after that helped work yields or returns.
- Debugger waits use its monotonic clock and explicit suspension. No per-input worker or persistent
  wait registration is created. Completion, timeout, cancellation, failure, and teardown release the
  suspended wait's task list without changing ownership of the tasks themselves.

## Runtime errors

- **`Wait` after the result was already taken:** wait each task handle at most once for its return value (see VM hint: do not double-await the same completion).
- **Unknown or detached task handle:** `Wait`, `WaitAll`, and `WaitAny` accept only handles returned by retained `go` expressions in the current VM. Forged handles and statement-form detached tasks produce an invalid-task diagnostic instead of waiting indefinitely.
- **Invalid wait-any size:** `WaitAny` requires between 1 and 1,048,576 task handles.
- **Task failure:** `Wait` and `WaitAll` propagate the spawned task's original diagnostic, including its code and source location. The runtime also enters its **failure** path so other spawned work can stop cooperatively. Fix the reported fault in the spawned task.
- **Main-task teardown:** retained tasks still suspended in `Std.Time.Sleep` are completed with a shutdown diagnostic when the main task finishes. Wait for every required result before leaving the main task. See [Scheduling](../../language/concurrency/scheduling.md).
- **Invalid cancellation handle:** sources and tokens must come from the current VM and be passed to
  the function matching their static type.
- **Invalid channel capacity:** `CreateChannel` accepts only `1..=1048576`.
- **Closed channel:** sends fail immediately; receives first drain buffered values and then fail.
- **Cancelled channel operation:** only `SendWithCancellation` and `ReceiveWithCancellation`
  observe their token, and cancellation leaves the channel open.
- **Invalid channel timeout:** timeout milliseconds must be non-negative.
- **Timed-out channel operation:** timeout variants return distinct send and receive errors and
  leave the channel open.
- **Invalid channel handle:** channel handles must come from the current VM. Forged or foreign
  opaque handles produce a runtime diagnostic.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Registration | [`loaded/channel_task.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/channel_task.rs), [`builtins/channel_task.rs`](../../../../crates/fpas-sema/src/std_registry/builtins/channel_task.rs) |
| Compiler | [`lowering/concurrency.rs`](../../../../crates/fpas-compiler/src/lowering/concurrency.rs), [`lowering/stmt.rs`](../../../../crates/fpas-compiler/src/lowering/stmt.rs), [`bytecode/selection.rs`](../../../../crates/fpas-compiler/src/bytecode/selection.rs) |
| Bytecode | [`instruction.rs`](../../../../crates/fpas-bytecode/src/instruction.rs), [`intrinsic/task.rs`](../../../../crates/fpas-bytecode/src/intrinsic/task.rs) |
| VM | [`tasks/mod.rs`](../../../../crates/fpas-vm/src/vm/tasks/mod.rs), [`tasks/scheduler.rs`](../../../../crates/fpas-vm/src/vm/tasks/scheduler.rs), [`tasks/state.rs`](../../../../crates/fpas-vm/src/vm/tasks/state.rs), [`shared/task_results.rs`](../../../../crates/fpas-vm/src/vm/shared/task_results.rs) |
| Cancellation | [`cancellation/registry.rs`](../../../../crates/fpas-vm/src/vm/cancellation/registry.rs), [`tasks/cancellation.rs`](../../../../crates/fpas-vm/src/vm/tasks/cancellation.rs) |
| Channels | [`channels/registry.rs`](../../../../crates/fpas-vm/src/vm/channels/registry.rs), [`tasks/channel.rs`](../../../../crates/fpas-vm/src/vm/tasks/channel.rs) |
| Completion selection | [`tasks/wait_any.rs`](../../../../crates/fpas-vm/src/vm/tasks/wait_any.rs), [`scheduler/result_polling.rs`](../../../../crates/fpas-vm/src/vm/tasks/scheduler/result_polling.rs) |

## See also

- [Concurrency index](README.md)
- [Standard library index](../README.md)
