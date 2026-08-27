# Future: Application Concurrency Primitives

> Deferred. Current `go`, `Wait`, and `WaitAll` behavior remains unchanged.

Fork-join tasks are sufficient when all work starts together and the caller waits for completion.
Long-running applications also need bounded communication, cooperative cancellation, multi-source
waiting, and explicit ownership of child-task failure.

## Proposed scope

- Typed bounded channels with send, receive, close, and documented end-of-stream behavior.
- Non-blocking and deadline-bounded send and receive operations.
- A cancellation source and clonable cancellation token checked by hosted blocking operations.
- `WaitAny` or an equivalent function-based multi-wait over tasks, channels, timers, and
  cancellation without adding new syntax.
- Task groups that own child tasks, propagate cancellation, collect failures, and join during close.
- A supervisor policy for restartable long-running workers with explicit retry limits and backoff.

## Interface rules

- Bounded channels are the default; unbounded queues require a separate explicit decision.
- Closing is idempotent, blocked senders and receivers wake, and ownership of buffered values is
  defined.
- Cancellation is cooperative and distinguishable from task failure.
- Dropping a retained task handle must not silently detach work whose lifetime is still owned by a
  task group.
- Hosted resources integrate through cancellation tokens instead of each module inventing a stop
  flag.

## Runtime seam

The scheduler should hide worker queues, timers, wakeups, and condition variables behind the task
and channel interface. TUI, network, database, and process modules should consume that interface
rather than reaching into scheduler implementation details.

## Acceptance requirements

- FIFO behavior, closure, full/empty queues, deadlines, and cancellation have deterministic tests.
- Multi-wait returns exactly one winning event and unregisters all losing waits.
- Task-group shutdown cannot leak workers or wait forever after its deadline.
- Child panics and ordinary error results follow separately documented paths.
- High-contention tests demonstrate bounded memory and absence of lost wakeups.
