# CLR documentation gaps

The handbook, agent reference, architecture and test coverage describe the current
CLR toolchain. IDE debugging is complete and documented in the
[VS Code guide](../pascal/tools/debugging.md).

| ID | Open follow-up | Owner |
| --- | --- | --- |
| D1 | Reproducible CLR benchmarks | [Profiling](../pascal/tools/profiling.md), [CLI inventory](cli-gap.md) |
| D3 | MCP agent debugging, profiling and dump walkthroughs | [MCP plan](debugger/README.md), [profiling](../pascal/tools/profiling.md) |

## D1: Reproducible benchmarks

Document and verify a repeatable procedure that:

- Separates program execution, host startup, compilation and SDK builds.
- Uses fixed inputs, deterministic result checks and bounded execution.
- Defines setup, warmup, sampling, variation and cleanup with selected .NET tools.
- Distinguishes cold startup from warmed JIT execution and records relevant build
  options and environment limits, including cloud execution.
- Stores sources, traces, samples and reports under `.temp-data/` with non-sensitive
  metadata and links to workload sources.

Accept the documented commands and workloads on Windows and Linux from a clean
prepared environment. No separate FPAS benchmark command is selected.

## D3: Concrete debugging/profiling walkthroughs

Remaining guidance covers:

- Agent-host MCP registration, NetCoreDbg installation and a real launch,
  breakpoint, value inspection, step and stop through the
  [MCP tools](debugger/mcp-debug.md), including cancellation and cleanup.
- CPU stacks, allocation/GC observations and controlled failure dumps with
  selected .NET tools; keep profiled and unprofiled measurements separate.
- Tool installation, platform limits, output locations and cleanup.

Accept each procedure on its stated platforms. D3 does not reopen completed IDE
debugging. Batch debugging is blocked and is not a documentation requirement.

## See also

- [Documentation hub](../README.md)
- [MCP debugger gaps](debug-gap.md)
- [Verification coverage](../../tests/coverage.md)
- [Development workflow](../build/development.md)
