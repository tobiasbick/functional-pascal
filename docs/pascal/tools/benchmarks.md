# Reproducible CLR measurements

Use the [diagnostic applications](../../../apps/diagnostics/) with fixed inputs.
They verify workload results and keep raw samples; they do not establish a language
performance guarantee. Traced runs belong to [diagnosis](diagnostic-workflows.md),
separate from these unprofiled timing runs.

## Choose the boundary

| Boundary | Timed work | Outside the timer |
| --- | --- | --- |
| `execution-cpu` / `execution-alloc` | Fixed batches in one already running process | Startup, warmups, result checking, CSV/statistics |
| `startup` | Fresh process, runtime/entry point, `noop`, output capture and owned cleanup | Parent startup, argument setup, CSV/statistics |
| `compiler` | Fresh compiler process and explicit source-to-output compilation | Source/reference preparation, parent startup, result checking |
| `sdk-build` | Fresh dotnet/MSBuild process and its selected build operation | Package restore, parent startup, result checking |

Compiler/SDK values are end-to-end process measurements, not isolated binder or
emitter costs. Startup includes the launcher's ownership/capture overhead and the
small `noop` output. Do not subtract unrelated averages to infer internal phases.

## Prepare

Use the [development workflow](../../build/development.md) to build the toolchain
and matching SDK. Build both applications before measuring; use the same source
revision, SDK/runtime and dependencies for a comparison:

```sh
dotnet build apps/diagnostics/Workload/Workload.fpasproj -c Release -warnaserror
dotnet build apps/diagnostics/Measure/Measure.fpasproj -c Release -warnaserror
```

Repeat for Debug. Complete restore beforehand. Set up a fresh output directory;
the applications refuse existing CSV filenames. Stop concurrent builds, tests and
profilers before collecting timing samples. Windows and WSL share host resources:
measure them sequentially and record their environments separately.

PowerShell 7, from the repository root:

```powershell
$run = Join-Path '.temp-data/diagnostics' ('measure-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $run | Out-Null
$dotnet = (Get-Command dotnet).Source
$workload = (Resolve-Path apps/diagnostics/Workload/bin/Release/net10.0/Workload.dll).Path
$measure = (Resolve-Path apps/diagnostics/Measure/bin/Release/net10.0/Measure.dll).Path
$compiler = (Resolve-Path src/Fpas.Cli/bin/Release/net10.0/Fpas.Cli.dll).Path
$pack = Get-ChildItem (Join-Path (Split-Path $dotnet) 'packs/Microsoft.NETCore.App.Ref') -Directory |
  Where-Object { Test-Path (Join-Path $_.FullName 'ref/net10.0') } |
  Sort-Object { [version]$_.Name } -Descending | Select-Object -First 1
$refs = Join-Path $pack.FullName 'ref/net10.0'
```

Bash, from the repository root, with the selected .NET SDK on PATH:

```sh
mkdir -p .temp-data/diagnostics
run=$(mktemp -d "$PWD/.temp-data/diagnostics/measure.XXXXXX")
dotnet=$(readlink -f "$(command -v dotnet)")
workload="$PWD/apps/diagnostics/Workload/bin/Release/net10.0/Workload.dll"
measure="$PWD/apps/diagnostics/Measure/bin/Release/net10.0/Measure.dll"
compiler="$PWD/src/Fpas.Cli/bin/Release/net10.0/Fpas.Cli.dll"
refs=$(rg --files "$(dirname "$dotnet")/packs/Microsoft.NETCore.App.Ref" -g System.Runtime.dll | sort -V | tail -n 1)
refs=$(dirname "$refs")
```

## Collect unprofiled samples

The following commands work in both shells with the variables above. For Debug,
select the Debug DLLs and labels explicitly. Use 2 discarded warmups and 7 retained
samples as a starting protocol; keep the same counts in a comparison.

```sh
dotnet "$workload" bench cpu 10000 2 7 "$run/execution-cpu.csv" cpu-Release local
dotnet "$workload" bench alloc 512 2 7 "$run/execution-alloc.csv" alloc-Release local
dotnet "$measure" --output "$run/startup.csv" --label jit-Release --boundary startup --contains verified:noop -- "$dotnet" "$workload" noop
dotnet "$measure" --output "$run/compiler.csv" --label compiler-Release --boundary compiler --contains Wrote -- "$dotnet" "$compiler" compile examples/hello/main.fpas --reference-dir "$refs" --output "$run/Compiled.dll" --debug portable --optimize
dotnet "$measure" --output "$run/sdk-incremental.csv" --label sdk-incremental-Release --boundary sdk-build -- "$dotnet" build apps/diagnostics/Workload/Workload.fpasproj -c Release --no-restore --disable-build-servers -nr:false -p:UseSharedCompilation=false
dotnet "$measure" --output "$run/sdk-rebuild.csv" --label sdk-rebuild-Release --boundary sdk-build -- "$dotnet" build apps/diagnostics/Workload/Workload.fpasproj -c Release --no-restore --disable-build-servers -nr:false -p:UseSharedCompilation=false -t:Rebuild
dotnet "$workload" cpu 1
```

