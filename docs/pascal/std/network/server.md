# Std.Server

Server lifetime ownership for cooperative stop, listener closure, a shared shutdown budget,
and explicitly authorized process escalation. Import with `uses Std.Server`.
There is no new language syntax and no change to ordinary task-group close semantics.

## Quick reference

| Kind | Signature | Description |
|------|-----------|-------------|
| type | `ServerLifetime` | Opaque VM-local identity for one server lifetime |
| function | `CreateLifetime(GraceMillis: integer; ForceExit: boolean): result of ServerLifetime, string` | Creates a ready lifetime with its own task group and stop token |
| function | `GetWorkGroup(Lifetime: ServerLifetime): TaskGroup` | Group for all admitted server workers; use `Std.Task.StartTaskInGroup` |
| function | `GetStopToken(Lifetime: ServerLifetime): CancellationToken` | Token notified by any lifecycle stop request |
| function | `IsReady(Lifetime: ServerLifetime): boolean` | True before shutdown begins; false once admission is sealed |
| function | `RequestStop(Lifetime: ServerLifetime): boolean` | Starts shutdown once; later calls return false without extending the deadline |
| function | `RemainingMillis(Lifetime: ServerLifetime): integer` | Remaining common grace budget, rounded up to milliseconds and clamped at zero |
| function | `OwnListener(Lifetime: ServerLifetime; Listener: Listener): result of boolean, string` | Transfers TCP/TLS listener cleanup to this lifetime |
| function | `ObserveSignals(Lifetime: ServerLifetime): result of boolean, string` | Explicitly subscribes this lifetime to process stop signals |
| function | `ShutdownErrors(Lifetime: ServerLifetime): array of string` | Snapshot of retained listener-close failures, without consuming them |
| function | `FinishShutdown(Lifetime: ServerLifetime): result of boolean, string` | Confirms caller-managed cleanup and disarms escalation after explicit group close |

Import `Std.Task` for the group/token types and `Std.Net` for `Listener`.
Foreign, fabricated, or wrong-kind handles raise a runtime diagnostic. Fallible configuration,
ownership transfer, signal installation, and incomplete shutdown return `Error(string)`.

## Ownership and stop ordering

`CreateLifetime` accepts 0..86400000 milliseconds. The budget starts at the first stop request,
not at creation. A VM permits at most 256 lifetime identities, including completed identities;
retaining completed records makes repeated completion idempotent. Each lifetime owns one task
group and at most 32 listeners. Existing task-group child limits still apply.

The creating task owns the group and must collect its failure reports. Workers join this group
through `StartTaskInGroup` or `StartSupervisedTask`; do not launch detached server work with `go`.
Children and the owner retain the existing task-group admission rules. `RequestStop` is callable
from any task, including a worker reporting a fatal dependency error.

The first stop request establishes one monotonic deadline, seals group admission and makes
readiness false, notifies the stop token, closes transferred listeners, and retains close errors.
Already admitted workers receive cooperative cancellation. Connections are still owned by their
session workers: those workers must close them on success and failure. Stop notification does not
prove that computation or a blocking host call has ended.

`OwnListener` returns `Ok(false)` for a listener already owned by this lifetime. Transfer to a
different lifetime, transfer after stop, and invalid/closed listeners return an error. After
transfer, applications must not close the listener themselves; failure tests deliberately violate
this rule to verify that cleanup diagnostics are retained.

## One shutdown budget

Use `CloseTaskGroupWithTimeout(GetWorkGroup(Life), RemainingMillis(Life))` to join workers.
Pass the remaining budget to every additional cleanup phase instead of granting each phase a
fresh full grace period. Before stop, `RemainingMillis` returns the configured grace period.
After expiration it remains zero, including after repeated stop requests.

`Ok(Failures)` from group close confirms completion, not success of each worker. Inspect these
reports and `ShutdownErrors`. A timeout retains the group and its reports; keep the lifetime and
retry close when completion becomes possible. `FinishShutdown` does not join, discard failures,
force-close a group, or certify that application-owned writes have been committed.

