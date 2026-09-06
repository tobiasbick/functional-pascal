# Future: Application Concurrency Primitives

> Partially implemented. Current `go`, `Wait`, and `WaitAll` behavior remains unchanged.

Fork-join tasks are sufficient when all work starts together and the caller waits for completion.
Long-running applications also need bounded communication, cooperative cancellation, multi-source
waiting, and explicit ownership of child-task failure.

## Progress

### 2026-09-06 — typed bounded channels

- Added the approved built-in `channel of T` language type across parsing, formatting, semantic
  analysis, compiled-unit interfaces, typed IR, portable debugger metadata, and language-service
  source spans.
- Added VM-owned FIFO channels with a fixed capacity of `1..=1048576`, blocking send and receive,
  idempotent close, buffered drain after close, and wakeup during close or VM shutdown.
- Added `CreateChannel`, `Send`, `Receive`, cancellable send/receive variants, and `CloseChannel` to
  `Std.Task`. Closed and cancelled operations return distinct documented errors.
- Added static rejection for mismatched element types and task-bound values, plus parser, formatter,
  semantic, registry, compiler/runtime, and FPAS suite regressions.
- Verified with `cargo fmt --check`, `cargo build --workspace`, `cargo test --workspace`, strict
  Clippy for every affected crate, and the full FPAS suite (406 passed, 1 skipped).
- Performance benchmarks are intentionally omitted because another VM is active on the host; this
  slice makes no performance claim.
- Remaining: non-blocking and deadline-bounded channel operations, broader cancellation coverage,
  multi-wait, task groups, and supervision.

### 2026-09-06 — cooperative cancellation foundation

- Implemented VM-owned `CancellationSource` and clonable `CancellationToken` handles in `Std.Task`.
- Implemented idempotent `Cancel` and non-blocking `IsCancellationRequested`.
- Added `Std.Net.AcceptWithCancellation` as the first end-to-end hosted consumer. Cancellation
  returns a distinct documented error and does not close the listener.
- Added registry, blocked-accept, and FPAS end-to-end regressions.
- Verified with `cargo build`, the full Rust workspace test suite, strict Clippy for the affected
  crates, and the FPAS suite (405 passed, 1 skipped). Whole-workspace strict Clippy remains blocked
  by pre-existing `unwrap` calls in an unrelated `fpas-build` integration test.
- Remaining: deadline-aware waits, integration with other blocking hosted operations, bounded
  channels, multi-wait, task groups, and supervision.
- Performance benchmarks were intentionally not used for this correctness slice because another VM
  was active on the host; no performance claim is recorded.

## Proposed scope

- Non-blocking and deadline-bounded send and receive operations.
- A cancellation source and clonable cancellation token checked by hosted blocking operations.
- `WaitAny` or an equivalent function-based multi-wait over tasks, channels, timers, and
  cancellation without adding new syntax.
- Task groups that own child tasks, propagate cancellation, collect failures, and join during close.
- A supervisor policy for restartable long-running workers with explicit retry limits and backoff.

## Interface rules

- Bounded channels are implemented as the default; unbounded queues require a separate explicit
  decision.
- Closing is implemented as idempotent, blocked senders and receivers wake, and buffered values
  remain owned by the channel until received or VM teardown.
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

- FIFO behavior, closure, full/empty queues, and cancellation have deterministic tests; deadline
  tests remain gated on deadline-aware operations.
- Multi-wait returns exactly one winning event and unregisters all losing waits.
- Task-group shutdown cannot leak workers or wait forever after its deadline.
- Child panics and ordinary error results follow separately documented paths.
- High-contention tests demonstrate bounded memory and absence of lost wakeups.