For Debug compiler samples omit `--optimize`. The no-change SDK case measures
incremental evaluation/checking. Rebuild measures compilation and normal output
publication; it still uses prepared packages and filesystem caches. Keep these
cases separate. The final workload invocation verifies the rebuilt application's
result outside the timing interval.

Every external invocation has a per-run deadline (`--timeout`, default 120 seconds)
and bounded captured output. A failure, timeout, checksum mismatch or failed
`--contains` check invalidates the run; it is not a fast sample. Keep the failure
log and rerun after diagnosis. The application only writes a CSV after all samples
succeed. CPU/alloc benchmarks have bounded rounds/warmups/sample counts; use an
outer job deadline for automation as well.

## JIT, startup and published applications

In-process warmups exercise the same JIT process as its later samples. Fresh
process startup samples always start a new runtime. Discarded startup samples
warm the parent and OS file caches; they do not make the next child's JIT warm.
These procedures do not flush the OS cache and must not be called cold-boot or
cold-disk measurements.

Record the defaults or explicit values of DOTNET_TieredCompilation,
DOTNET_TieredPGO and DOTNET_ReadyToRun. Samples include them in CSV metadata. A
fixed number of warmups does not prove tiering has finished for every application;
retain all samples and investigate drift. Debug measurements verify diagnostic
behavior; use optimized Release for performance conclusions.

To compare Native AOT startup, publish this same Workload through the
[publishing contract](../../build/publishing.md), select its native executable as
Measure's target and pass `noop`. Repeat the in-process `bench` command on that
executable. Build/publish before timing. Keep JIT and AOT outputs/settings distinct;
do not compare a traced JIT run with an unprofiled AOT run. On Windows the native
filename is Workload.exe; on Linux it is Workload.

Prepare the native output before timing. Windows:

```powershell
dotnet publish apps/diagnostics/Workload/Workload.fpasproj -c Release -r win-x64 -p:PublishAot=true --output "$run/aot"
dotnet "$measure" --output "$run/startup-aot.csv" --label aot-Release --boundary startup --contains verified:noop -- "$run/aot/Workload.exe" noop
```

WSL/Linux:

```sh
dotnet publish apps/diagnostics/Workload/Workload.fpasproj -c Release -r linux-x64 -p:PublishAot=true --output "$run/aot"
dotnet "$measure" --output "$run/startup-aot.csv" --label aot-Release --boundary startup --contains verified:noop -- "$run/aot/Workload" noop
```

## Interpret and retain

CSV contains each raw elapsed value in milliseconds, the warmup count, workload
rounds, boundary, label, local/cloud marker, runtime, OS, architecture and tiering
settings. External process cases use rounds 0. Labels identify configuration and
workload; record exact source revision, tool/SDK versions, source/binary/PDB hashes
and invocation in an accompanying local run note. Do not include host/user names,
absolute private paths, secrets or environment dumps in a shareable summary.

The summary gives median, min/max, arithmetic mean and sample standard deviation
(denominator n-1). Compare equivalent inputs and boundaries; retain variation,
not just the best sample. If samples are too close to timer/launch overhead,
increase fixed work or report the measurement limit. If order, drift or contention
changes the conclusion, repeat alternating A/B runs without profiling. There is
no universal percentage gate and no committed performance history.

Cloud/CI measurements describe that runner. Shared CPU, quotas, virtualization,
container memory limits and diagnostic permissions can differ from local machines.
Use the explicit `cloud` marker when applicable; these local Windows/WSL acceptance
runs do not establish a cloud baseline. Preserve raw samples and diagnostic
artifacts under `.temp-data/`, and remove them only after their owning processes
have exited.

## See also

- [Diagnostic collection and dumps](diagnostic-workflows.md)
- [Profiling overview](profiling.md)
- [Diagnostic application contracts](../../../apps/diagnostics/README.md)
