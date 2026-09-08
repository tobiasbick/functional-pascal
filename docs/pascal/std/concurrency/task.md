# `Std.Task`

Blocking helpers for **`task`** handles, explicit task groups, cooperative cancellation,
and typed bounded channels shared by tasks. For the `go` keyword, the `task` type, threading model,
and fork-join patterns, see [Concurrency](../../language/concurrency/README.md).

## Waiting and execution

Waiting blocks the calling FPAS task, not necessarily its operating-system thread. A child waiting
in `Wait`, `WaitAll`, `WaitAny`, a controlled task wait, a channel operation, `Select`, or
either group-close operation saves its pending operation and releases the pool thread. The shared timer driver
requests another readiness probe after one millisecond; platform timer granularity and scheduler
load can delay the actual probe. This permits producer/consumer work and nested joins with one
pool worker. Suspended values, destinations, and monotonic deadlines survive each probe.

The main task may help queued tasks while waiting. Child waits yield back to that caller rather
than retaining its stack. Hosted blocking I/O and non-cooperative computation can still delay
progress; this is not a hard shutdown or wall-clock deadline guarantee. Debugger execution uses
the same pending-operation state with its deterministic scheduling clock.

`CloseTaskGroupWithTimeout` is an exception to main-task helping: it leaves queued child work to
the pool so arbitrary child code cannot retain the timed caller's stack.

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

`T` below is the task result type or the channel element type, depending on the operation.

| Kind | Name | Notes |
|------|------|--------|
| type | `CancellationSource` | opaque VM-owned cancellation source |
| type | `CancellationToken` | opaque clonable view of one source's state |
| type | `WaitCase` | opaque single-use selection case owned by its creating task |
| type | `TaskGroup` | opaque owner of explicitly registered child tasks |
| type | `TaskFailureKind` | `ReturnedError`, `Panicked`, `RuntimeError`, or `Cancelled` |
| type | `TaskFailure` | child identity, failure kind, message, diagnostic code, and source position |
| function | `CreateTaskGroup(): TaskGroup` | creates a group owned by the calling task |
| function | `StartTaskInGroup(Group: TaskGroup; Work: function(Token: CancellationToken): T): task` | starts a retained child; also accepts a procedure with the same token parameter |
| function | `StartSupervisedTask(Group: TaskGroup; Work: function(Token: CancellationToken): T; RetryLimit: integer; BackoffMillis: integer): task` | starts one group-owned task with bounded retries; also accepts a procedure |
| function | `GetTaskGroupToken(Group: TaskGroup): CancellationToken` | returns the group-owned cancellation token |
| function | `CancelTaskGroup(Group: TaskGroup): boolean` | requests cancellation without joining; true only for the first request |
| function | `CloseTaskGroup(Group: TaskGroup): array of TaskFailure` | cancels, joins registered children, releases their results, and returns failures |
| function | `CloseTaskGroupWithTimeout(Group: TaskGroup; TimeoutMillis: integer): result of array of TaskFailure, string` | attempts cooperative close with a waiting budget; timeout retains group ownership |
| function | `TryCloseCompletedTaskGroup(Group: TaskGroup): option of array of TaskFailure` | closes without cancellation only when every child is already terminal |
| function | `ReceiveCase(Queue: channel of T; Callback: procedure(Outcome: result of T, string)): WaitCase` | describes one receive and its typed delivery callback |
| function | `SendCase(Queue: channel of T; Value: T; Callback: procedure(Outcome: result of boolean, string)): WaitCase` | describes one send without enqueueing its value |
| function | `TaskCase(Handle: task; Callback: procedure()): WaitCase` | describes non-consuming task completion |
| function | `TimerCase(Milliseconds: integer; Callback: procedure()): WaitCase` | describes a timer relative to `Select` entry |
| function | `CancellationCase(Token: CancellationToken; Callback: procedure()): WaitCase` | describes observation of cancellation |
| function | `Select(Cases: array of WaitCase): integer` | commits one operation, runs its callback, then returns its input index |
| function | `CloseWaitCase(Handle: WaitCase): boolean` | discards an unused case; false when already closed or selected |
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

Sources and tokens belong to the VM that created them. Ordinary source storage is released when
that VM ends; group-owned token storage is released when the group is closed.

---

## Task groups

