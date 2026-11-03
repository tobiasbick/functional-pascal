---
name: fpas-bench
description: Investigate CLR FPAS program, compiler, or SDK performance with supported .NET tools and controlled before/after observations.
---

# FPAS CLR performance

Use the [profiling guide](../../../docs/pascal/tools/profiling.md),
[debugging guide](../../../docs/pascal/tools/debugging.md), and
[development workflow](../../../docs/build/development.md). The current toolchain
has no repository benchmark command or curated CLR benchmark suite. Its full
measurement workflow remains a [documentation gap](../../../docs/future/documentation-gaps.md#d1-reproducible-benchmarks).
Do not invent commands, report schemas, or completed benchmark coverage.

## Investigate the requested cost

1. Identify whether the question concerns program execution, compilation, SDK
   builds, startup, allocation/GC, or blocking work. Choose a bounded existing
   workload with verified results. Use [fpas-authoring](../fpas-authoring/SKILL.md)
   and [fpas-projects](../fpas-projects/SKILL.md) when source/project changes are
   part of the authorized task.
2. Inspect the actual generated program with supported .NET tooling before
   writing samplers or variants. Determine which tools and options are installed;
   the guide names dotnet-trace, dotnet-counters, and dotnet-dump as possible tools,
   not bundled compiler commands.
3. Build the measured configuration before collecting samples. For program
   execution, keep compilation outside the timer. Treat startup and warmed JIT
   execution as separate questions, and record the chosen boundary and warmup.
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
