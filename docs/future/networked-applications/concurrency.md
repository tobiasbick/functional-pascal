# Future: Application Concurrency Primitives

> Partially implemented. Current `go`, `Wait`, and `WaitAll` behavior remains unchanged.

Fork-join tasks are sufficient when all work starts together and the caller waits for completion.
Long-running applications also need bounded communication, cooperative cancellation, multi-source
waiting, and explicit ownership of child-task failure.

## Progress

### 2026-09-06 — task-only WaitAny completion barrier

- Implemented `Std.Task.WaitAny(Tasks): integer`: a bounded, non-consuming completion barrier
  returning the lowest ready input index. Task-array typing, `Wait`, and `WaitAll` are unchanged.
- All identities are validated before failure propagation or success selection. Duplicate handles
  and consumed successful results retain their input positions; losing tasks are not cancelled.
- Extracted retained-result polling into `scheduler/result_polling.rs`. Added a condition-variable
  wait that also returns for queued work, synchronized with enqueue notification to avoid losing
  the transition from predicate inspection to sleep. No per-input worker or registration is created.
- Added explicit debugger suspension with the same selection policy, scheduler race/cleanup
  regressions, semantic checks, and end-to-end compiler/runtime tests including a single worker.
- Updated the current Task reference and regenerated editor declarations. Formatting, workspace
  build, full workspace tests (including bundled FPAS suites), all twelve new regressions, and
  strict Clippy for every affected crate passed. Documentation links and the diff were checked.
  This correctness slice makes no performance claim.
- Next: timeout and cancellation variants; mixed-source channel operations remain deferred until
  typed value transfer and atomic winner ownership are specified.

### 2026-09-06 — multi-wait contract and implementation sequence

- Inspected retained-task polling, blocking channel ownership, and debugger suspension. These use
  distinct state paths; combining blocking receives in helper tasks could consume losing values.
- Added the [multi-wait design](multi-wait.md): implement a non-consuming task-only `WaitAny`
  barrier first, then deadlines/cancellation, then atomic mixed-source operations after their
  typed value-transfer and registration-cleanup contract is settled.
- Specified winner ordering, invalid-handle/failure precedence, consumed results, input bounds,
  debugger behavior, source ownership, file layout, and acceptance tests. Existing task typing,
  `Wait`, and `WaitAll` remain unchanged; no language extension is selected.
- This step changes future documentation only. No new API or runtime behavior is implemented;
  current docs, generated declarations, and tests remain unchanged. Checked links and the diff.
- Next implementation: the task-only completion barrier described in the linked design.

### 2026-09-06 — OS resolver retained, cancellable TCP attempts

- Selected the investigation's recommendation after the user delegated the choice: retain OS
  name resolution and certificate trust. No dedicated resolver or detached blocking worker is added.
- Replaced blocking TCP attempts in the cancellation-aware connect variants with one non-blocking
  socket per address, using `socket2` and Unix-only `libc` error constants. Repository code remains
  unsafe-free. Existing `Connect` and `ConnectTls` are unchanged.
- Pending attempts poll cancellation and the shared monotonic deadline, with at most 10 ms of
  parking between checks. Cancellation closes the owned socket; success restores blocking mode
  before TLS setup or publication. No attempt is restarted merely to check cancellation.
- Added focused tests for pending cancellation, expiry, readiness races, connection errors, and
  restored blocking I/O. The unpublished-socket cleanup test now observes a real accepted socket
  instead of depending on a fixed number of cancellation checkpoints.
- Updated the Net reference and generated editor declarations. Formatting, workspace build,
  full workspace tests (including bundled FPAS regressions), 14 focused establishment tests,
  and strict Clippy for every affected crate passed on Windows. The Unix-specific error-code
  branch and its regression were not executed; only the Windows target was installed.
- Remaining limitation: OS DNS and synchronous TLS configuration/certificate verification cannot
  be interrupted. This change does not claim end-to-end bounded shutdown or a performance gain.
  Next: multi-source waiting, followed by task groups and supervision; resolver replacement is not
  selected.

### 2026-09-06 — resolver/socket backend investigation

- Inspected the current synchronous resolver, TCP connect path, and platform TLS verifier. The
  workspace forbids unsafe Rust; direct platform FFI is not an in-repository implementation option.
- Separated cancellation into three requirements: bounded caller return time, no late handle
  publication, and bounded lifetime/resource use of underlying work. Meeting the first two does
  not establish the third.
