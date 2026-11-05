# Public CoreCLR debugging entry points

Deferred technical option: access CoreCLR's public native debugging interfaces
directly. No backend implementation is selected here. The entry points below
provide runtime access; source mapping, session control and value presentation
would still belong to the debugger using them.

## Runtime discovery and connection: dbgshim

Start with the official [dbgshim exports](https://github.com/dotnet/diagnostics/blob/main/src/dbgshim/dbgshim.h)
and [implementation](https://github.com/dotnet/diagnostics/blob/main/src/dbgshim/dbgshim.cpp).
The native shim is available as
[Microsoft.Diagnostics.DbgShim](https://www.nuget.org/packages/Microsoft.Diagnostics.DbgShim).
It is a native dependency, not a complete managed debugger API.

| Entry point | Possible use |
| --- | --- |
| CreateProcessForLaunch | Start a target suspended so the debugger can register before runtime initialization. |
| RegisterForRuntimeStartup / Ex / 3 | Receive the runtime's ICorDebug interface through a startup callback. Supports launch and attach, including a runtime already loaded in the target. |
| ResumeProcess / CloseResumeHandle | Resume a suspended launch and release its launch handle. |
| UnregisterForRuntimeStartup | Remove the startup registration and release its token. |
| EnumerateCLRs / CloseCLREnumeration | Discover runtime modules in an existing process and release enumeration resources. |
| CreateVersionStringFromModule | Obtain the runtime-identifying string for interface creation. |
| CreateDebuggingInterfaceFromVersion and its variants | Obtain an ICorDebug interface for an identified runtime. |

A possible launch path is suspended creation, startup registration, process
resume, then initialization of ICorDebug in the callback. The startup callback
runs on another thread and can hold runtime initialization until it returns.
Callback synchronization and ownership must follow that lifecycle.

Attach can use the same startup-registration family with an existing process ID.
Explicit runtime enumeration and version-based interface creation are another
entry point when runtime selection is needed. Check the selected shim's supported
runtime-instance and export behavior rather than assuming every variant exists.

## Execution and inspection: ICorDebug

The authoritative interface definitions are in
[cordebug.idl](https://github.com/dotnet/runtime/blob/main/src/coreclr/inc/cordebug.idl).

| Interface or method | Possible use |
| --- | --- |
| ICorDebug.Initialize, SetManagedHandler, DebugActiveProcess | Initialize debugging services, install callbacks and connect to a live target. |
| ICorDebugManagedCallback and its versioned extensions | Receive module/thread/process events, breakpoint hits, step completion and exceptions. |
| ICorDebugProcess / ICorDebugController | Stop, continue, detach and control the target according to the callback contract. |
| ICorDebugModule, ICorDebugFunction, ICorDebugCode | Resolve metadata tokens and IL locations, then create breakpoints. |
| ICorDebugThread, ICorDebugFrame, ICorDebugILFrame | Inspect stacks, arguments and available local values. |
| ICorDebugStepper | Request instruction/IL-range stepping. Source and async stepping need symbol-aware policy above this interface. |
| ICorDebugValue and its specialized interfaces | Read primitive, reference, object, string and array values. |
| ICorDebugEval / ICorDebugEval2 | Invoke managed evaluation operations when the target permits it; this is distinct from passive inspection. |

Portable PDB metadata supplies documents, sequence points and scopes; CLR
metadata identifies modules, methods and types. Turning these into source
breakpoints or source-level values requires that mapping above ICorDebug.
The interfaces do not supply an FPAS expression evaluator.

## Data-target inspection: ICLRDebugging

[metahost.idl](https://github.com/dotnet/runtime/blob/main/src/coreclr/inc/metahost.idl)
defines ICLRDebugging.OpenVirtualProcess and ICLRDebuggingLibraryProvider interfaces.
OpenVirtualProcess obtains an ICorDebugProcess view over a caller-supplied
ICorDebugDataTarget, such as a dump or externally stopped process.

This is an entry point for inspecting an externally controlled data target. It
does not establish a live callback-driven launch/attach session. The caller owns
memory/context access, target consistency and the lookup/lifetime of matching
runtime debugging libraries. ICLRDebuggingLibraryProvider3 also participates in
the newer dbgshim live-connection variants.

## Official examples to inspect

The [DbgShim.UnitTests sources](https://github.com/dotnet/diagnostics/tree/main/src/tests/DbgShim.UnitTests)
provide concrete entry points for a managed implementation:

- [DbgShimAPI.cs](https://github.com/dotnet/diagnostics/blob/main/src/tests/DbgShim.UnitTests/DbgShimAPI.cs): native loading, exported functions and resource handling.
- [DbgShimTests.cs](https://github.com/dotnet/diagnostics/blob/main/src/tests/DbgShim.UnitTests/DbgShimTests.cs): launch/attach, runtime enumeration, interface creation and library-provider scenarios.
- [ICorDebug.cs](https://github.com/dotnet/diagnostics/blob/main/src/tests/DbgShim.UnitTests/ICorDebug.cs): a small managed wrapper around the native vtable.
- [ManagedCallbackWrapper.cs](https://github.com/dotnet/diagnostics/blob/main/src/tests/DbgShim.UnitTests/ManagedCallbackWrapper.cs): managed implementations of native callback interfaces.

These are test helpers and examples, not a supported complete debugger library.
They are useful references for ABI layout, HRESULT handling, callback lifetimes
and reference ownership. Native COM-style interfaces need explicit cross-platform
interop; ordinary Windows COM interop must not be assumed to work on every OS.

## Open evaluation

- Select a stable diagnostics/runtime version and supported OS/architecture pairs.
  The links above track upstream main; inspect matching release sources and exports
  before implementation.
- Establish deployment and version matching for dbgshim, DBI and data-access
  components, including the library-provider route where appropriate.
- Verify launch/attach, callbacks, stop/continue, detach and cleanup before adding
  symbol mapping, stepping, value inspection or evaluation.
- Validate debug and optimized code separately: the interfaces cannot reconstruct
  locals or execution locations that the compiler/JIT did not preserve.

Keep this document limited to open evaluation. Move selected contracts into the
owning reference and remove the document when the evaluation is complete.