After workers close, finish application-specific session notifications, durable-state flushing,
queue and log cleanup in the application's required order, using the same budget. Call
`FinishShutdown` only when these phases have actually completed. It requires the creating task,
a prior stop request, completed listener closure, and an explicitly closed work group.
Successful and repeated completion return `Ok(true)`. Close errors remain available afterwards.

Returning from the VM root or a fatal VM failure requests the same stop path for unfinished
lifetimes. Returning without explicit `FinishShutdown` reports incomplete shutdown rather than
success. Ordinary VM teardown can still wait for non-cooperative work; without escalation it
does not turn a timeout into a successful shutdown.

## Process authority and signals

`fpas run` and bundled standalone applications authorize process lifecycle operations. A program
must still explicitly choose `ForceExit = true` and/or call `ObserveSignals`.
An embedded `fpas_vm::Vm` defaults to neither authority. Its host may explicitly call
`Vm::allow_process_lifecycle()` before running. Source-debugger instances do not authorize process
control, and recording capture rejects `Std.Server` host effects.

`ObserveSignals` returns `Ok(true)` for the first subscription and `Ok(false)` for a duplicate.
It reports installation conflicts rather than overwriting an existing Unix signal handler.
The process-wide handler remains installed for the rest of the process; this is an explicit host
ownership decision, not a temporary per-VM signal hook. Signals request lifecycle cancellation,
not application callbacks. At most 16 active subscriptions are admitted.

The [ctrlc adapter](https://docs.rs/ctrlc/3.5.2/ctrlc/index.html) handles Ctrl+C and Ctrl+Break on
Windows, and SIGINT, SIGTERM, and SIGHUP on Unix. Windows service-control events, console close,
forced OS termination, and SIGKILL are not normalized into a portable grace period. Applications
without a subscribed lifetime receive no lifecycle notification from the installed handler.

## Explicit escalation

`ForceExit = false` never terminates the hosting process. A timed-out lifetime remains owned and
must later be joined; embedding hosts that require independently terminable work must isolate it
in a child process.

With host authority and `ForceExit = true`, an independent host thread watches the common deadline
and aborts the entire application process when it expires before `FinishShutdown`. It does not
wait for a VM instruction boundary, return from DNS/TLS/other blocking host calls, or diagnostic
output. Its poll interval is 10 ms; operating-system scheduling prevents a hard real-time promise.
This is a process abort, not a graceful numeric exit code or an in-place task kill. An external
supervisor must wait for the child and inspect its exit status to confirm termination.

Escalation can skip destructors, buffered logs, unfinished writes, and external protocol cleanup.
It cannot promise transaction commit/rollback or peer delivery. Report known pending resources
before the deadline where possible, but logging must not be relied on to make escalation work.
There is no automatic subprocess isolation or database/log adapter in this unit.

## Example

The [parallel TCP echo server](../../../../examples/network/tcp_parallel_echo_server.fpas) uses
an owned listener, four group-owned sessions, a finite run duration or process signal, and an
explicit one-second escalation policy. It runs only on loopback by default.

```sh
fpas run examples/network/tcp_parallel_echo_server.fpas -- 18082 10000
```

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Signatures | `crates/fpas-sema/src/std_registry/loaded/server.rs` |
| Intrinsics | `crates/fpas-bytecode/src/intrinsic/server.rs` |
| Runtime and watchdog | `crates/fpas-vm/src/vm/hosted/server/` |
| Disposable-process regressions | `crates/fpas-vm/tests/server_lifecycle.rs` |
| FPAS ownership regressions | `tests/stdlib/server/` |

## See also

- [Networking](README.md)
- [Standard library](../README.md)
- [Task groups and timed close](../concurrency/task.md)
- [Server lifecycle progress](../../../future/networked-applications/server-lifecycle.md)
