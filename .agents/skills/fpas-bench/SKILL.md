---
name: fpas-bench
description: Investigate CLR FPAS program, compiler, or SDK performance with supported .NET tools and controlled before/after observations.
---

# FPAS CLR performance

Use [unprofiled measurements](../../../docs/pascal/tools/benchmarks.md) for
execution, startup, compiler-process and SDK timing. Use
[`fpas profile`](../../../docs/pascal/tools/cli-diagnostics.md) for owned trace
and counter collection and structured summaries. Use the
[manual .NET workflows](../../../docs/pascal/tools/diagnostic-workflows.md) for
direct tool invocation and platform acceptance. The
[FPAS diagnostic applications](../../../apps/diagnostics/) supply verified
workloads and a bounded process timer.

## Investigate the requested cost

1. Identify whether the question concerns program execution, compilation, SDK
   builds, startup, allocation/GC, or blocking work. Choose a bounded existing
   workload with verified results. Use [fpas-authoring](../fpas-authoring/SKILL.md)
   and [fpas-projects](../fpas-projects/SKILL.md) when source/project changes are
   part of the authorized task.
2. Inspect the actual generated program with supported .NET tooling before
   writing samplers or variants. Check the available tool versions and options.
   Follow the setup for the chosen path. `fpas profile setup` uses its own tool
   directory. The manual workflow uses the repository's `dotnet tool restore`.
3. Build the measured configuration using the
   [development workflow](../../../docs/build/development.md). Finish other
   builds, tests, and profilers before timing. Measure Windows and WSL separately.
   Keep compilation outside program execution samples. Record the boundary and
   warmup for startup or warmed JIT execution.
4. Compare the same workload, arguments, build/runtime configuration, and result
   checks. Use repeated unprofiled samples for timing comparisons; retain sample
   counts and variation. Investigate noisy or conflicting results before making
   performance claims. Keep correctness verification independent of timing.

## Reports and environment

Store scratch sources, traces, samples, and comparisons under `.temp-data/`.
Record relevant non-sensitive configuration and tool/runtime versions. Inspect
reports for secrets, personal data, and machine identifiers before sharing them.
Do not create committed benchmark history as part of a measurement run.

Identify whether the run is local or cloud-hosted. Cloud results describe that
environment and do not establish a controlled local baseline. When the user
requests a cloud comparison, report its observations and limitations within that
scope; do not substitute unrelated local measurements.

Apply [repository policy](../../../AGENTS.md) and
[contribution checks](../../../CONTRIBUTING.md) before committing any authorized
implementation change. A request to investigate performance does not authorize
inventing new language behavior or changing FPAS syntax.
