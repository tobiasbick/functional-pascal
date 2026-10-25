# AP50.2: Process status and interactive launch

Package: [AP50](README.md). Status: complete.

## Prerequisites

Existing `Std.Proc` interactive processes and process-group/job ownership on
`main`. This work is independent of AP50.1.

## Contract and scope

The process API supplies precise completion and explicit interactive launch
alongside the existing binary pipes. These operations are independent of the
ACP connection driver.

1. `ProcessExitStatus` has `ExitCode: Option of integer` and
   `Signal: Option of string`. `TryWaitStatus(Process)` returns
   `Result of (Option of ProcessExitStatus, ProcError)`. Pending is `None`;
   an exited child has a status record. Cached status survives pipe EOF and
   repeated probes. Unix retains the actual terminating signal and its canonical
   `SIG...` name, with decimal signal text as fallback. Windows preserves the
   unsigned 32-bit native exit code in `integer` and leaves Signal absent.
2. `RunInteractive(Command, Args, ProcessStartOptions, TimeoutMillis,
   CancellationToken)` returns `Result of (ProcessExitStatus, ProcError)`.
   TimeoutMillis is `Option of integer` and CancellationToken is
   `Option of Tasks.CancellationToken`; absent values impose no such bound.
   It inherits the host's interactive stdin/stdout/stderr while applying the
   supplied child cwd/environment. It does not borrow the ACP agent pipes,
   call a shell implicitly, or change the parent's cwd/environment.

Interactive launch uses existing command/configuration validation and owned
process-group/job cleanup. Cancellation, timeout, enclosing task-scope failure
and VM failure terminate and reap descendants before ownership is released,
including calls without an explicit timeout or cancellation token. Without an
installed terminal provider the call returns `InvalidArgument('Terminal', ...)`
before launch. A debugger or embedded host explicitly supplies interactive
terminal support rather than inheriting the debugger protocol's stdin.

Legacy `TryWait` keeps its signed host-code conversion and `-1` fallback. Other
Run functions and pipe polling retain their contracts. Both status views read
the same cached native result; neither
consumes the other's result or changes when descendants are cleaned up.
No PTY emulator, terminal-window manager, generic signal API or shell quoting
language belongs to this work package.

## Implementation owners

| Action | Path | Purpose |
| --- | --- | --- |
| Modify | `crates/fpas-vm/src/vm/hosted/proc/interactive.rs`, `platform.rs`, `registry.rs` | Preserve precise native completion and common process ownership |
| Modify | `crates/fpas-vm/src/vm/hosted/proc/arguments.rs`, `mod.rs`, `interactive_dispatch.rs` | Interactive launch validation and hosted dispatch |
| Create | `crates/fpas-vm/src/process_terminal.rs`, `vm/hosted/proc/status.rs`, `run_interactive.rs`, `platform/foreground.rs` | Explicit host terminal provider, exact status conversion, owned launch and Unix foreground restoration |
| Modify | `crates/fpas-vm/src/lib.rs`, `vm/hosted/mod.rs`, `vm/debug/session/io.rs` | Export provider and install it explicitly per VM/debugger host |
| Modify | `crates/fpas-cli/src/cli_run.rs`, `src/bin/fpas-runner.rs` | Offer actual inherited terminals for CLI/native programs |
| Modify | `crates/fpas-sema/src/std_registry/loaded/proc.rs` | Public result record and operation types |
| Modify | `crates/fpas-std/src/std_units/symbols/std_symbols/proc.rs`, `crates/fpas-bytecode/src/intrinsic/proc.rs` | Names and intrinsic IDs |
| Modify | `crates/fpas-compiler/src/intrinsic_catalog.rs`, native execution metadata | Correct catalog/lowering and hosted ownership |
| Modify/verify | `crates/fpas-cli/tests/proc_pipes.rs`, `proc_options.rs` | Distinct public process contract cases and retained pure defaults |
| Create | `crates/fpas-vm/tests/process_terminal.rs`, `tests/process_terminal/foreground.rs` | Public embedding, exact native status, process receipts, scope failures and a real Unix controlling terminal |
| Verify | `crates/fpas-vm/src/vm/hosted/proc/platform/tests.rs` | Existing Unix pipe and group ownership cases |
| Generate | `lib/api/Std/Proc.fpas` | Accurate editor declarations |
| Modify | `docs/pascal/std/host/proc.md`, `docs/pascal/agents/tools.md` | New operations and retained legacy behavior |

`InteractiveProcessTerminal` opens owned child streams without borrowing Console
input queues. The CLI and native runner offer inherited streams only when all
three are actual terminals. The embedding/debugger setter installs a separate
provider; debugger protocol streams never provide that capability. Unix foreground
ownership is transferred to the child group and restored after cleanup.

## Regression coverage

- Normal zero/nonzero exit, pending, repeated probes and output after exit use
  one cached completion with the documented conversion for each status view.
  Unix signal termination retains a signal and absent exit code; Windows native
  codes retain all 32 bits and are not relabeled as signals.
- Interactive child arguments, cwd and environment match the requested launch;
  inherited streams reach the configured terminal, while ACP pipes stay intact.
- Reject invalid configuration and unavailable terminal support before launch.
  Empty or already-cancelled budgets launch no child.
- Timeout, cancellation, runtime failure and host teardown leave no owned
  descendant or pipe worker alive. Extend existing lifecycle fixtures instead
  of creating a second process supervision test framework.
- A scripted terminal host proves interactive input behavior. Ordinary
  debugger protocol input is never consumed as login input.
- A real Linux controlling terminal admits input to the foreground child and
  is restored afterward. A sibling task failure interrupts a tokenless,
  unbounded interactive call and preserves the original scope failure.

The owning VM/CLI tests protect these contracts on Windows and Unix. The Linux
controlling-terminal case runs in an isolated terminal session; scripted streams
protect the portable host interface. API export, ordinary FPAS tests and the
required workspace checks cover registration, artifacts and existing behavior.
The complete ACP client remains owned by the other AP50 work packages.
