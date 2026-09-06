# Future: Multi-source Waiting

> Partially implemented: task-only `WaitAny` and its timeout/cancellation variants are implemented.
> Mixed-source selection remains future work.

Part of [application concurrency](concurrency.md). Build this in complete, independently tested
slices. Keep `go`, task result typing, `Wait`, `WaitAll`, and channel ownership unchanged.

## First slice: task completion

`Std.Task.WaitAny(Tasks: array of task): integer` is implemented as a completion barrier. It returns the
zero-based position of one completed task, not the task's result. The caller still uses `Wait`
to consume that result. Accept the same task-array types as `WaitAll`; do not introduce
heterogeneous task-array conversion or erase result types.

Implemented contract (current reference: [Std.Task](../../pascal/std/concurrency/task.md)):

- Require a non-empty array, bounded to 1,048,576 entries. Reject invalid sizes before copying IDs.
- Validate every handle before selecting a winner. Unknown or detached identities are runtime
  errors, even if another entry is ready. Duplicate handles are allowed.
- Observe the entire array under the scheduler's retained-result lock. If several successful
  completions are visible, return the lowest input position. This is deterministic priority, not
  a claim of fairness or an ordering of physical completion times.
- Consumed successful results still count as completed, matching `WaitAll`. A later `Wait` on
  such a handle remains an error; `WaitAny` does not recreate a consumed result.
- Propagate a visible task failure with its original diagnostic rather than returning an index.
  Match `WaitAll` precedence: invalid identity, then failure, then successful completion, then
  pending. Among failures, use input order. Do not turn task failure into an ordinary error string.
- Waiting neither cancels nor detaches losing tasks. Their existing ownership and lifetime remain.
- A successful observation wins over a completion or failure published after that observation.
  Existing runtime-wide worker-failure handling remains active.
- Do not start one helper task or OS thread per input. Reuse scheduler helping and condition-variable
  waiting. The predicate and transition to sleep must be synchronized to prevent lost wakeups.
- Debugger execution suspends explicitly and polls the same selection policy without blocking its
  execution lane. Shutdown releases the wait through the existing task-failure path.

This first slice does not satisfy the mixed-source acceptance requirement by itself.

## Second slice: deadlines and cancellation

`WaitAnyWithTimeout(Tasks, TimeoutMillis)` and `WaitAnyWithCancellation(Tasks, Token)` are
implemented with `Result of integer, string` outcomes. Task failure still retains its diagnostic.
The current [Task reference](../../pascal/std/concurrency/task.md) owns their exact contract,
including the cooperative scheduler-helping limitation.

- Distinguish no completed task yet, cancellation, expiry, and task failure.
- Use one monotonic deadline; scheduler helping and spurious wakeups do not extend it.
- A zero timeout performs one immediate observation. Pre-cancellation takes precedence over a
  ready task; otherwise a ready task wins the immediate attempt before timeout is reported.
- Use the debugger clock for deterministic deadline tests.
- Remove every wait registration on completion, cancellation, expiry, failure, and shutdown.

## Mixed sources: settle value ownership before adding an API

Tasks have non-consuming completion state. Channels instead perform ownership-changing send and
receive operations. A channel-ready notification does not reserve a value: another receiver may
consume it before the notified task runs. Returning a ready index and then calling blocking
`Receive` therefore cannot promise an atomic selection.

The mixed-source operation must commit exactly one winning operation. Do not implement it by
running ordinary blocking receives in helper tasks and cancelling the losers: a loser may already
have consumed a value. Likewise, do not erase channel element types into an untyped payload.

Before implementation, specify:

1. How task completion, typed channel operations, timers, and cancellation are represented without
   new syntax or loss of static type checking. If the design requires language changes, obtain
   explicit approval before implementing them.
2. How one shared selection state arbitrates a winner while holding the relevant source lock.
   Only a committed winning send/receive may move a value across the channel boundary.
