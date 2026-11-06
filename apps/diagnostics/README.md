# Diagnostic workloads and measurements

Two CLR FPAS applications support the [benchmark](../../docs/pascal/tools/benchmarks.md)
and [diagnostic](../../docs/pascal/tools/diagnostic-workflows.md) procedures. They
use ordinary .NET APIs. Neither application adds a compiler command or a profiler.

Prepare the matching SDK with the platform's `dist.ps1`/`dist.sh`, then build:

```sh
dotnet build apps/diagnostics/Workload/Workload.fpasproj -c Release -warnaserror
dotnet build apps/diagnostics/Measure/Measure.fpasproj -c Release -warnaserror
dotnet tool restore
```

Repeat with `-c Debug` for diagnostic builds. `Measure` references the existing
Fpas.Processes project for Windows job/Linux process-group ownership; Workload
has only framework/Core dependencies and can also be published with Native AOT.

## Workload

```sh
dotnet apps/diagnostics/Workload/bin/Release/net10.0/Workload.dll --help
dotnet apps/diagnostics/Workload/bin/Release/net10.0/Workload.dll cpu 8000
dotnet apps/diagnostics/Workload/bin/Release/net10.0/Workload.dll alloc 8000
dotnet apps/diagnostics/Workload/bin/Release/net10.0/Workload.dll bench cpu 10000 2 7 .temp-data/cpu.csv cpu-Release local
```

`cpu` repeatedly sums squares from 0 through 32767; one batch must produce
11727587164160. `alloc` creates 512 byte arrays of 8192 bytes, retaining at most
32 at once; the sum of the last byte of each array must be 62795. Each batch is
checked. The allocation workload includes queue growth and GC work; it does not
measure raw array allocation in isolation. Both modes accept a bounded duration
in milliseconds. Fixed-loop `bench` accepts rounds, warmup, sample count, a fresh
CSV filename, label and `local`/`cloud` environment. Timed samples exclude their
result check and report the computed checksum.

`noop` writes `verified:noop` and exits. `fault READY RELEASE` captures a real
InvalidOperationException, prints its source stack, writes its PID to READY and
holds the exception until RELEASE exists or 60 seconds elapse. It then exits 23.
This permits controlled dump collection without changing OS crash-report policy.

## Measure

`Measure --help` describes the external process wrapper. Supply an absolute
executable after `--`; every following token is an argument, without a shell.
Choose `startup`, `compiler` or `sdk-build` as the measurement boundary. Each
sample starts a fresh process and includes launch, output capture and owned
process cleanup. Warmups warm the launcher/filesystem, not a later child's JIT.

Children must exit zero. `--contains TEXT` additionally checks stdout after the
timer stops. One stream is limited to 64 KiB; a per-run deadline defaults to
120 seconds. Cancellation/deadline stops and joins owned processes. Failed runs
produce no successful CSV; an existing output file is refused. Reports record
only labels, boundaries, sample times and non-sensitive runtime/settings metadata,
not the target command line or absolute paths.

CSV files contain raw milliseconds, workload rounds, warmup and runtime settings.
Stdout summarizes median, minimum, maximum, arithmetic mean and sample standard
deviation. No performance threshold is imposed.

## Verification

DiagnosticsWorkflowTests builds these actual sources through a fresh SDK package.
It checks independent checksum expectations, sample publication, invalid results,
deadline cleanup, actual EventPipe stack/GC/allocation events, runtime counters
and a heap dump analyzed after the failing process exits. The platform acceptance
procedure is in [diagnostic workflows](../../docs/pascal/tools/diagnostic-workflows.md#acceptance).
