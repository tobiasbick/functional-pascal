# Diagnostic CLI commands

`fpas profile` collects and summarizes .NET observations. `fpas diagnose` captures
failures through the debugger core or collects/analyzes dumps. Both operate on
explicit built targets; they do not build applications. Use
[benchmarks](benchmarks.md) for unprofiled timing.

## Prepare the Microsoft tools

```sh
fpas profile setup
fpas profile setup --tools-directory .temp-data/diagnostic-tools --timeout 300
```

Setup installs dotnet-trace, dotnet-counters and dotnet-dump 10.0.750501 into a
private versioned directory under the user's local application data by default.
It uses an isolated NuGet configuration with nuget.org; an existing configuration
in that tools directory can select an offline feed. A populated download cache
can be reused. Setup is idempotent and serializes concurrent installation into
the same directory. It does not update global tools or the repository manifest.

Collection needs those prepared tools and a .NET host. Summary decoding uses the
compiler's packaged TraceEvent dependency and does not invoke a global profiler.
The [manual diagnostic workflow](diagnostic-workflows.md) remains available through
the separately restored repository tool manifest.

## Collect and summarize

```sh
fpas profile collect apps/diagnostics/Workload/bin/Release/net10.0/Workload.dll --mode threads --duration 5 --output cpu.nettrace -- cpu 8000
fpas profile collect apps/diagnostics/Workload/bin/Release/net10.0/Workload.dll --mode gc --duration 5 --output gc.nettrace -- alloc 8000
fpas profile collect apps/diagnostics/Workload/bin/Release/net10.0/Workload.dll --mode counters --duration 5 --output runtime.csv -- alloc 8000
fpas profile summary cpu.nettrace --limit 20
fpas profile summary gc.nettrace
```

Build the [diagnostic workload](../../../apps/diagnostics/) before these examples.
A DLL selects the installed dotnet host and requires its runtimeconfig.json.
A published executable can be selected through its absolute native path.
Arguments after `--` are individual target
arguments, without a shell. `--cwd` selects its working directory. Collection
duration is 1..300 seconds, default 10. Target exit can end collection earlier.
Microsoft's collector owns launch, duration and target-exit interpretation; its
captured diagnostics and exit code remain in `process`.

`threads` selects `dotnet-sampled-thread-time,dotnet-common`; `gc` selects
`gc-verbose`. Counters select System.Runtime in CSV format. Use `.nettrace` for
threads/GC and `.csv` for counters. `--tools-directory` selects an explicitly
prepared tool installation.

Output must be new. The collector writes into an invocation-owned staging
directory and publication refuses an existing destination, including one created
concurrently. Failed/interrupted captures can retain a partial artifact; the
failure report identifies it when available. Never treat a partial trace as a
successful recording. Raw traces/dumps can contain private process data.

Summary reports decoded event totals, GC starts, sampled allocation ticks, lost
events and per-thread sample counts with inclusive method counts. A method is
counted once per stack sample, even under recursion. `--limit` selects the top
1..100 methods per thread; `totalMethods` exposes the unpaged count. Unresolved
frames remain explicit. No symbol download or guessed source mapping occurs.

These are **thread wall-time samples**, including waiting runtime threads. They
are not CPU utilization, thread scheduling transitions, exact allocation counts
or a deterministic execution history. Correlate them with counters and the
workload. An unresolved-only trace does not prove useful source profiling.

Decoding accepts files up to 512 MiB and bounds transferred events to 5 million,
threads to 256, distinct methods per thread to 4096 and stack depth to 128. Lost
events or an internal bound produce `incomplete`, `success=false` and exit 1.
The requested top-method page alone is not an incomplete recording. Conversion
runs in an owned private CLI worker so its non-cancellable library work is also
subject to the command deadline. Intermediate converted files are removed after
that worker and its descendants exit.

## Failure reports and dumps