3. How losing registrations release their references and any unsent values. A wait owns its
   registrations; closing or dropping it must not leave callbacks retained by sources.
4. The global lock order, including registration, source readiness, winner selection, and cleanup.
   Never invoke scheduler work while holding a channel or selection lock.
5. Closure, simultaneous readiness, fairness, and error precedence. A closed channel is a selectable
   terminal outcome, not a source that remains pending forever.
6. Bounds on cases per wait and outstanding registrations. Repeated waits must not grow retained
   source state after their calls finish.

Keep this separate from `WaitAny`'s task-only array API. Do not pre-allocate a generic public
`WaitCase` or `WaitSet` handle until its typing and ownership contract is resolved.

## Implementation layout for the first slice

Paths below are relative to the repository root. Create only modules that have a production caller.

| Action | Path | Concern |
|--------|------|---------|
| Split | `crates/fpas-vm/src/vm/tasks/scheduler.rs` | Move retained-result polling into the focused module below before growing this roughly 400-line scheduler |
| Create | `crates/fpas-vm/src/vm/tasks/scheduler/result_polling.rs` | Existing single/batch polling and new atomic any-completion observation |
| Modify | `crates/fpas-vm/src/vm/shared/task_results.rs` | Explicit pending, winning-index, failed, and unknown outcomes |
| Create | `crates/fpas-vm/src/vm/tasks/wait_any.rs` | Runtime argument checks, blocking/helping loop, and debugger polling |
| Modify | `crates/fpas-vm/src/vm/tasks/mod.rs`, `crates/fpas-vm/src/vm/tasks/suspension.rs` | Dispatch and explicit suspended wait state; keep selection logic in the new module |
| Modify | `crates/fpas-bytecode/src/intrinsic/task.rs`, `crates/fpas-bytecode/src/intrinsic/execution.rs` | Stable intrinsic identity and execution classification |
| Modify | `crates/fpas-std/src/std_units/symbols/std_symbols/task.rs`, `crates/fpas-sema/src/std_registry/loaded/channel_task.rs`, `crates/fpas-sema/src/std_registry/builtins/channel_task.rs` | Symbol, declaration, and task-array checking |
| Modify | `crates/fpas-compiler/src/intrinsic_catalog.rs` | Intrinsic mapping |
| Modify | `docs/pascal/std/concurrency/task.md`, `lib/api/Std/Task.fpas` | Implemented contract and regenerated editor declaration, only after implementation |

Inspect exhaustive intrinsic matches and debugger suspension consumers before changing the enum;
the table identifies ownership, not permission to ignore additional required call sites. If a
proposed addition changes the language contract rather than only the Std API, stop at that boundary.

## Acceptance tests and exit gates

### Task-only barrier

- Pending tasks followed by one completion; first and last array positions; duplicate handles.
- Multiple ready tasks select the lowest input position, independent of task identity order.
- Successful results remain consumable exactly once; already consumed results remain complete.
- Invalid handles anywhere in the array, empty input, and the size bound.
- Failure precedence and preservation of diagnostic code, message, and source location.
- Completion between predicate inspection and sleeping; shutdown while waiting; repeated wakeups.
- A waiting worker can help queued work, including a single-worker configuration.
- Normal and deterministic debugger execution produce the same result and failure policy.
- Static rejection of non-task arrays and end-to-end FPAS tests for result indexing and later `Wait`.

### Mixed-source operation

- Exactly one committed send/receive under simultaneous readiness and competing waiters.
- Losing receives leave values untouched; losing sends retain their unsent values.
- Closure, cancellation, expiry, task failure, and shutdown unregister every case.
- Registration races cannot lose wakeups; repeated high-contention waits leave bounded storage.

Each implementation slice requires formatting, workspace build/tests, relevant FPAS regressions,
and current documentation. Use performance benchmarks only for a performance claim; correctness
tests must not infer fairness or a hard scheduling-latency bound from wall-clock measurements.
