# Future: Application Concurrency Primitives

> C1-C4 implemented and verified. S1 now separates cooperative waiting from process escalation.
> Current API behavior is documented in the [Task reference](../../pascal/std/concurrency/task.md).

Fork-join tasks are sufficient when all work starts together and the caller waits for completion.
Long-running applications also need bounded communication, cooperative cancellation, multi-source
waiting, and explicit ownership of child-task failure.

## Progress

### Implementation status (2026-09-08)

The concurrency primitives are implemented. The approved two-stage shutdown policy separates
timed cooperative group close (S1a) from explicit process-level escalation (S1b), now supplied by
[`Std.Server`](../../pascal/std/network/server.md).
Existing cancellation, bounded channels, and task-only completion waits are also implemented.

- [x] C1: Mixed task/channel/timer/cancellation selection with typed value delivery, exactly one
  committed operation, wait-owned registrations, deterministic debugger support, and race tests.
  Public case constructors, Select, shared wake registrations, and debugger continuation are
  implemented and verified, including callback failure and active-wait teardown coverage.
- [x] C2: Task groups that retain ownership of children, propagate cancellation, collect failures,
  and join on explicit close. Keep ordinary error results distinct from child panics and preserve
  the existing behavior of tasks created outside a group.
- [x] C3: Supervision of restartable workers with bounded retries and backoff, built on task-group
  ownership. Define successful completion, ordinary errors, panic handling, and cancellation paths.
  The selected retry contract is in [supervision](supervision.md).
- [x] C4: End-to-end examples, current documentation, deterministic debugger coverage, and
  high-contention/lifetime regressions for C1-C3, plus complete workspace verification.
- [x] S1a: Add a timed cooperative group close that retains ownership after timeout, with retry,
  validation, race, and debugger tests. The two-stage shutdown policy was approved on 2026-09-07.