Run the [task-group worker example](../../../../examples/pascal/concurrency/task_group_workers.fpas)
to see cooperative stop and an ordinary error collected by the same group.

### CreateTaskGroup and StartTaskInGroup

`CreateTaskGroup` creates an empty group owned by the calling task. `StartTaskInGroup` registers
one child before publishing its task handle. The worker receives the group's `CancellationToken`
as its only argument and may be a procedure or a function. Captures must be immutable. The task
retains the routine's result type, so `Wait` retrieves its value using the existing task rules.
A worker returning a top-level `result of T, E` must use `string` for `E`.

The creator and registered children may start children in an open group. Unrelated tasks may not.
Ordinary `go` does not implicitly register with a group. Discarding a child's handle does not
detach it: the group keeps ownership until close. Limits are 4096 live groups per VM and 1024
total registered children per group, including children that have already finished.

### StartSupervisedTask

`StartSupervisedTask` has the same ownership, worker-signature, immutable-capture, and result-type
rules as `StartTaskInGroup`. `RetryLimit` is `0..1023` retries after the initial attempt;
`BackoffMillis` is a fixed `0..60000` millisecond pause between failed attempts. Invalid policy
values are rejected before a child is allocated. Zero delay still yields scheduling opportunities.

A successful value or procedure return ends the logical task. Top-level Result errors and Pascal
panics may retry while budget remains. Other runtime diagnostics are terminal. Each attempt
starts with the same immutable captures and group token; local execution state is rebuilt.
External effects such as consumed messages or writes are not rolled back. Workers must account
for partially completed operations themselves.

The task identity remains stable. `Wait`, `WaitAny`, and `Select` observe only the final outcome.
Successful recovery leaves no group failure. Exhaustion preserves the final ordinary error or
panic diagnostic and produces one group failure, not a failure per attempt. With a zero retry
limit, the first failure is final. Message text such as `'cancelled'` is never used to classify errors.

Cancellation is checked before the initial attempt and each retry. Cancellation before a pending
attempt produces a `Cancelled` failure and prevents its body from starting. It also interrupts
backoff: the normal scheduler checks in slices of at most 10 milliseconds of requested timer delay;
the deterministic debugger uses its clock and cancellation checks. Scheduler load can delay actual
execution of those checks, so this is not a hard wall-clock termination guarantee. Already running
workers remain cooperative. A successful return remains successful even if its token was cancelled.

Closing the group requests cancellation and joins its supervised child. To allow retries to finish
naturally, observe the final task result before closing the group. Explicit debugger entry completion
ends the logical task rather than requesting an automatic retry.

See the [supervised-worker example](../../../../examples/pascal/concurrency/supervised_worker.fpas).

### GetTaskGroupToken and CancelTaskGroup

`GetTaskGroupToken` returns the shared cooperative cancellation token. `CancelTaskGroup` requests
cancellation and prevents further child registration, but does not wait for children or release
their results. It returns `true` for the first request and `false` thereafter. Workers must observe
the token themselves or pass it to cancellation-aware operations.

### CloseTaskGroup

Only the creating task may call `CloseTaskGroup`. Close seals registration, requests cancellation,
and waits for all registered children to finish. It then releases the group's membership, token
storage, and retained child results, returning failures in child registration order. Required
successful values must be retrieved with `Wait` **before** close. After close, child results are
consumed and copies of the group token are invalid. Repeated close returns an empty array.

Close is cooperative and has no deadline. A worker that ignores cancellation can delay it
indefinitely. If VM shutdown interrupts close, the call propagates shutdown or the original fatal
diagnostic; synthetic shutdown results do not establish that the underlying workers have joined.

### CloseTaskGroupWithTimeout

Only the creating task may call this operation. It validates the non-negative integer timeout,
seals admission, requests cancellation, and attempts the same join and cleanup as `CloseTaskGroup`.
Invalid arguments or wrong ownership are runtime diagnostics, not timeout results, and do not
request cancellation or seal an otherwise open group.

- `Ok(Failures)` means every registered child has a terminal outcome and the group has been
  closed. Failure ordering, result consumption, and token invalidation match `CloseTaskGroup`.
- `Error('Task group close timed out')` means the waiting budget expired while the group was
  observed incomplete. The group remains sealed and cancelled. Its children, retained results,
  failure reports, and cancellation-token storage remain owned and valid. No worker is detached
  or reported as terminated by the timeout.
