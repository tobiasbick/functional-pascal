# AP50.2: Process status and interactive launch

Package: [AP50](README.md). Status: planned; implementation not started.

## Prerequisites

Existing `Std.Proc` interactive processes and process-group/job ownership on
`main`. This work is independent of AP50.1.

## Contract and scope

The existing pipe operations already support the ACP data transport.
Two additions supply information or launch behavior currently unavailable
through the public process interface.

1. Add `ProcessExitStatus` with `ExitCode: Option of integer` and
   `Signal: Option of string`. Add `TryWaitStatus(Process)` returning
   `Result of (Option of ProcessExitStatus, ProcError)`. Pending is `None`;
   an exited child has a status record. Preserve cached status after pipe EOF
   and repeated probes. On Unix use the actual terminating signal and the
   documented canonical `SIG...` name, with decimal signal text as fallback.
   On Windows preserve the host exit code and leave Signal absent.
2. Add `RunInteractive(Command, Args, ProcessStartOptions, TimeoutMillis,
   CancellationToken)` returning `Result of (ProcessExitStatus, ProcError)`.
   TimeoutMillis is `Option of integer` and CancellationToken is
   `Option of Tasks.CancellationToken`; absent values impose no such bound.
   It inherits the host's interactive stdin/stdout/stderr while applying the
   supplied child cwd/environment. It does not borrow the ACP agent pipes,
   call a shell implicitly, or change the parent's cwd/environment.

Interactive launch uses existing command/configuration validation and owned
process-group/job cleanup. Cancellation, timeout and VM failure terminate and
reap its descendants before ownership is released. A terminal provider must
be available; otherwise return a typed unsupported/invalid-configuration error
before launch. A debugger or embedded host must explicitly supply interactive
terminal support rather than inheriting the debugger protocol's stdin.

Preserve `TryWait` and its existing `-1` fallback, all Run functions and pipe
polling contracts. Both status views read the same cached native result; neither
consumes the other's result or changes when descendants are cleaned up.
No PTY emulator, terminal-window manager, generic signal API or shell quoting
language belongs to this work package.

## Affected files and implementation owners

| Action | Path | Purpose |
| --- | --- | --- |
| Modify | `crates/fpas-vm/src/vm/hosted/proc/interactive.rs`, `platform.rs`, `platform/unix.rs`, `platform/windows.rs` | Preserve precise native completion and common process ownership |
| Modify | `crates/fpas-vm/src/vm/hosted/proc/arguments.rs`, `options.rs`, `engine.rs`, `interactive_dispatch.rs` | Interactive launch validation, inherited I/O and dispatch |
| Modify | `crates/fpas-sema/src/std_registry/loaded/proc.rs` | Public result record and operation types |
| Modify | `crates/fpas-std/src/std_units/symbols/std_symbols/proc.rs`, `crates/fpas-bytecode/src/intrinsic/proc.rs` | Names and intrinsic IDs |
| Modify | `crates/fpas-compiler/src/intrinsic_catalog.rs`, native execution metadata | Correct catalog/lowering and hosted ownership |
| Modify | `crates/fpas-cli/tests/proc_pipes.rs`, `proc_options.rs` | Distinct public process contract cases |
| Modify | `crates/fpas-vm/src/vm/hosted/proc/platform/tests.rs` | Unix signal and platform-native status cases |
| Generate | `lib/api/Std/Proc.fpas` | Accurate editor declarations |
| Modify | `docs/pascal/std/host/proc.md`, `docs/pascal/agents/tools.md` | New operations and retained legacy behavior |

Inspect console/host terminal hooks in `crates/fpas-vm/src/vm/hosted/console.rs`
and the console/input host abstractions. Add an explicit native interactive
launch capability for CLI and embedding hosts; debugger protocol streams do
not provide that capability. Split process files before they
become unrelated status/launch/pipe implementations in one large module.

## Acceptance and verification

- Normal zero/nonzero exit, pending, repeated status and output after exit
  agree between the two status views. Unix signal termination retains a signal
  and absent exit code; Windows exit codes are not relabeled as signals.
- Interactive child arguments, cwd and environment match the requested launch;
  inherited streams reach the configured terminal, while ACP pipes stay intact.
- Reject invalid configuration and unavailable terminal support before launch.
  Empty or already-cancelled budgets launch no child.
- Timeout, cancellation, runtime failure and host teardown leave no owned
  descendant or pipe worker alive. Extend existing lifecycle fixtures instead
  of creating a second process supervision test framework.
- A scripted terminal host proves interactive input behavior. Ordinary
  debugger protocol input is never consumed as login input.

Run the owning process tests on Windows and Unix, regenerate the Proc editor
declaration, and complete the relevant Rust/library checks. Report unavailable
platform coverage explicitly; a host test cannot establish another OS's signal
or inherited-terminal behavior.
