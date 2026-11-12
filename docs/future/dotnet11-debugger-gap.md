# Deferred .NET 11 upgrade

FPAS targets .NET 10. Retry the upgrade after a .NET 11 runtime update resolves
the Linux native debugger crash below. Runtime-native async remains deferred;
revisit C# implementation support and FPAS async emission during that upgrade.

The .NET 11.0.0 native debugging library can crash the FPAS debugger worker
when a target exits while a child process remains alive. Ordinary execution
passes the ownership test; execution with automatic failure capture can fail
with `worker_exited` or a disconnected worker. The failure is intermittent and
also reproduces without concurrent debugger tests.

## Reproducer and evidence

When validating a future upgrade, retarget the solution and generated programs
to .NET 11, build Release on Linux, and run:

```sh
dotnet test tests/Fpas.IntegrationTests/Fpas.IntegrationTests.csproj --no-build --no-restore --configuration Release --filter 'FullyQualifiedName~RunOwnershipTests' --logger 'console;verbosity=normal'
```

The `captureFailure: true` case starts a child, observes the parent's natural
exit, drains output and joins the child. The worker can terminate with SIGSEGV
(exit code 139). A captured native stack places the fault in `libmscordbi.so`;
disassembly of the failing instruction matches the transport send path's null
channel dereference. A crash during shutdown can occur after the host has
already received the target's successful exit.

The .NET 11 source clears `m_channel` in `CleanupTargetProcess()` without the
state lock used by `SendMessage()`. The latter can still observe `SS_Open` and
dereference that channel through `SendBlock()`. This is the suspected race;
matching Microsoft native symbols are unavailable under the current cloud
network policy, so the exact symbolic stack remains unverified.

## Required resolution

- Resolve the native transport crash through a supported runtime update or
  debugger API integration. Do not patch a locally installed runtime, add
  sleeps, or skip the ownership assertion to declare the upgrade complete.
- Repeat the ownership case, full debugger/async tests and failure-capture CLI
  tests against the resolution, including parallel execution and natural exit.
- Treat .NET 11 migration validation as incomplete while this case crashes.

Source references:

- [Runtime transport session](https://github.com/dotnet/runtime/blob/v11.0.0/src/coreclr/debug/shared/dbgtransportsession.cpp)
- [Native process exit handling](https://github.com/dotnet/runtime/blob/v11.0.0/src/coreclr/debug/di/process.cpp)