- Call either close operation again to finish joining. Each timed call has a new budget; retries
  do not reopen admission or reset cancellation. Successful close is idempotent: later timed
  closes return `Ok([])` and ordinary closes return `[]`.
- Zero requests cancellation and performs one immediate completion probe without waiting or
  executing a queued child inline. An empty or already completed group closes successfully.
- One monotonic deadline is captured per invocation, before requesting cancellation. Wakeups and
  scheduler probes do not restart it. At each probe, observed completion wins over timeout; this
  does not certify that the final child finished before the nominal deadline. VM shutdown or a
  fatal scheduler error remains a runtime diagnostic, even when the timeout has also elapsed.

The main task waits for notifications without executing queued workers itself. Child tasks save
the deadline and yield the pool thread; the debugger uses the same suspension with its clock.
Timer granularity, scheduling delays, and non-cooperative code occupying available execution
threads can delay observation. This is a cooperative waiting budget, not a real-time guarantee,
forced worker termination, or a bound on VM teardown. Keep the VM alive and arrange a later join
after timeout; returning from the main task does not turn the timeout into completed cleanup.

See the [timeout regression](../../../../tests/concurrency/task_group_close_timeout_test.fpas)
for cancellation, retained ownership, and retrying close after releasing a blocked worker.

### TryCloseCompletedTaskGroup

`TryCloseCompletedTaskGroup` is the non-blocking completion probe for an owned group. It does not
request cancellation and does not wait. `None` means at least one registered child is still running;
the group remains open and can still admit work. `Some(Failures)` means every child was already
terminal, so the call atomically seals and closes the group, releases retained child results and the
group token, and returns the same ordered failure records as `CloseTaskGroup`.

An empty group closes immediately as `Some([])`. Repeating the operation after a successful close
also returns `Some([])`. Only the creating task may call it. It is useful for event loops that must
collect grouped panic and runtime diagnostics as data without observing a failed child through
`Wait`, `WaitAny`, or `TaskCase` and without cancelling work merely to poll it.

### TaskFailure and TaskFailureKind

`TaskFailure` has the following readable fields:

| Field | Type | Meaning |
|-------|------|---------|
| `TaskId` | `integer` | identity of the registered child |
| `Kind` | `TaskFailureKind` | failure category |
| `Message` | `string` | message truncated to at most 4096 characters |
| `Code` | `integer` | runtime diagnostic code, or zero for a returned error |
| `Line` | `integer` | runtime source line, or zero for a returned error |
| `Column` | `integer` | runtime source column, or zero for a returned error |

`ReturnedError` means the worker returned a top-level `Error(Message)`. `Panicked` means it
executed `panic`; `RuntimeError` covers other runtime diagnostics; `Cancelled` identifies runtime
cancellation. An ordinary `Error('cancelled')` remains `ReturnedError`: message text does not
determine the category. Successful returns do not produce failure records.

Ordinary error results do not automatically cancel siblings. Group-owned runtime failures are
collected without aborting unrelated work; nongroup task failure behavior is unchanged. Explicit
`Wait` on a failed child still propagates its original diagnostic. Consuming an ordinary Result
with `Wait` does not remove that child's failure from the group's report.

Normal and deterministic debugger execution share these rules. A group-owned debugger failure
emits a task exit event without stopping the parent. The exited child cannot be resumed through
debugger failure recovery after its terminal report has been published.

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

The main task can help queued tasks while waiting; helping may delay its next completion observation.
Child tasks suspend as described above. There is no per-input helper thread and no busy waiting.
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
- Main-task waits request parking intervals of at most 10 ms, shortened to the remaining timeout;
  child waits use the shared suspended-operation path described above.
  This is cooperative, not a hard wall-clock bound: scheduler helping can execute task code that
  blocks or runs for a long time. Control checks resume after that helped work yields or returns.
- Debugger waits use its monotonic clock and explicit suspension. No per-input worker or persistent
  wait registration is created. Completion, timeout, cancellation, failure, and teardown release the
  suspended wait's task list without changing ownership of the tasks themselves.

## Mixed-source selection

Construct cases for the events an operation can handle, then pass them to `Select`. Case construction
does not send, receive, start tasks, or start timers. Channel constructors check the callback against
that channel's element type, so one selection can safely combine differently typed channels.

