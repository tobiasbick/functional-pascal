# Future: Server Lifecycle

> Core implemented in [`Std.Server`](../../pascal/std/network/server.md).
> Resource-specific protocol, storage, and observability adapters remain separate work.

## Implementation progress (2026-09-08)

- [x] A VM-owned lifetime, one owned task group, and explicit listener transfer.
- [x] Idempotent administrative, signal, normal-return, and fatal-error stop paths; sealed admission
  and readiness false before listener closure.
- [x] One monotonic grace deadline, remaining-budget queries, retained group ownership and failure
  reports after timeout, and explicit completion after application cleanup.
- [x] Host-authorized process abort on an independent watchdog thread, without VM execution or
  diagnostic-output progress. Standalone CLI/bundles authorize this; embedding defaults deny it.
- [x] Portable signal adapter with explicit registration; duplicate subscriptions, installation
  failures, and unsupported platform events have documented behavior.
- [x] Disposable-process tests confirm clean completion, termination of non-cooperative computation,
  blocked/failing host output, and survival of the parent. FPAS tests cover ownership, listener
  closure, retained failures, timeout and retry; runtime tests cover common deadlines and signals.
- [x] The parallel TCP echo example uses the lifecycle and an explicit escalation policy.

Verification: `cargo build` and the focused lifecycle tests passed (nine process/API integration
tests, three runtime unit tests, and three FPAS tests). After cleaning regenerable Cargo artifacts,
`cargo build`, `cargo test --workspace -j 2`, and strict workspace Clippy with all targets and
features passed. The initial disk-space blocker is resolved. Actual
signal delivery by the operating system and non-Windows execution remain platform validation work;
signal installation and the shared dispatch path are covered separately.
The complete FPAS suite passed: 428 passed, 1 intentionally skipped, 0 failed (429 total).
The standalone finite TCP echo smoke run, Rust/FPAS format checks, and the diff check passed.
No benchmark or performance claim is made for this API addition.

The current API requires application code to close its group, inspect failures, complete its own
cleanup phases, and call `FinishShutdown`. It does not infer application-specific cleanup order.
Database transactions, WebSocket session notifications, log/metric adapters, automatic worker-process
isolation, Windows service-control integration, and per-resource adapters are not implemented here.
Existing `Std.Proc` or an external supervisor can isolate a standalone application and confirm exit;
the lifecycle does not force-kill an individual group inside a shared VM.

Long-running servers need one coordinated lifetime instead of unrelated loops and stop flags. The
lifecycle module should turn host shutdown requests into cancellation, stop admission of new work,
drain owned work to a deadline, and report the final outcome.

## Original scope and remaining integration requirements

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

## Selected shutdown policy (2026-09-07)

The user selected cooperative group cleanup followed by explicit process-level escalation. This
policy is now implemented at the process boundary by `Std.Server`; resource-specific adapters
remain outside the current interface.

1. The lifecycle owner stops admission, marks readiness false, and requests cooperative
   cancellation. All cleanup phases share one monotonic grace-period deadline. A nested group
   close receives only the remaining budget, not a fresh full grace period.
2. Use `Std.Task.CloseTaskGroupWithTimeout` to observe owned group completion. Timeout means work
   remains owned, not detached or successfully terminated. Keep the group for retry or escalation;
   never discard its failure reports merely because a waiting call returned.
3. Escalation is explicitly selected by the host. Without a forced-exit policy, report incomplete
   shutdown and retain ownership. Do not silently terminate the embedding process from `Std.Task`
   or a VM destructor.
4. With forced exit selected, request termination of the entire application process after grace
   expires. Individual task groups sharing a VM are not force-killed in place. If the host must
   survive, run independently terminable work in a separate worker process and let its external
   lifecycle owner enforce the deadline.
5. Forced exit may skip destructors, finally-style cleanup, pending log flushes, and unfinished
   writes. It does not promise transaction commit, rollback of external effects, or peer delivery.
   Record incomplete resources before escalation where possible, but do not let logging extend
   the deadline. An external owner must distinguish a termination request from confirmed exit.

Blocking OS DNS, synchronous certificate verification, and non-cooperative computation must not
be disguised as successfully cancelled tasks. A hard process-exit policy must work without
requiring those calls to return or the blocked application's cleanup loop to run. Platform process
abort, signal handling, and disposable-process exit confirmation are implemented. Automatic worker
isolation remains separate work; the cooperative Task operation alone does not satisfy the
process-level requirements.

Required lifecycle tests include a cooperative clean exit, a worker ignoring cancellation, a
blocked host call, timeout with retained ownership, explicit escalation and confirmed process exit,
survival of an isolating host, repeated stop requests, and failing/blocked diagnostic output. Run
forced-exit tests in disposable subprocesses, never against the test runner or embedding host.

## Acceptance requirements

- Interrupt, administrative stop, listener failure, and normal completion share one cleanup path.
- New work is rejected after shutdown begins while admitted work receives its documented grace
  period.
- Database transactions, network sessions, queues, logs, and metrics close in a tested order.
- Repeated stop requests and partial cleanup failures remain deterministic.
- Platform-specific signal differences are normalized or documented without changing application
  logic.
