# CLR documentation gaps

| ID | Open follow-up | Reference |
| --- | --- | --- |
| D1 | Reproducible CLR benchmarks | [Profiling](../pascal/tools/profiling.md) |
| D3 | Profiling and dump walkthroughs | [Profiling](../pascal/tools/profiling.md) |

## D1: Reproducible benchmarks

Document and verify a repeatable procedure that:

- Separates program execution, host startup, compilation and SDK builds.
- Uses fixed inputs, deterministic result checks and bounded execution.
- Defines setup, warmup, sampling, variation and cleanup with selected .NET tools.
- Distinguishes cold startup from warmed JIT execution and records relevant
  build options and environment limits, including cloud execution.
- Stores sources, traces, samples and reports under `.temp-data/` with
  non-sensitive metadata and links to workload sources.

Accept the documented commands and workloads on Windows and Linux from a clean
prepared environment. No separate FPAS benchmark command is selected.

## D3: Profiling and dump walkthroughs

Document and verify CPU stacks, allocation/GC observations and controlled failure
dumps with selected .NET tools. Keep profiled and unprofiled measurements separate.
Include tool installation, platform limits, output locations and cleanup.
Accept each procedure on Windows and Linux.

## See also

- [Documentation hub](../README.md)
- [CLI gaps](cli-gap.md)
- [Verification coverage](../../tests/coverage.md)
- [Development workflow](../build/development.md)