```sh
fpas diagnose failure artifacts/App.dll --timeout 20 -- input.txt
fpas diagnose dump --process-id 1234 --output failure.dmp
fpas diagnose inspect-dump failure.dmp --command "clrstack -all" --command "dumpheap -type System.InvalidOperationException -short"
fpas diagnose inspect-dump failure.dmp --command "pe EXCEPTION_ADDRESS"
```

Failure mode uses the [batch debugger](batch-debugging.md), stops at an unhandled
exception, and captures actual exception information, source locations, managed
thread/frame context and passive bindings. `--print` requests additional passive
expressions. `--stdin` provides a bounded input file; otherwise input is EOF.
The owned program is terminated after capture. A captured exception exits 3;
natural exit, observation/setup failures, limits and cancellation follow the
batch contract. Captured locals come from the debugger, never from stderr text.

Dump collection requires an explicit live PID and creates a Heap dump. The CLI
owns the collector; an externally selected target remains under its caller's
ownership. Keep it alive until collection succeeds. Analysis can happen after
target exit, with matching OS/architecture/runtime dependencies. The controlled
[failure workload](../../../apps/diagnostics/) provides a bounded collection target.

Inspection forwards 1..16 SOS commands of at most 4096 characters each, appends
`exit`, and captures stdout/stderr. Default is `clrstack -all`. The tool's outer
exit status describes the analysis process; inspect SOS diagnostics too because
individual commands can fail while that process exits zero. Addresses are
snapshot-local. The [manual procedure](diagnostic-workflows.md#capture-a-controlled-failure)
shows how to recover the actual exception object and its saved throw stack.

CoreCLR debugger/EventPipe/dump support does not establish Native AOT parity.
Use the appropriate runtime diagnostics and native tools for an AOT application.

## Tracepoints

```sh
fpas debug artifacts/App.dll --tracepoint main.fpas:12 --print Iteration --max-hits 50
```

Tracepoints use real source breakpoints. At each hit the debugger captures a
detached stop, threads, frames, passive bindings and requested expressions, then
continues. `tracepoints` preserves observation order; its entries use the batch
observation shape. No target getter or method is executed during inspection.
The original breakpoint report includes hit counts. Every requested tracepoint
must be hit before a successful natural exit; missed points and unavailable
requested expressions fail the observation.

Do not combine tracepoints and one-shot `--break`. `--max-hits` requires a
tracepoint, defaults to 100 and accepts 1..256. Reaching that bound terminates
the target with exit 4 before further user code executes; it is not a complete
execution. The whole invocation retains the batch output, time and 8 MiB report
ceilings. Report truncation is explicit. An unhandled exception ends tracing
with its actual failure observation. Recording pauses the application and is
separate from sampling, replay and performance measurements.

## Reports, deadlines and cleanup

Profile and dump operations always emit one [operation report](report-schemas.md).
Failure/tracepoint observations use the debugger schema. Argument errors exit 2;
tool/decoding/artifact errors and deadlines exit 1; cancellation exits 130.
Profile defaults to 120 seconds and dump inspection/collection to 30;
`--timeout` accepts 1..3600. Failure mode uses the batch deadline/exit contract.

Collectors, targets launched by a collector and decoder workers are owned as
Windows jobs or Linux process groups. Cancellation/deadlines terminate and join
them before a result is returned. Cleanup has a bounded grace period. Captured
tool output is limited to 1 MiB; decoder/operation JSON to 4 MiB. Retain raw
artifacts privately under `.temp-data/`, then remove the owning directory after
all processes have exited.

## See also

- [Manual traces, counters and dumps](diagnostic-workflows.md)
- [Profiling overview](profiling.md)
- [Unprofiled benchmarks](benchmarks.md)
- [Batch debugger](batch-debugging.md)
- [Microsoft diagnostic tools](https://learn.microsoft.com/en-us/dotnet/core/diagnostics/tools-overview)
