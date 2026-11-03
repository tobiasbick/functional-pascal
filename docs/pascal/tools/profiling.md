# Profiling CLR programs

Investigate the actual generated program with a suitable .NET profiler or
diagnostic tool before writing external samplers or benchmark variants.

## Select the observation

| Question | Suitable .NET observation |
| --- | --- |
| Which routines consume CPU? | Sampling/tracing with stack and symbol resolution |
| Is the program allocating or collecting heavily? | Allocation and GC counters/traces |
| Is work blocked or awaiting? | Thread/task stacks and async-aware tracing |
| Why did the process fail? | Exception stacks or a controlled dump/debugger run |

Tools such as dotnet-trace, dotnet-counters, and dotnet-dump may be installed
for the target environment; their availability and options belong to their .NET
tool versions. Preserve assembly/PDB/source identity when interpreting stacks.
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

- [Debugging](debugging.md)
- [Compiled programs](../program-structure/compiled-programs.md)
- [Development configurations](../../build/development.md)
- [.NET diagnostic tools](https://learn.microsoft.com/en-us/dotnet/core/diagnostics/tools-overview)
