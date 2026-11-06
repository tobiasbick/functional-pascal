# Debugging CLR programs

The FPAS debugger uses one CoreCLR core for [batch reports](batch-debugging.md)
and a stdio DAP adapter. It supports .NET 10 targets on Windows x64 and Linux x64,
including native Linux processes inside WSL2. Its native shim is packaged with
the compiler. No external debugger installation or debug-time download is needed.

Build the executable separately. Keep its DLL, matching portable PDB, runtime
configuration, dependencies and recorded sources available. Source checksums and
module identities must match. Debugging does not build or restore a project.
Use `fpas env --json` to inspect packaged debugger/runtime requirements.

```sh
fpas debug artifacts/Mandelbrot.dll --cwd examples/mandelbrot --break main.fpas:13 --print Iteration
fpas debug --protocol dap
```

Batch is the default and returns one detached JSON report. It captures the first
requested breakpoint/exception, then terminates its owned target. Normal exit,
inspection errors, deadlines and output limits have explicit report outcomes and
exit codes. See the [batch contract](batch-debugging.md) for input, paths and bounds.

## Interactive DAP session

Start `fpas debug --protocol dap` through a stdio DAP client. Launch takes an
absolute built DLL, optional absolute cwd and individual argv elements. Configure
source breakpoints and exception filters before configurationDone; the core holds
the target internally at entry until configuration is complete.

The core stops all CLR threads. Source steps operate on the actual stop thread;
selecting another thread for a step returns an explicit error. Frame/value
handles expire after resume. Each adapter owns one launch and waits for owned
cleanup before acknowledging shutdown. EOF, cancellation and transport failure
also clean up the worker and target. Attach and restart are outside this protocol.

Target stdin is closed in DAP. Batch `--stdin` can supply an explicit file;
interactive console input and runInTerminal are unsupported.

## Passive values and expressions

Scopes separate source locals/arguments, statics and raw CLR storage. The source
view exposes supported records, unions, domains, captures and resumed async locals
through stored metadata/fields. Cycles and unavailable or optimized-out values
are explicit. Expansion, hover and watch never execute getters, constructors,
methods, static initializers, ToString or debugger proxy/display code.

The [expression reference](../../architecture/debugger-expressions.md) defines
supported literals, primitive operators, stored fields and array/string indexes.
Unsupported expressions return an error while preserving the stopped context.
The [core reference](../../architecture/debugger-core.md) owns value identities,
source mapping, native supervision, stepping and exception policy.

## DAP protocol and bounds

The [adapter source reference](../../../src/Fpas.Debugging/Dap/README.md) pins the
Microsoft DAP schema. Transport uses UTF-8 JSON with byte-counted Content-Length
headers. It supports initialize, launch, configurationDone, source breakpoints,
raised/unhandled exception filters, threads, stackTrace, scopes, variables,
evaluate, exceptionInfo, continue, pause, next, stepIn, stepOut, cancel,
disconnect and terminate. Configuration is completed at an internal entry hold
before execution; `stopAtEntry` defaults to false.

Launch takes absolute program/cwd paths and individual argv elements. Initialize
negotiates path/file-URI form and zero/one-based UTF-16 source coordinates.
Unsupported URI schemes, conditional/data/function breakpoints, logpoints,
mutation and method evaluation are rejected. Only implemented capabilities are
advertised. Output events preserve stream categories and expose discarded text.

Headers are bounded at 8 KiB, message bodies and the complete queued output at
1 MiB, pending requests at 64 with reserved control slots, and frame/value handles
at 4096 each per stop. Thread catalogs have at most 1024 entries. Stack/value pages
have at most 128 entries and retain the core's byte bound. An unpaged request
that cannot fit returns a limit error; a shortened page never silently implies
the end of a catalog. Paging clients receive known named/indexed totals.

Startup has a 30-second deadline, ordinary operations 10 seconds and shutdown a
5-second grace. An idle breakpoint has no deadline. Cancelling active native
work ends its unresponsive worker; ordinary pause remains available while the
target runs. Control events are not silently discarded under backpressure; an
unserviceable transport fails and releases the launch.

`terminate` and `disconnect` await owned cleanup, including forced termination
of an unresponsive worker, before acknowledging shutdown. Pending ordinary
requests complete with their cancellation responses even when their queue is full.

FPAS [portable debug metadata](../../architecture/debug-information.md) is also
usable by other CLR debuggers. Their evaluation and display policies are owned
by those tools; the passive policy above belongs to the FPAS debugger.

## See also

- [Batch debugging](batch-debugging.md)
- [Debugger core](../../architecture/debugger-core.md)
- [Debug information](../../architecture/debug-information.md)
