# Profiling CLR programs

Investigate the actual generated program with a suitable .NET profiler or
diagnostic tool before writing external samplers or benchmark variants.

The [diagnostic CLI](cli-diagnostics.md) provides owned collection, bounded
EventPipe decoding, per-thread sampled stacks, counters and dump workflows.

Use [reproducible measurements](benchmarks.md) for fixed workloads, startup,
compilation and SDK timing. Use [diagnostic workflows](diagnostic-workflows.md)
for pinned tools, actual stack/GC recordings and controlled failure dumps.
The [FPAS applications](../../../apps/diagnostics/) supply verified workloads
and an unprofiled process timer; they are separate from compiler commands.

## Select the observation

| Question | Suitable .NET observation |
| --- | --- |
| Which routines consume CPU? | Sampling/tracing with stack and symbol resolution |
| Is the program allocating or collecting heavily? | Allocation and GC counters/traces |
| Is work blocked or awaiting? | Thread/task stacks and async-aware tracing |
| Why did the process fail? | Exception stacks or a controlled dump/debugger run |

The local tool manifest pins dotnet-trace, dotnet-counters and dotnet-dump.
Restore it before the [diagnostic procedure](diagnostic-workflows.md#prepare-tools-and-artifacts).
Preserve assembly/PDB/source identity when interpreting stacks.
FPAS free routine, closure, and async names follow their CLR representation.

## Controlled measurements

Choose a bounded workload and verify its result before comparing observations.
Record the build configuration, workload, and relevant environment. Separate
startup/compilation from program execution when answering a runtime question.
Keep measurement outputs under .temp-data. A measurement describes that run;
it does not create a language performance or scheduling guarantee.

Debuggers and profilers can affect timing and scheduling. Use appropriate
unprofiled runs for a controlled before/after comparison, with equivalent inputs
and configuration. Shared-state correctness still follows ordinary .NET rules.

## See also

- [Reproducible measurements](benchmarks.md)
- [Trace, counter and dump walkthroughs](diagnostic-workflows.md)
- [Debugging](debugging.md)
- [Compiled programs](../program-structure/compiled-programs.md)
- [Development configurations](../../build/development.md)
- [.NET diagnostic tools](https://learn.microsoft.com/en-us/dotnet/core/diagnostics/tools-overview)