- `ReceiveCase` delivers `Ok(Value)` or `Error('Channel is closed')` after the closed buffer drains.
- `SendCase` delivers `Ok(true)` after enqueueing its value, or `Error('Channel is closed')`.
  The value must match the channel type and cannot contain a task-bound callable, as with `Send`.
- `TaskCase` observes completion without consuming its value. The callback or caller can still use
  `Wait` under its existing single-consumption rule. Consumed successful tasks still count as complete.
- `TimerCase` requires non-negative milliseconds. All timers in a selection share the monotonic
  start at `Select` entry; time spent holding an unused case does not count toward its deadline.
- `CancellationCase` is ready when its token is cancelled. Selecting cancellation does not cancel
  the other sources, close channels, or stop tasks.

### Ordering and callbacks

`Select` validates its sources and checks task failures before transferring a channel value. It scans
cases in input order and commits the first ready operation at that source. Its return value is the
zero-based input index, delivered only after the winning callback returns. This is ordered observation,
not a simultaneous snapshot or fairness guarantee. Put a cancellation case first when it should take
priority over other ready cases; timer cases participate in the same ordering.

Exactly one callback runs on the selecting task. Losing receives do not remove values and losing
sends do not enqueue values. A wake notification itself neither chooses a winner nor consumes data.
Competing selections serialize channel transfers through the channel's normal lock.

Callbacks must be procedures with the parameter shown above. They may use mutable captures owned by
the selecting task, wait, sleep, or perform a nested selection. Callback failure propagates normally;
the already committed channel operation is not rolled back. Task failures retain their original
runtime diagnostic, and existing runtime-wide failure handling remains active.

### Ownership and limits

A case belongs to its creating VM and task. Copies alias the same case; another task cannot select
or close a live case. `Select` accepts 1..=1024 distinct case identities. It validates the entire array
before atomically claiming it, so an invalid, duplicate, closed, already selected, or wrong-owner
case cannot partially claim the array. Different cases may refer to the same channel or task.

Claimed cases are single-use even if subsequent source validation or callback execution fails.
All losing cases and wait registrations are released before the winning callback starts. The caller's
original immutable send values remain usable. At most 4096 unused cases may belong to one VM;
select or explicitly close cases to release their captured values and callback references.

`CloseWaitCase` returns true only when discarding a live case. Repeated close, or close after selection,
returns false. An invalid handle kind or live case owned by another task is a runtime diagnostic.
VM teardown releases unused cases. Closing a case never closes or cancels its underlying resource.

Pending selection registers with event sources before probing and uses bounded parking, shortened
for the earliest timer. A pending child selection saves its cases and execution state and releases
the pool thread; the shared timer driver schedules another probe with a requested one-millisecond
delay. This is not a wall-clock wakeup guarantee. The main task leaves queued computations to the
worker pool while selecting, so it can continue probing events without running an unrelated child
inline. Selected callbacks still run on the selecting task and may delay its next selection.
`Select` does not guarantee hard wall-clock termination. Debugger selection uses explicit
suspension and its deterministic clock.

Runnable example: [`select_events.fpas`](../../../../examples/pascal/concurrency/select_events.fpas).

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
| Mixed selection | [`tasks/selection/`](../../../../crates/fpas-vm/src/vm/tasks/selection/mod.rs), [`shared/wakeups.rs`](../../../../crates/fpas-vm/src/vm/shared/wakeups.rs), [`callbacks/`](../../../../crates/fpas-vm/src/vm/hosted/callbacks/mod.rs) |
| Task groups | [`tasks/groups/`](../../../../crates/fpas-vm/src/vm/tasks/groups/mod.rs), [`scheduler/group_results.rs`](../../../../crates/fpas-vm/src/vm/tasks/scheduler/group_results.rs), [`driver/failure.rs`](../../../../crates/fpas-vm/src/vm/debug/tasks/driver/failure.rs) |
| Supervision | [`tasks/supervision/`](../../../../crates/fpas-vm/src/vm/tasks/supervision/mod.rs), [`tasks/pool.rs`](../../../../crates/fpas-vm/src/vm/tasks/pool.rs) |

## See also

- [Concurrency index](README.md)
- [Standard library index](../README.md)
