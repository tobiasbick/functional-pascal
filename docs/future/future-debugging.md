# Own CLR debugger core

Build an FPAS debugger on CoreCLR's public native debugging interfaces. One
protocol-neutral core serves DAP and one-shot batch debugging. Remove the current
MCP/NetCoreDbg implementation when the replacement passes acceptance. MCP and
JSONL are outside the selected design.

This plan contains open replacement work. The [current debugging guide](../pascal/tools/debugging.md)
and [batch reference](../pascal/tools/batch-debugging.md) describe the public surfaces. These plans own the replacement:

| Plan | Open work |
| --- | --- |
| This document | Shared architecture and replacement completion boundary. |
| [Frontend plan](debugger-frontends-plan.md) | DAP adapter and removal of MCP/NetCoreDbg. |
| [Acceptance plan](debugger-acceptance-plan.md) | Required Windows and native Linux/WSL evidence, packaging and completion gates. |

## Selected scope

- Support .NET 10 CoreCLR targets on native Windows x64 and Linux x64 under WSL2.
  Run Linux debugger and target processes inside WSL. Running a Windows binary
  from WSL does not establish Linux support.
- Launch an already built managed executable DLL with its runtime configuration,
  dependencies and matching portable PDB. Do not build, restore or discover a
  project implicitly. Keep the compiler's language and CLR representations.
- Provide source breakpoints, continue, pause, step into/over/out, CLR threads,
  stacks, lexical locals, arguments, captures, async source views, exception
  stops, passive value expansion and bounded passive FPAS expressions.
- Run the target through normal CoreCLR execution. CLR threads are not FPAS VM
  tasks. Do not introduce a scheduler or serialize execution instruction by
  instruction.

Public attach, dumps, Native AOT targets, native/mixed-mode debugging, mutation,
managed function evaluation, tracepoints, hot reload, instruction quotas and
deterministic scheduling are outside this baseline. Other OS/architecture pairs
need a separate acceptance decision. These exclusions are capability limits,
not hidden fallback paths through NetCoreDbg.

## Module layout and interface

Keep the implementation in the existing `Fpas.Debugging` library. Group it into
`Core`, `Native`, `Symbols`, `Values`, `Evaluation`, `Hosting`, `Batch` and `Dap`
directories with matching namespaces. Keep files focused and usually below 500
lines. `Fpas.Core` gains no debugger dependency. Read CLR metadata and portable
PDBs with `System.Reflection.Metadata`; do not load target assemblies to execute
their reflection or display code in the debugger.

The core's interface accepts typed launch/control/inspection operations and
returns typed results and ordered events. It owns session state, stop identity,
breakpoint binding, source mapping, value lifetimes, limits and errors. DAP names,
MCP types, argv and JSON documents do not appear in this interface. DAP and batch
use the same operations. Batch receives a detached snapshot through the core;
it cannot borrow native sessions or bypass inspection policy.

Native loading, vtables, HRESULT translation, callback implementations and
reference ownership stay inside `Native`. There is one selected CoreCLR
implementation. Do not create an interchangeable-backend framework or retain a
second product backend for compatibility. Reuse useful session/output/process
logic from the current implementation after removing its DAP-client coupling.

The [native connection reference](../../src/Fpas.Debugging/Native/README.md) owns
the pinned shim, ABI sources, callback lifetime and runtime-library lookup.
The [core reference](../architecture/debugger-core.md) owns the implemented
session supervision, source identities, threads, frames and bounded shutdown.

## Implementation order

Finish and verify each step through its production interface before starting
the next. Run the relevant native cases on Windows and inside WSL throughout
development; do not defer Linux interop and process ownership to final acceptance.

| Step | Deliverable | Required proof before advancing |
| --- | --- | --- |
| [F3](debugger-frontends-plan.md#f3-remove-mcp-and-integrate-the-replacement) | Product cutover, deployment and documentation. | Delivered launchers pass the full matrix without MCP, JSONL or NetCoreDbg. |

The [acceptance plan](debugger-acceptance-plan.md) owns the complete matrix.
After accepting a step, move its behavior to the owning reference and remove
its completed tasks here. Do not retain this table as a completion history.

The implemented [value contract](../architecture/debugger-core.md#passive-stored-values)
and [passive expression subset](../architecture/debugger-expressions.md) belong
to their references. Batch and DAP must use these same bounded operations.
Managed function evaluation and execution of display/proxy code require a later
explicit scope decision.

## Sources and completion

Use official sources for interop and protocol behavior:

- [dbgshim exports](https://github.com/dotnet/diagnostics/blob/main/src/dbgshim/dbgshim.h)
  and [implementation](https://github.com/dotnet/diagnostics/blob/main/src/dbgshim/dbgshim.cpp).
- [DbgShim package](https://www.nuget.org/packages/Microsoft.Diagnostics.DbgShim)
  and [managed launch/callback examples](https://github.com/dotnet/diagnostics/tree/main/src/tests/DbgShim.UnitTests).
- [.NET 10 ICorDebug definitions](https://github.com/dotnet/runtime/blob/v10.0.0/src/coreclr/inc/cordebug.idl)
  and [public debugging reference](https://learn.microsoft.com/en-us/dotnet/core/unmanaged-api/debugging/).
- [DAP overview and framing](https://microsoft.github.io/debug-adapter-protocol/overview)
  and [specification source](https://github.com/microsoft/debug-adapter-protocol/blob/main/debugAdapterProtocol.json).

Upstream `main` links are discovery references. Follow the pinned release sources
in the native reference before revising exports or layouts. `ICLRDebugging.OpenVirtualProcess`
is for caller-supplied data targets and is not the live launch implementation.
The Rust debugger supplies architectural examples, not CLR runtime behavior.

Close this plan only with the [acceptance gates](debugger-acceptance-plan.md).
Move implemented contracts into the architecture/handbook references, remove
completed entries and delete the plan once no open work remains.
