# Future: Server Lifecycle

> Deferred. This plan describes hosted process behavior, not an application protocol.

Long-running servers need one coordinated lifetime instead of unrelated loops and stop flags. The
lifecycle module should turn host shutdown requests into cancellation, stop admission of new work,
drain owned work to a deadline, and report the final outcome.

## Proposed scope

- Portable observation of interrupt and termination requests supported by the host platform.
- A server lifetime object that owns a cancellation source, listeners, task groups, and shutdown
  deadline.
- Ordered shutdown phases: stop accepting, notify sessions, finish or cancel in-flight work, flush
  durable state, close resources, and return status.
- Per-resource and whole-process deadlines with failures collected instead of discarded.
- Idempotent shutdown callable from a signal, administrative request, or fatal dependency failure.
- Readiness state that becomes false before listeners stop accepting new work.

## Interface rules

- Signal callbacks perform no application work directly; they request cancellation through the
  lifetime interface.
- Every admitted session is owned by exactly one task group.
- Detached tasks are not allowed to outlive the server lifetime accidentally.
- A second shutdown request may shorten the deadline but must not start a second cleanup sequence.
- Forced termination after the deadline is explicit and reports which resources did not close.

## Acceptance requirements

- Interrupt, administrative stop, listener failure, and normal completion share one cleanup path.
- New work is rejected after shutdown begins while admitted work receives its documented grace
  period.
- Database transactions, network sessions, queues, logs, and metrics close in a tested order.
- Repeated stop requests and partial cleanup failures remain deterministic.
- Platform-specific signal differences are normalized or documented without changing application
  logic.