- [x] S1b (lifecycle work): `Std.Server` adds explicit process escalation for non-cooperative code
  and blocking host calls, with disposable-process exit confirmation tests. The selected policy is in
  [server lifecycle](server-lifecycle.md#selected-shutdown-policy-2026-09-07).
  A timeout alone does not establish termination; no in-place forced task-group kill is planned.

The dated entries below describe their original implementation stages. The lifecycle and echo
example update on 2026-09-08 supersedes their references to pending S1b work. See the
[lifecycle verification record](server-lifecycle.md#implementation-progress-2026-09-08) for current
coverage and the completed full workspace verification after build-artifact cleanup.

### 2026-09-07 — practical concurrency examples

- Added finite worker-pipeline and timeout/retry-close tutorials with exact-output smoke tests.
  Capacity-two queues demonstrate backpressure without changing existing task benchmarks.
- Migrated Julia to four group-owned workers and a capacity-four row queue. Extracted the CPU
  kernel into `Julia.Compute` and shared the units through `julia-core.fpasprj`. The Console
  event loop draws bounded batches; replacement and exit cancel and join old work. CPU iteration
  checks and cancellable sends cover both computation and full-queue cancellation.
- Added a separate four-worker loopback TCP byte-stream echo server with cancellable accept,
  read, and write, partial-write handling, explicit connection cleanup, and timed close. Its
  fixed lifetime interrupts active clients; it does not implement graceful draining or S1b.
- Added Julia row-equivalence/replacement regressions and a real-client TCP test for simultaneous
  connections, multi-chunk responses, idle-read cancellation, and listener-worker shutdown.
- Workspace build and the complete Rust workspace suite passed (460 CLI tests, 544 VM tests).
  The complete FPAS suite passed with 414 passed, 1 intentionally skipped, and 0 failed. All 61
  example-filtered tests passed. The strengthened TCP first-client handshake regression was
  rerun successfully after the workspace run. Affected-crate strict Clippy, final CLI Clippy,
  Rust/FPAS formatting, documentation links, and diff checks passed.
- An initial overlapping Clippy build replaced the generated standard-library tree while Rust
  tests were reading it, causing 20 missing-manifest failures. A concurrent rebuild also hit a
  Windows executable lock. Repeating without overlapping build/check jobs passed; no production
  change was made for this verification interference. Julia's interactive terminal appearance
  was not manually inspected; its project and headless computation/cancellation were verified.
- No benchmark or speedup claim. The other fractal demos, the
  existing simple TCP tutorial, HTTP Serve, and OpenAI chat remain unchanged. TUI background
  messages and process escalation remain separate roadmap work.

### 2026-09-07 — selected two-stage shutdown and timed group close

- Committed the completed C1-C4 foundation as `d7a7aa71` with the requested co-author trailer.
- Added `CloseTaskGroupWithTimeout`: successful close returns the final failure array in Ok;
  timeout returns a distinct Error while preserving children, results, reports, and token storage.
  Close remains creator-only, sealed after cancellation, and retryable. Zero performs one probe;
  completion observed by a probe wins over expiry without claiming a historical completion time.
- Extracted the shared close path into `tasks/groups/close.rs`. Timed main-task waits do not run
  arbitrary queued children inline; child waits retain a monotonic deadline in cooperative
  suspension. Existing unbounded close behavior remains available.
- Nine new VM tests cover retained ownership through 100 repeated timeouts, 200 completion races,
  invalid inputs and ownership, deadline preservation, shutdown precedence, queued work, and a
  running worker that ignores cancellation. Integration tests cover the default pool, one worker,
  and deterministic debugger execution, including nested groups. Semantic checks cover argument
  rejection and the concrete Result/TaskFailure array type.
- `cargo build --workspace`, `cargo test --workspace` (all 544 VM tests), affected-crate strict
  Clippy, Rust/FPAS formatting, editor generation, documentation links, and the diff check passed.
  The standalone timeout regression passed. The complete `tests/suite.fpasprj` run passed:
  412 passed, 1 skipped, 0 failed. S1a is complete.
- The lifecycle plan records the approved process escalation and isolation policy, its
  cleanup/data-loss limits, and required future subprocess tests. S1b is not implemented by
  this Task operation.

### 2026-09-07 — documentation and example-index consistency

- Updated the repository README, Pascal documentation hub, language concurrency overview,
  scheduling and task-handle references, and standard-library roadmap to match the implemented
  Task surface. Pool activation uses verified task-start metadata, including group starts;
  child waits suspend cooperatively, and group-owned failures are contained until observed.
- Removed stale stack-machine wording from the spawn overview. No language, grammar, or runtime
  behavior changed. Timed cooperative close remains distinct from planned process escalation.
- Listed the three selection/group/supervision examples and added each to the existing CLI smoke
  test macros. Corrected the examples README's obsolete allowlist names.
- All 56 tests selected by `cargo test -p fpas-cli example_` passed, including the three new
  entries. Workspace build, Rust formatting, relative documentation links, and diff checks passed.
  The full workspace and FPAS suites were not repeated for this docs/test-registration-only
  follow-up; their timed-close verification is recorded above.

### 2026-09-07 — C4 compatibility audit and completion

- Completed the ordinary-channel compatibility audit. A capacity-one producer/consumer regression
  reproduced an inline-help deadlock with a timed send. Child channel operations and task waits
  now reuse cooperative suspension instead of retaining their waiting parent's stack. The same
  mechanism covers nested group close; normal sleep retains its existing scheduler timer path.
- Eight compatibility tests cover ordinary, cancellable, and timed sends and receives, five
  nested task-wait variants, and nested group close. Each runs with the default pool, one worker,
  and the deterministic debugger. The combined 1,024-message selection/supervision fixture passes
  in all three modes as well.
- Added debugger entry-completion coverage: forcing a supervised Result error terminates that
  task without executing its body or retrying, and produces exactly one group failure report.
- Shared task-clock deadlines round positive budgets upward without delaying zero-timeout probes.
  Three regressions verify real-time minimum budgets, exact manual-clock ticks, and saturation.
  Requested timer intervals are not hard wall-clock wakeup guarantees.
- The final `cargo test --workspace` run passed, including all 535 VM tests and the bundled FPAS
  concurrency suite. Workspace build, strict Clippy for the five affected crates, all three new
  examples, editor API generation, Rust/FPAS formatting, and diff checks passed.
- Current Task documentation covers the APIs, cooperative suspension, ownership, retry outcomes,
  and cancellation limits. C1-C4 are complete. S1 is not implemented: group close has no deadline,
  and neither non-cooperative workers nor blocking host calls acquire a hard termination guarantee.

### 2026-09-07 — capture lifetime and combined selection integration

- Added five direct capture-lifetime regressions, each repeated 100 times. A weak reference to
  the captured record's layout verifies retention between attempts and release after success,
  retry exhaustion, pre-admission cancellation, backoff shutdown, and queued-task teardown.
- Added a combined FPAS regression with eight supervised producers, a capacity-one channel,
  and send/receive/timer/cancellation selection. Four groups deliver 1,024 unique messages and
  close without failures. The same fixture runs with the default pool, one worker, and the
  deterministic debugger, checking 32 child identities despite retries.
- This integration exposed an inline-help deadlock: a pending child Select retained the stack
  of its waiting consumer. A two-message capacity-one regression reproduced it without retries;
  changing only capacity to two avoided it. Pending child selections now save their state and
  yield the pool thread, using the shared timer driver to request a new probe after one millisecond.
  Timer granularity and scheduler load still preclude a wall-clock wakeup guarantee.
- The original combined fixture and minimal regression pass with this repair. Added another
  100-iteration weak-reference check for shutdown of a parked selection. Resume-time task failures
  now pass through the same supervision outcome handling as instruction-time failures; a nested
  child-panic regression verifies recovery in normal and debugger execution.
- The full VM run passed all 522 tests before the additional resume-panic regression, which then
  passed independently. Editor generation, workspace build, the standalone combined FPAS test,
  and affected-crate strict Clippy passed. The fresh complete workspace verification passed,
  including all 523 VM tests and the bundled FPAS concurrency suite. Rust/FPAS formatting and
  the diff check passed. C3's selected supervision contract is implemented and verified.
- C4 remains open for the final compatibility audit, including combinations with ordinary
  blocking channel operations and debugger entry completion. No hard-shutdown contract is added.

### 2026-09-07 — supervision implementation and task-start scheduler repair

- Added bounded supervised attempts with one task identity, group membership, and final outcome.
  Retryable Result errors and Pascal panics restore the original callable inputs; successful
  values and procedures, exhausted retries, and other runtime errors are terminal. Pending
  backoff observes cancellation through the existing scheduler timer driver.
- Added the current Task reference, generated editor API, a standalone FPAS regression, and a
  runnable retry example. Eleven focused VM tests pass, with execution tests covering default-pool,
  single-worker, and deterministic-debugger modes.
- The standalone regression exposed missing spawn metadata: task-group and supervised starts
  did not activate the default worker pool or timer driver in programs without `go`. Compiler
  lowering and bytecode validation now share intrinsic task-start metadata. Added exhaustive
  intrinsic flag validation and a default-pool task-group timer regression; both pass.
- The previously blocked standalone retry test and example now pass with the rebuilt CLI.
  The workspace build passed. The earlier full test run was stopped after reproducing the
  missing-timer failure; the fresh complete workspace run passed, including all 513 VM tests
  built for that run and the bundled FPAS concurrency suite. The subsequently added nested-child
  regression passed in the focused eleven-test supervision run. It checks two worker attempts,
  two ordinary children in the same group, and exactly one supervisor identity. Strict Clippy
  for the five affected crates, Rust/FPAS formatting, and the diff check passed.
- C3 remains open pending repeated capture/teardown verification and final
  checks. C4 combined contention/lifetime coverage remains open. S1 is still reserved for the
  subsequent hard-shutdown contract discussion.

### 2026-09-07 — task-group completion

- C2 is implemented and verified. The current contract lives in the Task reference; the
  [task-group plan](task-groups.md) now records completion evidence rather than a proposed API.
- Added four semantic tests covering invalid handles, worker arity and parameter types, mutable
  captures, Result error types, and preservation of concrete task result types.
- Added nested-child lifecycle coverage in both normal single-worker and deterministic debugger
  execution. It checks discarded task handles, explicit cancellation, cancellation during close,
  and completion of a grandchild's final side effect before close returns.
- Added debugger cancellation and exited-child recovery rejection, creator-only close without
  self-join, explicit Wait failure propagation, and ignored-report metadata coverage. Added a
  200-iteration registration-versus-close race test with exact membership cleanup assertions.
- The Task-group slice now has 22 focused VM tests, four semantic tests, a bundled FPAS lifecycle
  test, and a runnable worker example. All 502 VM tests and the full workspace suite passed in the
  final run, including bundled FPAS tests. Workspace build, affected-crate strict Clippy, Rust/FPAS
  formatting, editor generation, the standalone lifecycle test, and the example also passed.
- Next: implement C3 according to [supervision](supervision.md), then perform C4 combined integration
  and lifetime verification. S1 remains a subsequent hard-shutdown contract discussion.

### 2026-09-07 — task-group ownership and failure containment in progress

- Added the initial C2 group registry, owned cancellation lifetime, admission bounds, explicit
  close, and failure collection. Reports retain registration order independently of consumed
  task results. Close releases retained child results, membership, and group-owned token storage.
- Connected failure ownership to the normal scheduler and deterministic debugger. Owned child
  failures publish terminal reports and exit events without stopping unrelated work. Debugger
  recovery cannot resume an already exited group child. Nongroup failure stops remain unchanged.
- Corrected failure positions to use diagnostic accessors and payload-free failure kinds to use
  the existing integer-backed enum representation. Extracted ordinary task spawning into its own
  module and shared initial register-state construction with group children.
- Added 14 tests covering owner/member permissions, close and cancellation, resource bounds and
  cleanup, report ordering, retained-result consumption, close racing failure publication, and
  normal/debugger execution with mixed child result types. Synthetic shutdown completion cannot
  certify a successful group join; close instead propagates shutdown or the original fatal error.
- All 494 VM unit tests passed, including after the spawn extraction in the final full workspace
  run. The workspace build, strict Clippy for the five affected crates, formatting, and diff
  checks passed.
- Added the Task-group API contract to the current Task reference and regenerated editor
  declarations after the workspace API-coverage test identified their omission. All 12 intrinsic
  editor-API tests and the repeated full workspace suite passed with those declarations present.
- C2 remains open: semantic rejection tests, cancellation and nested-worker integration,
  debugger recovery regressions, examples, and final verification. C3 supervision and C4 combined
  integration remain pending. No hard shutdown guarantee is added.

### 2026-09-07 — typed mixed-source selection

- Added `WaitCase`, `ReceiveCase`, `SendCase`, `TaskCase`, `TimerCase`, `CancellationCase`,
  `Select`, and `CloseWaitCase` across semantic checking, intrinsic registration, compilation,
  VM execution, and generated editor declarations. No language syntax or task typing changed.
- Cases are single-use, bounded, and owned by their creating VM/task. Whole-array claims reject
  invalid identities, duplicates, and wrong owners before removing any cases. Process-unique
  identities prevent cross-VM aliasing; explicit close and teardown release captured references.
- Selection validates sources before transferring values, scans in input order, and commits only
  the winner. Task completion is non-consuming; channel closure delivers the existing Result error.
  Registrations precede probing and are dropped before callbacks or scheduler helping.
- Winning callbacks use the existing resumable continuation, including on the main task and in
  the deterministic debugger. Timers share a start at Select entry. Cooperative helping does not
  introduce a hard shutdown guarantee.
- Added 16 focused VM tests, two semantic rejection tests, a bundled FPAS regression, and a runnable
  example. Checks cover heterogeneous channels, nested/suspending callbacks, pending receives,
  deterministic timers, wrong owners, source-validation precedence, losing sends/receives,
  case limits, capture release, and four concurrent selectors delivering 1000 values exactly once.
  Callback panic preserves a committed send, losing captures are released before callback execution,
  and normal/debugger wait teardown releases owned cases without invoking a callback.
- The focused tests, example, editor generation, workspace build, affected-crate strict Clippy,
  Rust/FPAS formatting, documentation links, and diff checks passed. The full workspace suite passed
  again after moving callback-reference cloning behind the case-capacity check; it includes all
  480 VM unit tests and bundled FPAS regressions.
- Current behavior is documented in `docs/pascal/std/concurrency/task.md` and linked from the std
  indexes. C1 is complete. Next is C2 task-group ownership and failure collection, followed by C3
  supervision and C4 combined integration. The final hard-shutdown contract remains reserved for discussion.

### 2026-09-07 — shared task and cancellation wake registrations

- Added wait-owned subscriptions to cancellation state and scheduler changes. Task completion,
  failure, debugger result replacement, runnable-work publication, and shutdown notify shared
  signals without consuming results or executing callbacks.
- Controlled `WaitAny` operations register one signal with the scheduler and cancellation source
  before probing either. They release registrations on return, error, each new observation cycle,
  and before helping queued work. Existing control precedence, bounded parking, and deterministic
  debugger suspension remain unchanged. Removed the unused condition-variable timeout branch.
- Added nine regressions covering pre-park and pre-registration cancellation, registration races,
  duplicate cleanup, invalid subscriptions, combined task/cancellation sources, runnable work,
  original failure retention, and shutdown. All 464 VM unit tests and the workspace build passed.
- The full workspace suite (including bundled FPAS tests), formatting, strict VM Clippy, and diff
  checks passed. Current API and language documentation remain unchanged because the externally
  specified contracts are unchanged. C1 still requires public
  typed cases, exactly-one channel commitment, callback continuation, and debugger integration;
  task groups, supervision, and C4 remain open.

### 2026-09-07 — wait-owned channel wake registrations

- Added a latched wake signal shared by event sources and RAII-owned subscriptions. Sources keep
  weak references only; dropping a registration removes precisely that subscription, including
  duplicate registrations of the same signal. Notifications do not consume values or run callbacks.
- Existing blocking channel send/receive paths register while holding the channel predicate lock,
  release it before parking, and remove their subscription before returning. Notifications arriving
  before parking remain visible. Close and VM teardown notify both operation directions.
- Extracted channel registry tests from the oversized implementation file. Six signal tests and
  ten channel tests cover pre-park notification, duplicate and multiple-source subscriptions,
  timeout/unwind cleanup, concurrent notification/removal, FIFO contention, and close/teardown.
- Current Task contracts remain unchanged. The public mixed-source operation, task groups, and
  supervision are still pending; this is their shared wakeup foundation, not C1 completion.
- Nine new regressions and the existing channel tests passed, as did formatting, the workspace
  build, strict VM Clippy, and the full workspace test rerun including bundled FPAS suites.
  The first workspace run was stopped after an HTTP server fixture remained blocked; that test
  passed both in isolation and in the complete rerun. No channel failure was observed. The exact
  fixture stall cause is unproven and was not folded into this change. Plan links and the diff were
  checked. No performance claim is made.

### 2026-09-06 — mixed task-result compiler repair

- Reproduced the procedure/integer task failure with local handles and direct `Wait(go ...)` calls.
  The first Unit-returning call fixed the shared intrinsic signature to Unit, rejecting subsequent
  value-returning calls. Reversing the call order avoided that validation failure.
- Extracted signature collection into `lowering/intrinsic_signatures.rs`. Shared signatures now
  account for every call site while each IR instruction retains its concrete result type. Intrinsics
  used only for Unit results keep their Unit signature.
- Corrected bytecode block widths to use each intrinsic call's result type, matching instruction
  selection's additional `LoadUnit`. Using the shared signature instead miscounted instructions
  and shifted branch targets after mixed-result calls.
- Added six compiler regressions covering both call orders, direct spawns, retained handles,
  routine and loop boundaries, and shared versus concrete IR types. All six regressions, formatting,
  workspace build, full workspace tests (including bundled FPAS suites), and strict compiler Clippy
  passed. The diff and the source documentation link were checked.
- Existing Task and language contracts are unchanged; this repairs their compilation. No new API,
  syntax, task-array typing rule, or runtime behavior was introduced.
- Next: settle typed value transfer and atomic winner ownership for mixed task/channel selection.

### 2026-09-06 — controlled task-completion barriers

- Added `WaitAnyWithTimeout` and `WaitAnyWithCancellation`, returning a completion index or
  distinct timeout/cancellation Result errors. Invalid identities and task failures remain runtime
  diagnostics. Neither control outcome consumes results or cancels the underlying tasks.
- Shared task-list validation and timeout-duration parsing with existing waits. The normal scheduler
  uses bounded condition-variable parking; debugger waits retain a single debugger-clock deadline.
  No helper thread per input or persistent source registration is introduced.
- Defined zero-timeout readiness, pre-cancellation, failure precedence, and late-completion behavior.
  Scheduler helping remains cooperative: blocking helped work can postpone the next control check,
  so these APIs do not establish a hard wall-clock shutdown bound.
- Added focused semantic, compiler/runtime, pending-operation, and deterministic debugger coverage.
  Updated the current Task reference and editor declarations. All nine new regressions, formatting,
  workspace build, full workspace tests (including bundled FPAS suites), and strict Clippy for every
  affected crate passed. Documentation links and the diff were checked. The full test run was
  repeated successfully in isolation after a concurrent build disrupted generated distribution
  files during the first run. This correctness slice makes no performance claim.
- Separately reproduced an IR validation failure using only existing `Wait` calls when a program
  retains both a procedure task and an integer-returning task. The failure reports an expected Unit
  result versus an actual Integer result. The later mixed task-result compiler repair above resolves
  this issue; the controlled-wait slice itself kept integer-returning fixtures consistently.
- Next: settle typed value transfer and atomic winner ownership for mixed task/channel selection.

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

## Implemented scope

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
- **Selected S1 split:** Timed group close stops waiting without releasing unfinished work.
  Hard termination of non-cooperative work belongs to explicit process-level lifecycle escalation,
  not forced termination of individual groups in a shared VM. The process-level requirement
  is implemented separately by `Std.Server`; ordinary group close still never terminates its host.
- Child panics and ordinary error results follow separately documented paths.
- High-contention tests demonstrate bounded memory and absence of lost wakeups.
