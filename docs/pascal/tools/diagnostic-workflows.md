# Collect traces, counters and controlled failure dumps

These procedures use the actual FPAS [diagnostic workload](../../../apps/diagnostics/).
They support native Windows x64 and WSL/Linux x64 with the .NET 10 SDK/runtime.
Use [unprofiled benchmarks](benchmarks.md) for timing comparisons.

The [diagnostic CLI](cli-diagnostics.md) supplies a separate owned frontend for
these tools, structured per-thread/GC summaries and debugger failure reports.

## Prepare tools and artifacts

The repository's [local tool manifest](../../../.config/dotnet-tools.json) pins
Microsoft's dotnet-trace, dotnet-counters and dotnet-dump packages. Restore it from
the repository root; global installations and editor integrations are unnecessary:

```sh
dotnet tool restore
dotnet tool run dotnet-trace -- --version
dotnet tool run dotnet-counters -- --version
dotnet tool run dotnet-dump -- --version
```

Tool restore requires NuGet access on first preparation; a populated cache permits
offline reuse. The target and collector must be the same architecture and run
as the same account. Build Workload in Debug and Release before collecting;
retain the matching DLL, portable PDB, sources and runtime dependencies. Set
`workload` and a fresh `run` directory as in the [benchmark setup](benchmarks.md#prepare).

Diagnostics must be enabled in the target runtime. Linux collectors and target
need the same TMPDIR so the diagnostic IPC endpoint can be found. Dump collection
needs readable target process information and permitted ptrace access; containers
may restrict those operations. Do not turn a permission/attachment error into a
successful empty recording. Collect and analyze dumps on the matching OS with
the matching target runtime/DAC available. These commands do not require changing
system crash handlers or enabling machine-wide collection.

## Inspect a busy CPU workload

```sh
dotnet tool run dotnet-trace -- collect --profile dotnet-sampled-thread-time,dotnet-common --output "$run/cpu.nettrace" -- dotnet "$workload" cpu 8000
dotnet tool run dotnet-trace -- report "$run/cpu.nettrace" topN -n 30
```

The report must resolve `Workloads.CpuBatch`, the non-inlined arithmetic kernel.
An empty trace or unresolved-only report is not evidence of a checked FPAS stack.
The target checks the sum-of-squares result and must exit zero. TopN's report
orders sampled stack time; it does not measure source lines or isolated instruction
costs.

In the pinned tool, the portable profile is `dotnet-sampled-thread-time`, not the
former `cpu-sampling` profile. It samples thread wall time, including waiting
runtime threads. A GC polling thread can occupy a large share of the aggregate
report without consuming that share of CPU. Correlate stacks with the workload
and process CPU counters. This is a busy-CPU workload; do not interpret thread
wall-time percentages as on-CPU percentages for arbitrary asynchronous programs.
The tool's separate `collect-linux` mode has different kernel/permission and
symbol requirements; it is outside this portable collection procedure.

## Observe allocations and GC

```sh
dotnet tool run dotnet-counters -- collect --format csv --counters System.Runtime --output "$run/runtime.csv" -- dotnet "$workload" alloc 8000
dotnet tool run dotnet-trace -- collect --profile gc-verbose --output "$run/gc.nettrace" -- dotnet "$workload" alloc 8000
```

The counter CSV must contain positive `dotnet.gc.heap.total_allocated` and
`dotnet.gc.collections` observations. Inspect generation tags, committed heap,
working set, CPU time and pause metrics together. Total allocated bytes are
cumulative managed allocation; they are not retained heap or process memory.
The workload retains only a bounded queue of arrays, so allocation growth alone
is not a leak. Collection counts and heap sizes have different units and sampling
boundaries. Very short targets can exit before the first periodic counter sample;
use the bounded multi-second workload.

The GC trace contains collection events and sampled allocation ticks. The
acceptance test reads the actual file with Microsoft's EventPipe reader and
asserts both event kinds; `gc-events.txt` records their observed counts. Sampled
allocation events are not an exact count of all allocated objects. Profiling can
change allocation rates and scheduling; preserve these recordings separately
from benchmark CSVs.

## Capture a controlled failure

The workload's `fault` mode catches and retains an InvalidOperationException from
`Workloads.CauseFailure`. It prints the real exception/source stack, writes its
PID to READY and waits for RELEASE or a 60-second bound, then exits 23. Start the
following command in one terminal, with a fresh run directory:

```sh
dotnet "$workload" fault "$run/fault.ready" "$run/fault.release"
```

In a second terminal set the same run path. PowerShell reads the PID with:

```powershell
$targetPid = [int](Get-Content "$run/fault.ready")
dotnet tool run dotnet-dump -- collect --process-id $targetPid --type Heap --output "$run/failure.dmp"
Set-Content "$run/fault.release" continue
```

Bash uses:

```sh
targetPid=$(cat "$run/fault.ready")
dotnet tool run dotnet-dump -- collect --process-id "$targetPid" --type Heap --output "$run/failure.dmp"
printf 'continue\n' > "$run/fault.release"
```

Wait for READY before reading it and collect within the holder's deadline. Verify
collector exit zero and a nonempty dump before releasing. The first terminal
must finish with exit 23. The dump is a controlled failure snapshot while the
exception is retained; it is not an OS crash dump. Analysis happens after the
target has exited:

```sh
dotnet tool run dotnet-dump -- analyze "$run/failure.dmp" -c "clrstack -all" -c "dumpheap -type System.InvalidOperationException -short" -c exit
```

The stack report must contain `Workloads.HoldFailure`. Take the exception object
address printed by dumpheap and inspect it, substituting that address below:

```sh
dotnet tool run dotnet-dump -- analyze "$run/failure.dmp" -c "pe EXCEPTION_ADDRESS" -c exit
```

If several exception objects exist, inspect them until the message is
`FPAS_DIAGNOSTIC_FAILURE`. Its recovered stack must contain `CauseFailure`.
The original stderr stack must name workloads.fpas and a line number, establishing
the portable-PDB/source relationship. Dump analysis must recover the exception
from memory; merely copying stderr is not dump analysis. SOS command failures
need inspection even when the outer analyze process exits zero. End batch
analysis with `-c exit` to avoid an interactive prompt.

`dotnet-dump` exposes managed stacks/heap, not a complete native debugger. Native
AOT uses a different runtime/diagnostic surface: do not apply these CoreCLR dump
or EventPipe acceptance claims to an AOT executable. Its unprofiled execution and
startup can be measured through the [benchmark procedure](benchmarks.md).

## Acceptance

For unattended acceptance, the integration tests perform the same steps, own the
target and collectors, wait for READY, check exit/cleanup and analyze the dump
after target exit. Run from the repository root after each toolchain build:

```sh
dotnet test tests/Fpas.IntegrationTests/Fpas.IntegrationTests.csproj -c Debug --no-build --no-restore --filter FullyQualifiedName~DiagnosticsWorkflowTests -- xUnit.MaxParallelThreads=1
dotnet test tests/Fpas.IntegrationTests/Fpas.IntegrationTests.csproj -c Release --no-build --no-restore --filter FullyQualifiedName~DiagnosticsWorkflowTests -- xUnit.MaxParallelThreads=1
```

The fixture packages the matching SDK with a unique version, builds real FPAS
applications outside the checkout, and restores the pinned tools. Default
artifacts are temporary. To retain evidence, set FPAS_DIAGNOSTICS_REPORTS to a
fresh absolute directory under `.temp-data/` before each run; use different
directories for Windows/WSL and Debug/Release. The retained subject directory
contains matching executable dependencies, PDB and workload sources.

Keep raw dumps/traces private: they contain process memory, paths and potentially
environment data. Commit source/contracts and assertions, not dumps, native
addresses, machine paths or performance histories. After all targets and
collectors have exited, delete the owning run directory when it is no longer
needed. A missing tool, failed attach, empty result or timeout is a failed
acceptance, not a skipped platform.

## See also

- [Profiling overview](profiling.md)
- [Unprofiled measurements](benchmarks.md)
- [Microsoft dotnet-trace](https://learn.microsoft.com/en-us/dotnet/core/diagnostics/dotnet-trace)
- [Microsoft dotnet-counters](https://learn.microsoft.com/en-us/dotnet/core/diagnostics/dotnet-counters)
- [Microsoft dotnet-dump](https://learn.microsoft.com/en-us/dotnet/core/diagnostics/dotnet-dump)
