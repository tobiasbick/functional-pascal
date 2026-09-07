# Implementation notes: Typed Mixed-source Selection

> Implemented and verified. C1 is complete in the [concurrency worklist](concurrency.md).

The current [Task reference](../../pascal/std/concurrency/task.md) owns the public behavior.
These notes retain the design rationale and acceptance requirements. Existing task result types,
channel element types, closure ownership, and language syntax remain unchanged.

## Interface and typed value delivery

Use an opaque `Std.Task.WaitCase` resource and function-based constructors. A case records one
operation and its completion callback; constructing it does not send, receive, or start a task.

| Operation | Constructor inputs | Completion callback |
|-----------|-----------------------------|---------------------|
| Channel receive | `ReceiveCase`: channel of T, callback | procedure accepting result of T, string |
| Channel send | `SendCase`: channel of T, value of T, callback | procedure accepting result of boolean, string |
| Task completion | `TaskCase`: retained task, callback | procedure with no arguments |
| Relative timer | `TimerCase`: non-negative milliseconds, callback | procedure with no arguments |
| Cancellation | `CancellationCase`: cancellation token, callback | procedure with no arguments |

`Select` accepts an array of cases and returns the winning zero-based index after its callback
returns. Different channel element types can appear in one selection because each constructor
checks its own callback parameter type. There is no public untyped payload or conversion of
heterogeneous task results. A task case observes completion only: its callback or caller may still
use `Wait` to consume the result under the existing rules.

Callbacks run on the selecting task, exactly once for the winning operation and never for losers.
Use the VM's resumable callback continuation path, including on the main task and in the debugger;
do not run arbitrary callback code synchronously inside the polling loop or any source lock.
Callback failure propagates normally; a committed channel operation cannot be rolled back.

## Case ownership and limits

- Cases are single-use and belong to the creating VM and task. Copies refer to the same logical
  case. Selection from another task is rejected before invoking any callback or transferring values.
- Validate the whole case array before claiming cases. Reject duplicate case identities, already
  claimed/closed cases, wrong owners, and invalid array size without partially claiming the input.
- Claim a valid array atomically in the case registry. Remove the claimed descriptors from that
  registry; the active wait owns all of their values and callbacks until selection ends.
- `CloseWaitCase` explicitly discards an unused case and its captured references. It returns true
  only when it discards a live case. A well-formed absent identity returns false; a live case owned
  by another task is an error. Allocate process-unique, tagged identities with checked exhaustion
  so another VM's case cannot alias a local case. Keep no closed-case tombstone collection.
- Bound the array to 1..=1024 cases and the number of unused descriptors to 4096 per VM. Reject
  exhaustion explicitly. Limits apply before copying or retaining user input.
- Losing sends never enter a channel. Release the wait's copies on every exit; the caller's
  original immutable values remain usable. Losing receives leave channel values untouched.
- After success, failure, cancellation selection, or teardown, release all source registrations
  and losing descriptors before invoking a winning callback. Repeated completed selections must
  not accumulate descriptor entries, callbacks, or source subscriptions.

## Observation and commitment

Validate all sources and inspect task failures before transferring a channel value. Preserve the
original diagnostic for failed task cases. Ordinary channel closure instead completes that case's
callback with its usual channel error result; it must not remain pending forever.

Scan cases in input order. Commit the first ready operation at its source lock; do not first return
a readiness index and then perform a separate blocking operation. A receive removes its value and
a send enqueues its value only as the winning action. Immediately stop scanning after commitment.
There is one selecting owner, not one worker per case. Different simultaneous selections compete
through the existing channel locks, so one channel value cannot satisfy two receivers.

This is ordered observation, not an atomic snapshot of all sources or a fairness guarantee. Timer
and cancellation cases participate in that same input order; callers can give cancellation priority
by placing its case first. Once an operation commits, later cancellation cannot undo it.

## Wakeups and lock order

The implemented `shared/wakeups.rs` signal is latched. Register a new signal with all relevant
sources before probing them. An event before registration is visible in the subsequent probe;
an event after registration is latched even if it precedes parking. Recheck readiness after every
wakeup. Notifications themselves never reserve values, call FPAS code, or choose a winner.

Registry locks are used only to resolve/claim handles and are released before observing sources.
Hold at most one channel or task-result lock at a time. Source operations may then take subscription
and signal locks, in that order. Parking holds only the signal lock. Drop subscriptions without
holding a signal lock; never help scheduler tasks or enter callbacks while holding any source lock.

Task results, queue availability, shutdown, and cancellation now have notification adapters alongside
the channel wake sources. Controlled task-completion waits use a shared task/cancellation signal;
the mixed-source selection dispatcher connects the same adapters to its cases.
Timer deadlines are computed once at `Select` entry, not case construction;
all timers in one selection share its monotonic start. Debugger execution uses its own clock and
explicit suspended state. Scheduler helping remains cooperative; no hard termination bound is
introduced by this operation.

## Verified C1 acceptance gates

- Semantic rejection of wrong callback types, wrong send values, wrong case-array types, and
  task-bound values crossing a send boundary. No new parser or type-conversion rule.
- End-to-end selection across differently typed channels, tasks, timers, and cancellation.
- Exactly one callback and committed operation with multiple ready cases and competing selectors.
- Losing values, closures, case descriptors, and registrations follow the ownership rules above.
- Invalid input anywhere prevents partial selection; task failures retain their diagnostics.
- Closed channels are terminal; timer/cancellation ordering matches normal and debugger execution.
- Register/probe/park races, repeated waits, capacity limits, callback suspension, and teardown.
- Current documentation and editor declarations, examples, formatting, full workspace tests,
  and affected-crate strict Clippy. The final group-shutdown contract remains a separate discussion.
