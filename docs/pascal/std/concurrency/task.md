# `Std.Tasks`

Blocking helpers for **`task`** handles, explicit task groups, cooperative cancellation,
and typed bounded channels shared by tasks. For the `go` keyword, the `task` type, threading model,
and fork-join patterns, see [Concurrency](../../language/concurrency/README.md).

Fixed-arity task operations accept all positional or all named arguments using
the labels in the API tables below, including polymorphic operations such as
`Send(Value := 3, Queue := Queue)` and `Wait(Handle := Job)`.
Named values evaluate in written order before they are mapped to parameter slots.

Known task-bound callables, arguments, send values and task results are rejected
with FP3016. Unknown capture state is accepted by the checker and checked at the
runtime boundary, including for imported factory results. Group and supervised
starts follow the same rule. Selection callbacks execute on their selecting task
and may capture its mutable locals; a `SendCase` value must still be transferable.
See [Capturing closures](../../language/functions/closures.md#concurrency).

## Waiting and execution

Waiting blocks the calling FPAS task, not necessarily its operating-system thread. A child waiting
in `Wait`, `WaitAll`, `WaitAny`, a controlled task wait, a channel operation, `Select`, or
either group-close operation saves its pending operation and releases the pool thread. The shared timer driver
requests another readiness probe after one millisecond; platform timer granularity and scheduler
load can delay the actual probe. This permits producer/consumer work and nested joins with one
pool worker. Suspended values, destinations, and monotonic deadlines survive each probe.

[Socket setup, timeout configuration, accepts and byte I/O](../network/net.md#task-execution-and-resource-ownership)
also release a pool worker, using bounded native jobs. Inline callbacks retain
their surrounding operation while that task waits. Main-program and synchronous
native callback calls keep their synchronous behavior.

[Process calls](../host/proc.md#blocking-and-concurrency), including startup, all Run
variants, terminal-aware execution and handle close, use bounded native jobs in pool
tasks and the all-stop debugger. Host terminal providers and stderr receivers run on
the native job. Abandonment terminates the owned group and joins cleanup; unpublished
handles are closed. Host calls already in progress must return before joining finishes.

[Filesystem calls](../host/fs.md#execution-and-cancellation), including handle,
whole-file and path operations, use bounded native jobs in pool tasks and the
all-stop debugger. Abandonment joins
started file work and closes undelivered opens; it cannot roll back I/O or forcibly
interrupt an OS or library call already in progress. Unpublished temporary paths
are removed; published temporary-path strings transfer cleanup to the caller.

[Console I/O and screen state](../console/README.md#task-execution-and-ownership)
also release pool workers and ordinary all-stop debugger tasks. Owned native jobs
preserve order per output, text-input or keyboard resource; at capacity, calls
queue before native admission. Unpublished saved regions are discarded. Terminal
readiness and lock waits observe cancellation, but host reader/writer calls
already in progress must return before joining finishes.

[ACP startup](../ai/acp/connections.md#setup) and
[host adapters](../ai/acp/host.md#execution-and-response-rights) use bounded native
jobs too, including terminal authentication and nested callbacks. Undelivered
connections and terminals remain owned by their job and are closed on abandonment.
Native resource admission precedes host request claims and OS effects.

[Cryptographic randomness](../cryptography/crypto.md#execution-and-ownership) and
[PRNG seeding](../numeric/random.md#execution-and-ownership) also release workers
through bounded native entropy jobs. Seeded PRNG draws remain in-memory calls.
[MCP publication](../ai/mcp/README.md#ownership-and-execution) covers native executor
initialization as well as listener setup, including the first publication.

The main task never executes queued tasks while it waits; pool workers run them. A queued child
may wait for the main task itself, for example for a request it sends on a channel or socket, so
running that child on the main task's thread could deadlock until the child's I/O times out, and a
timed wait could not observe its budget. The main task parks until a notification or its next
probe. Hosted blocking I/O and non-cooperative computation in children can still occupy every pool
worker and delay progress; this is not a hard shutdown or wall-clock deadline guarantee. Debugger
execution uses the same pending-operation state with its deterministic scheduling clock.

```pascal id=example-1
program Example;

uses Std.Console as Console, Std.Tasks as Tasks;

function N(): integer;
begin
  return 7;
end function;

begin
  scope
    const T: task of integer := go N();
    Console.WriteLn(Tasks.Wait(T));
  end scope;
end.
```


## Importing and names

Import with `uses Std.Tasks as Tasks;` and access public symbols as `Tasks.Name`. User enum variants require `Tasks.Type.Variant`.

---

## Operational errors

`TaskError` is a native error family with `Closed(Operation)`, `Timeout(Operation)`,
`Cancelled(Operation)`, `InvalidArgument(Name, Reason)` and `InvalidGroup(Operation)`.
Operation is the qualified attempted API name. Channel selection callbacks carry
this same family; closed `ReceiveCase` and `SendCase` outcomes name their case
constructor. Negative timeout values and invalid controlled-wait list sizes return
InvalidArgument; foreign group-close ownership returns InvalidGroup for the timed
Result operation. Fabricated task/group identities and worker failures remain
runtime diagnostics. Infallible operations keep their existing return shapes.

Match variants for decisions. Call `Cause.Message()` only at an output or a
public boundary that deliberately returns text. A timeout retains ownership;
a returned TaskError never replaces the separate TaskFailure report.

## Quick reference

T below is the function-task result type or the channel element type.
Bare `task` means a procedure task with no result. Task-aware operations also
accept that procedure form as described below; it is never an erased function
handle. Fixed-arity task operations accept all positional or all named arguments,
with no mixture within one call.

| Kind | Name | Notes |
|------|------|--------|
| type | `CancellationSource` | opaque VM-owned cancellation source |
| type | `CancellationToken` | opaque clonable view of one source's state |
| type | `WaitCase` | opaque single-use selection case owned by its creating task |
| type | `TaskGroup` | opaque owner of explicitly registered child tasks |
| type | `TaskFailureKind` | `Panicked`, `RuntimeError`, or `Cancelled` |
| type | `TaskFailure` | child identity, failure kind, message, diagnostic code, and source position |
| function | `CreateTaskGroup(): TaskGroup` | creates a group owned by the calling task |
| function | `StartTaskInGroup<T>(Group: TaskGroup; Work: function(Token: CancellationToken): T): task of T` | starts a retained function child; a procedure worker returns task |
| function | `StartSupervisedTask<T>(Group: TaskGroup; Work: function(Token: CancellationToken): T; RetryLimit: integer; BackoffMillis: integer): task of T` | starts a function task with bounded retries; a procedure worker returns task |
| function | `GetTaskGroupToken(Group: TaskGroup): CancellationToken` | returns the group-owned cancellation token |
| function | `CancelTaskGroup(Group: TaskGroup): boolean` | requests cancellation without joining; true only for the first request |
| function | `CloseTaskGroup(Group: TaskGroup): array of TaskFailure` | cancels, joins registered children, releases their results, and returns failures |
| function | `CloseTaskGroupWithTimeout(Group: TaskGroup; TimeoutMillis: integer): result of (array of TaskFailure, TaskError)` | attempts cooperative close with a waiting budget; timeout retains group ownership |
| function | `TryCloseCompletedTaskGroup(Group: TaskGroup): option of array of TaskFailure` | closes without cancellation only when every child is already terminal |
| function | `ReceiveCase(Queue: channel of T; Callback: procedure(Outcome: result of (T, TaskError))): WaitCase` | describes one receive and its typed delivery callback |
| function | `SendCase(Queue: channel of T; Value: T; Callback: procedure(Outcome: result of (boolean, TaskError))): WaitCase` | describes one send without enqueueing its value |
| function | `TaskCase<T>(Handle: task of T; Callback: procedure()): WaitCase` | observes function-task completion; also accepts a procedure-task handle |
| function | `TimerCase(Milliseconds: integer; Callback: procedure()): WaitCase` | describes a timer relative to `Select` entry |
| function | `CancellationCase(Token: CancellationToken; Callback: procedure()): WaitCase` | describes observation of cancellation |
| function | `Select(Cases: array of WaitCase): integer` | commits one operation, runs its callback, then returns its input index |
| function | `CloseWaitCase(Handle: WaitCase): boolean` | discards an unused case; false when already closed or selected |
| function | `CreateCancellationSource(): CancellationSource` | creates an active source |
| function | `GetCancellationToken(Source: CancellationSource): CancellationToken` | returns a token linked to the source |
| function | `Cancel(Source: CancellationSource): boolean` | requests cancellation; true only for the first request |
| function | `IsCancellationRequested(Token: CancellationToken): boolean` | reads the shared cancellation state |
| function | `CreateChannel(Capacity: integer): channel of T` | creates a VM-owned bounded channel; capacity is `1..=1048576` |
| function | `Send(Queue: channel of T; Value: T): result of (boolean, TaskError)` | blocks while full; returns an error after close |
| function | `TrySend(Queue: channel of T; Value: T): result of (boolean, TaskError)` | sends immediately; `Ok(false)` means the open channel is full |
| function | `SendWithCancellation(Queue: channel of T; Value: T; Token: CancellationToken): result of (boolean, TaskError)` | send that also observes cancellation |
| function | `SendWithTimeout(Queue: channel of T; Value: T; TimeoutMillis: integer): result of (boolean, TaskError)` | sends before a relative monotonic deadline |
| function | `Receive(Queue: channel of T): result of (T, TaskError)` | blocks while empty and open |
| function | `TryReceive(Queue: channel of T): result of (option of T, TaskError)` | receives immediately; `Ok(None)` means the open channel is empty |
| function | `ReceiveWithCancellation(Queue: channel of T; Token: CancellationToken): result of (T, TaskError)` | receive that also observes cancellation |
| function | `ReceiveWithTimeout(Queue: channel of T; TimeoutMillis: integer): result of (T, TaskError)` | receives before a relative monotonic deadline |
| function | `CloseChannel(Queue: channel of T): boolean` | closes and wakes waiters; true only for the first close |
| function | `Wait<T>(Handle: task of T): T` | consumes a function-task result once; on task it is a procedure call without a value |
| procedure | `WaitAll<T>(Tasks: array of task of T)` | observes all completions; also accepts array of task; does not consume results |
| function | `WaitAny<T>(Tasks: array of task of T): integer` | observes the lowest completed index; also accepts array of task |
| function | `WaitAnyWithTimeout<T>(Tasks: array of task of T; TimeoutMillis: integer): result of (integer, TaskError)` | completion index or timeout; also accepts array of task |
| function | `WaitAnyWithCancellation<T>(Tasks: array of task of T; Token: CancellationToken): result of (integer, TaskError)` | completion index or cancellation; also accepts array of task |

---

## Cooperative cancellation

`CreateCancellationSource` creates one active state. `GetCancellationToken` returns an opaque token
that may be copied and passed to worker tasks. `Cancel` atomically changes the shared state and
returns `true`; later calls return `false`. `IsCancellationRequested` is non-blocking.

Cancellation is a request, not forced task termination. A task or hosted blocking operation stops
only when it checks the token. The cancellation-aware `Std.Net.Sockets` connect, accept, read, and write
operations observe tokens while their interruptible network phases are pending.

```pascal id=example-2 fragment context=body-console-tasks
const Source: Tasks.CancellationSource := Tasks.CreateCancellationSource();
const Token: Tasks.CancellationToken := Tasks.GetCancellationToken(Source);
discard Tasks.Cancel(Source);
if Tasks.IsCancellationRequested(Token) then
  Console.WriteLn('stopping');
end if;
```

Sources and tokens belong to the VM that created them. Ordinary source storage is released when
that VM ends; group-owned token storage is released when the group is closed.

---

## Task groups

For ownership bound to one block, use a [task scope](../../language/concurrency/task-scopes.md)
instead: it cancels without tokens and re-raises child failures when it ends. Task groups cover
dynamic cases such as passing ownership between routines or supervised retries.

Run the [task-group worker example](../../../../examples/pascal/concurrency/task_group_workers.fpas)
to see an ordinary Result error retrieved with `Wait` and handled before group close,
followed by cooperative worker stop. Returned Error values do not produce group failure records.

### CreateTaskGroup and StartTaskInGroup

`CreateTaskGroup` creates an empty group owned by the calling task. `StartTaskInGroup` registers
one child before publishing its task handle. The worker receives the group's `CancellationToken`
as its only by-value argument and may be a procedure or a function. A `var`
parameter is rejected. The worker's callable type must be fully instantiated;
bind a generic routine to an explicit `function(Token: CancellationToken): T`
or `procedure(Token: CancellationToken)` type before starting it. Captures must be immutable. The task
retains the routine's result type, so `Wait` retrieves its value using the existing task rules.
Function workers return `task of T`; procedure workers return `task`. Their
declarations, stored fields, and containers use those same fixed handle types.
Result success and error payloads follow the ordinary type/transfer rules;
the error payload is not restricted to string.

The creator and registered children may start children in an open group. Unrelated tasks may not.
Ordinary go requires a same-routine lexical scope and does not register with a group.
Bind a returned child handle when its result is needed, or explicitly discard it.
The [discard keyword](../../language/functions/discard.md#task-handles-and-aggregates)
accepts handles without cancelling, detaching, joining, or consuming results.
The group retains ownership until close. Workers cannot access foreign mutable
variables, including globals through helpers/callbacks; pass values or channels.
Known access is FP3042; dynamic targets retain mandatory runtime checks.
Limits are 4096 live groups per VM and 1024
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
Successful recovery leaves no group failure. Exhaustion returns a final Result Error as ordinary
data without a group failure record. An exhausted panic preserves its original diagnostic and
produces one `Panicked` failure, rather than a failure per attempt. With a zero retry limit, the
first outcome is final. Message text such as `'cancelled'` is never used to classify errors.

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
- `Error(TaskError.Timeout(Operation))` means the waiting budget expired while the group was
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

Like every main-task wait, it waits for notifications without executing queued workers itself. Child tasks save
the deadline and yield the pool thread; the debugger uses the same suspension with its clock.
Timer granularity, scheduling delays, and non-cooperative code occupying available execution
threads can delay observation. This is a cooperative waiting budget, not a real-time guarantee,
forced worker termination, or a bound on VM teardown. Keep the VM alive and arrange a later join
after timeout; returning from the main task does not turn the timeout into completed cleanup.

See the [timeout regression](../../../../tests/concurrency/task_group_close_timeout_test.fpas)
for cancellation, retained ownership, and retrying close after releasing a blocked worker.

[`Std.Net.Server`](../network/server.md) supplies server ownership, one common shutdown budget,
and separately host-authorized process escalation. Timed task-group close itself never kills
the process.

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
| `Code` | `integer` | runtime diagnostic code |
| `Line` | `integer` | runtime source line, or zero when unavailable |
| `Column` | `integer` | runtime source column, or zero when unavailable |

`Panicked` means the worker executed panic; `RuntimeError` covers other runtime
diagnostics; `Cancelled` identifies runtime cancellation. Returned Error values
are ordinary Result data and produce no group failure records. This also applies
to a supervised worker's final Error after its explicit retry policy is exhausted.

Ordinary error results do not automatically cancel siblings. Group-owned runtime failures are
collected without aborting unrelated work. Explicit Wait on a failed child still
propagates its original diagnostic. Retrieve ordinary Result data before close;
explicitly handle its branches with case/try when required. On creating-task
termination, any open groups are cancelled and fully closed before its value or
failure is published. Unexpected cleanup failures retain their original diagnostics.

Normal and deterministic debugger execution share these rules. A group-owned debugger failure
emits a task exit event without stopping the parent. The exited child cannot be resumed through
debugger failure recovery after its terminal report has been published.

---

## Bounded channels

`CreateChannel` creates a FIFO queue with fixed capacity. The declaration supplies the element type
because capacity alone cannot infer `T`:

```pascal id=example-3 fragment context=body-tasks
const Messages: channel of string := Tasks.CreateChannel(16);
```

`Send` waits until space is available. `Receive` waits until a value is available. Successful sends
return `Ok(true)`; successful receives return `Ok(Value)`. Values are received in send order.

```pascal id=example-4 fragment context=channel-messages
case Tasks.Send(Messages, 'ready') of
  when Ok(_):
    begin
    end;
  when Error(const Message):
    panic(Message.Message());
end case;

case Tasks.Receive(Messages) of
  when Ok(const Message):
    Console.WriteLn(Message);
  when Error(const Message):
    panic(Message.Message());
end case;
```

`CloseChannel` is idempotent: the first close returns `true`, and later closes return `false`.
Buffered values remain receivable after close. Once drained, `Receive` returns
`Error(TaskError.Closed(Operation))`; `Send` returns that error immediately. Closing or VM shutdown wakes
blocked senders and receivers.

The cancellable variants additionally observe a `CancellationToken`. They return
`Error(TaskError.Cancelled(Operation))`. Cancellation
does not close the channel. If cancellation is already requested when an operation starts, the
cancellation result takes precedence over ready transfer on every probe. Invalid native handles and wrong runtime types remain diagnostics; cancellation never transfers a value.

`TrySend` and `TryReceive` never wait. `TrySend` returns `Ok(false)` when the channel is open but
full. `TryReceive` returns `Ok(None)` when it is open but empty and `Ok(Some(Value))` after receiving
a value. A closed channel still returns `Error(TaskError.Closed(Operation))`, so closure is distinct from a
temporary full or empty state.

`SendWithTimeout` and `ReceiveWithTimeout` take a relative, non-negative millisecond duration. The
runtime converts it to a monotonic deadline once, so wakeups and scheduler work do not restart the
timeout. A zero timeout performs one immediate attempt. An available slot or buffered value wins
every probe, including after suspension with an expired deadline; otherwise the operation returns `Error(TaskError.Timeout(Operation))`. A timeout does not close or otherwise change the channel.

Channel handles belong to their creating VM. A channel accepts only its declared element type, and
task-bound values with mutable captures cannot cross the channel boundary. See
[Channel types](../../language/types/channels.md).

---

## Waiting for one task

Blocks until the spawned call completes, then returns its value. The task result is **consumed**: calling `Wait` again on the same logical completion is a runtime error.

For a `task of T` handle this is a function returning T. For a `task` handle
it is a procedure operation: `Wait(Handle);` completes without a value.
Using that procedure form as an expression is rejected, including inside an
aggregate or another call's arguments.

```pascal id=example-5 fragment context=task-square
scope
  const T: task of integer := go Square(6);
  Console.WriteLn(Tasks.Wait(T));
end scope;
```

**Hint:** If you need the result only once, assign `Wait(T)` to a variable and reuse that value.

---

## Waiting for all tasks

Blocks until every task in the array has finished. This is a **barrier only**; it does not pop return values. Typical use: synchronize before reading results with `Wait`, or when you only need to know that all work finished.

The array is homogeneous: `array of task` for procedures or `array of task of T`
for functions with one result type. Differently typed function/procedure handles
cannot share a task array. Use a scope to join differently typed children, or
wait on separately typed handles outside a shared scope.

```pascal id=example-6 fragment context=task-procedures
scope
  const Ta: task := go Work(1); // Work is a procedure.
  const Tb: task := go Work(2);
  Tasks.WaitAll([Ta, Tb]);
  // still valid:
  Tasks.Wait(Ta);
  Tasks.Wait(Tb);
end scope;
```

An empty array completes immediately.

---

## Waiting for any task

Waits for at least one successful task completion and returns its zero-based input position.
The array must contain between 1 and 1,048,576 retained task handles. It follows the same
task-array typing rules as `WaitAll`; duplicate handles are allowed.
For mixed-result completion selection, construct TaskCase values and use
Select. This observes completion without converting the original typed
handles to an erased task type.

The runtime validates every identity before selecting a result. Invalid identities take precedence
over task failures; visible failures take precedence over successful completion. Among failures,
the first in input order is propagated with its original diagnostic. Among successful completions,
the lowest input position wins. This ordering describes one synchronized observation, not the
physical order in which workers finished, and does not promise fairness.

`WaitAny` does not consume results or cancel losing tasks. A successful result already consumed
by `Wait` still counts as complete, as with `WaitAll`; waiting for its value again remains an error.
Existing runtime-wide worker-failure handling remains active.

```pascal id=example-7 fragment context=task-wait-any
const First: integer := Tasks.WaitAny([Ta, Tb]);
// Both results still belong to their task handles.
Tasks.WaitAll([Ta, Tb]);
Tasks.Wait(Ta);
Tasks.Wait(Tb);
```

The main task waits without executing queued tasks; see [Waiting and execution](#waiting-and-execution).
Child tasks suspend as described above. There is no per-input helper thread and no busy waiting.
VM shutdown releases pending waits through the existing task-failure path.

## Controlled task-completion waits

`WaitAnyWithTimeout` and `WaitAnyWithCancellation` use the same non-empty bounded task list,
input ordering, and non-consuming completion policy as `WaitAny`. Success returns `Ok(Index)`.
Timeout returns `Error(TaskError.Timeout(Operation))`; cancellation returns
`Error(TaskError.Cancelled(Operation))`. Neither outcome cancels tasks or consumes their results.

- Timeout milliseconds must be non-negative. One monotonic budget starts after argument validation;
  wakeups and scheduler probes do not reset it. Zero performs one immediate completion observation.
- On every observation, including resumed probes, readiness wins over an expired
  budget, even if completion became available while the caller
  was not observing. Completions are not timestamped.
- A cancelled token wins over successful completion, including on entry. Invalid task identities
  and task failures are checked first; they remain runtime diagnostics, not ordinary Result errors.
  Invalid tokens and timeouts also produce runtime diagnostics.
- Main-task waits request parking intervals of at most 10 ms, shortened to the remaining timeout;
  child waits use the shared suspended-operation path described above.
  This is cooperative, not a hard wall-clock bound: timer granularity and scheduling delays can
  postpone the next observation.
- Debugger waits use its monotonic clock and explicit suspension. No per-input worker or persistent
  wait registration is created. Completion, timeout, cancellation, failure, and teardown release the
  suspended wait's task list without changing ownership of the tasks themselves.

## Mixed-source selection

Interactive process reads, partial writes, stdin close and native exit status can also
participate through [`Std.Os.Proc` case constructors](../host/proc.md#selectable-interactive-pipes).
They share the single-use and owner-task rules below. Discarding such a case leaves
its independent process handle open.

Construct cases for the events an operation can handle, then pass them to `Select`. Case construction
does not send, receive, start tasks, or start timers. Channel constructors check the callback against
that channel's element type, so one selection can safely combine differently typed channels.

- `ReceiveCase` delivers `Ok(Value)` or `Error(TaskError.Closed(Operation))` after the closed buffer drains.
- `SendCase` delivers `Ok(true)` after enqueueing its value, or `Error(TaskError.Closed(Operation))`.
  The value must match the channel type and cannot contain a task-bound callable, as with `Send`.
- `TaskCase` observes completion without consuming its value. The callback or caller can still use
  `Wait` under its existing single-consumption rule. Consumed successful tasks still count as complete.
  It accepts both `task` and `task of T`; the callback has no payload. Cases for
  different task result types may share an array of WaitCase. Keep the original
  typed handles for Wait, and do not use a case to bypass scope escape rules.
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
Unused cases close when their creating task terminates, after structured child/group
cleanup and before publishing its outcome. An abandoned supervised attempt closes
its unused cases before retry; stale copies never refer to the new attempt's cases.
Claimed cases remain owned by their pending selection. Cleanup does not run callbacks
or close or cancel underlying resources. A reusable Rust callback session retains
its cases across invocation returns and closes them at session shutdown or destruction.
Closing an automatically closed case returns false, as with an explicitly closed case.

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
- **Unknown task handle:** Wait operations accept owned handles created in the current VM by a scoped go expression, StartTaskInGroup, or StartSupervisedTask. Forged or foreign identities produce an invalid-task diagnostic. Owner close releases child results; retrieve required data beforehand.
- **Invalid wait-any size:** `WaitAny` requires between 1 and 1,048,576 task handles.
- **Task failure:** `Wait` and `WaitAll` propagate the spawned task's original diagnostic, including its code and source location. The runtime also enters its **failure** path so other spawned work can stop cooperatively. Fix the reported fault in the spawned task.
- **Creating-task teardown:** remaining groups are cancelled and fully joined before publishing the task's outcome, including on the main task. Cooperative children must observe their cancellation tokens. See [Scheduling](../../language/concurrency/scheduling.md).
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
| Bytecode | [`instruction.rs`](../../../../crates/fpas-code/src/instruction.rs), [`intrinsic/task.rs`](../../../../crates/fpas-code/src/intrinsic/task.rs) |
| VM | [`tasks/mod.rs`](../../../../crates/fpas-runtime/src/execution/vm/tasks/mod.rs), [`tasks/scheduler.rs`](../../../../crates/fpas-runtime/src/execution/vm/tasks/scheduler.rs), [`tasks/state.rs`](../../../../crates/fpas-runtime/src/execution/vm/tasks/state.rs), [`shared/task_results.rs`](../../../../crates/fpas-runtime/src/execution/vm/shared/task_results.rs) |
| Cancellation | [`cancellation/registry.rs`](../../../../crates/fpas-runtime/src/execution/vm/cancellation/registry.rs), [`tasks/cancellation.rs`](../../../../crates/fpas-runtime/src/execution/vm/tasks/cancellation.rs) |
| Channels | [`channels/registry.rs`](../../../../crates/fpas-runtime/src/execution/vm/channels/registry.rs), [`tasks/channel.rs`](../../../../crates/fpas-runtime/src/execution/vm/tasks/channel.rs) |
| Completion selection | [`tasks/wait_any.rs`](../../../../crates/fpas-runtime/src/execution/vm/tasks/wait_any.rs), [`scheduler/result_polling.rs`](../../../../crates/fpas-runtime/src/execution/vm/tasks/scheduler/result_polling.rs) |
| Mixed selection | [`tasks/selection/`](../../../../crates/fpas-runtime/src/execution/vm/tasks/selection/mod.rs), [`shared/wakeups.rs`](../../../../crates/fpas-runtime/src/execution/vm/shared/wakeups.rs), [`callbacks/`](../../../../crates/fpas-runtime/src/execution/vm/hosted/callbacks/mod.rs) |
| Task groups | [`tasks/groups/`](../../../../crates/fpas-runtime/src/execution/vm/tasks/groups/mod.rs), [`scheduler/group_results.rs`](../../../../crates/fpas-runtime/src/execution/vm/tasks/scheduler/group_results.rs), [`driver/failure.rs`](../../../../crates/fpas-runtime/src/execution/vm/debug/tasks/driver/failure.rs) |
| Supervision | [`tasks/supervision/`](../../../../crates/fpas-runtime/src/execution/vm/tasks/supervision/mod.rs), [`tasks/pool.rs`](../../../../crates/fpas-runtime/src/execution/vm/tasks/pool.rs) |

## See also

- [Concurrency index](README.md)
- [Standard library index](../README.md)