- A non-blocking socket backend such as [socket2](https://docs.rs/socket2/latest/socket2/struct.Socket.html)
  can support polling one persistent TCP connection attempt and dropping that socket on cancellation.
  Repeated short `connect_timeout` calls are not equivalent: they restart connection establishment.
- Native asynchronous DNS is not automatically a hard cleanup guarantee. Windows
  [GetAddrInfoExCancel](https://learn.microsoft.com/en-us/windows/win32/api/ws2tcpip/nf-ws2tcpip-getaddrinfoexcancel)
  completes the caller's cancellation notification, but synchronous name-service providers may
  continue consuming resources until they finish. Platform completion ownership must be explicit.
- Wrapping resolution in [Tokio spawn_blocking](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)
  does not cancel a started OS call. A bounded pool can limit resource growth but cannot guarantee
  that occupied workers terminate by a shutdown deadline.
- [Hickory](https://docs.rs/hickory-resolver/latest/hickory_resolver/) implements its own asynchronous
  resolver rather than using the OS resolver. Reading system DNS configuration is not full OS
  name-service equivalence. Any replacement requires an explicit policy decision and validation
  for local names and system-specific routing; dropping lookup futures also needs resource-lifetime
  tests before asserting bounded cleanup.
- Recommendation, not yet selected: preserve OS name-resolution behavior and certificate trust,
  implement independently cancellable TCP socket attempts first, and keep native DNS/verification
  limits explicit. Do not promise end-to-end bounded shutdown from that TCP improvement alone.
- **Decision resolved by the subsequent TCP implementation above:** preserve OS name-service
  behavior instead of selecting a dedicated resolver. Platform TLS verification remains a separate
  cancellation boundary.
- This investigation changes only the plan. No backend dependency, runtime change, build, or test
  run was added in this step; the documentation diff was checked.

### 2026-09-06 — connection-establishment cancellation checkpoints

- Added `ConnectWithCancellation` and `ConnectTlsWithCancellation` with one monotonic budget
  shared across resolution, TCP address attempts, and TLS setup. Existing connect APIs are unchanged.
- Added cancellation checks around DNS and TCP, during TLS handshake I/O, and before handle
  publication. Cancelled or expired attempts release their sockets without retaining a VM handle.
- Kept OS certificate and hostname verification unchanged. No detached resolver/connect workers,
  new dependencies, or language changes were introduced.
- Added VM tests for cancellation boundaries, socket cleanup, successful TCP use, TLS cancellation
  and expiry, invalid arguments, and resolution failure; added semantic and FPAS compiler/runtime
  coverage. Updated the Net reference and generated declarations.
- Verified formatting, the workspace build and full workspace tests (including bundled FPAS
  regressions), nine focused VM tests, semantic/compiler coverage, and strict Clippy for every
  affected crate. No performance claim is made.
- **Still open:** prompt cancellation inside blocking OS DNS/TCP calls and synchronous platform
  certificate verification. The token is observed after those calls return, so the new APIs do not
  promise a hard wall-clock cancellation bound. A resolver/socket backend decision is required
  before claiming end-to-end bounded shutdown; multi-wait, task groups, and supervision also remain.

### 2026-09-06 — cancellable TCP/TLS writes

- Added `Std.Net.WriteWithCancellation(Connection, Data, Token)` with the existing bounded byte
  input and integer progress result. A successful partial write wins over concurrent cancellation;
  callers retry only the unaccepted suffix. TLS acceptance does not imply peer delivery.
- Cancellation and timeout before progress return distinct errors and leave the connection open.
  Previously accepted TLS output remains owned by the connection.
- Extracted shared connection-lock, cancellation, timeout, and non-blocking-mode handling for
  cancellable reads and writes. Failure to restore blocking mode invalidates the connection without
  hiding already reported I/O progress.
- Added nine write regressions covering TCP backpressure, pending attempts, lock contention,
  monotonic timeouts, size limits, partial-write races, and both TLS connection directions. Existing
  cancellable-read tests still pass. Added semantic and compiler/runtime write coverage.
- Updated the `Std.Net` reference and regenerated intrinsic editor declarations. Formatting,
  workspace build, full workspace tests (including bundled FPAS regressions), targeted tests,
  and strict Clippy for every affected crate passed. No performance claim is made.
- Next: cancellation during connection establishment (including DNS and TLS-handshake boundaries),
  then multi-wait, task groups, and supervision.

### 2026-09-06 — cancellable TCP/TLS reads

- Added `Std.Net.ReadWithCancellation(Connection, MaxBytes, Token)` without changing the language.
  Cancellation leaves the connection open and does not discard buffered TLS state.
- Cancellation is checked while acquiring the connection lock and between non-blocking I/O
  attempts. The configured read timeout starts after lock acquisition and uses a monotonic deadline.
- Added tests for pre-cancellation, pending reads, lock contention, timeout preservation, restored
  blocking reads, EOF, and resuming an incomplete TLS record. Added semantic token-type rejection
  and an FPAS compiler/runtime regression using a task-owned read.
- Updated the `Std.Net` reference and generated intrinsic editor declarations.
- Verified formatting, the workspace build and full workspace test suite (including bundled FPAS
  regressions), targeted semantic/VM/compiler tests, and strict Clippy for every affected crate.
- Next: cancellation-aware writes with an explicit partial-write contract, followed by remaining
  blocking-operation coverage, multi-wait, task groups, and supervision. This slice makes no
  performance claim.

### 2026-09-06 — non-blocking and timeout-bounded channel operations

- Added `TrySend` and `TryReceive` with explicit full and empty outcomes that remain distinct from
  channel closure.
- Added `SendWithTimeout` and `ReceiveWithTimeout` with one monotonic deadline per call, precise
  zero-timeout behavior, and distinct send and receive timeout errors.
- Integrated timeout suspension with the deterministic debugger clock and normal task scheduler.
- Added semantic, compiler/runtime, debugger-clock, and FPAS suite regressions.
- Verified formatting, the workspace build and test suite, strict Clippy for every affected crate,
  and the full FPAS suite (407 passed, 1 skipped).
- Captured and compared a full-suite benchmark baseline after the competing VM stopped. The suite
  has no channel-specific workload; unrelated rows varied widely in both directions, while
  `task_spawn_wait` changed from 567 ms to 594 ms (+4.8%). This correctness slice makes no causal
  performance claim and does not add a benchmark-history entry.
- Remaining: broader cancellation coverage, multi-wait, task groups, and supervision.

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
- Remaining: broader cancellation coverage, multi-wait, task groups, and supervision.

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

- FIFO behavior, closure, full/empty queues, deadlines, and cancellation have deterministic tests.
- Multi-wait returns exactly one winning event and unregisters all losing waits.
- Task-group shutdown cannot leak workers or wait forever after its deadline.
- Child panics and ordinary error results follow separately documented paths.
- High-contention tests demonstrate bounded memory and absence of lost wakeups.
