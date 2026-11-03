# CLR documentation gaps

These are documentation follow-ups for the current CLR toolchain. They do not
declare missing language/runtime features implemented or authorize extensions.
Required compiler work remains in [language gaps](language-gaps.md); demonstrated
maintenance blockers belong in [dogfooding gaps](dogfooding-gaps.md).

The handbook, agent reference, architecture, contribution guide, and test coverage
already have CLR owners. Remaining work concerns practical guidance and verified
workflows, rather than reproducing Rust-specific manuals or implementation history.

| ID | Follow-up | Current documentation | Status |
| --- | --- | --- | --- |
| D1 | Reproducible CLR benchmarks | [Profiling overview](../pascal/tools/profiling.md) | Open |
| D3 | Concrete debugging/profiling walkthroughs | [Debugging](../pascal/tools/debugging.md), [profiling](../pascal/tools/profiling.md) | Open |

## D1: Reproducible benchmarks

The profiling guide explains measurement boundaries and available .NET tool
families. There is no curated CLR suite, fixed workload catalog, or complete
before/after command workflow. The performance skill provides basic guidance;
it is not an implemented benchmark runner.

Document and verify a repeatable procedure that:

- Distinguishes FPAS program execution, host startup, compilation, and SDK builds.
- Uses fixed inputs and workload arguments with deterministic result checks and
  bounded execution. Explains when compilation, setup, warmup, and cleanup occur.
- Separates cold startup from warmed JIT execution, including relevant runtime
  and build options. Defines the sampling/summary method and retains variation.
- Supports comparable before/after observations and states relevant environment
  limitations, including cloud execution. Does not mix unrelated host baselines.
- Stores sources, traces, raw samples, and reports under `.temp-data/`, with
  non-sensitive metadata and clear links to workload sources.

Close D1 when the documented commands and actual selected workloads can be run
from a clean prepared environment and the result checks pass. If additional tools
are needed, identify them explicitly; do not document an imaginary CLI command.
Historical timing reports and the Rust VM harness are not CLR documentation.

## D3: Concrete debugging/profiling walkthroughs

Current documentation describes portable symbols, source identity, generated
methods, and possible .NET tools. It does not yet provide complete tested steps
for a user to inspect a synchronous or asynchronous FPAS program.

Provide bounded walkthroughs for:

- Building matching assembly/PDB/source inputs and selecting a compatible .NET
  debugger. Set a source breakpoint, inspect parameters/locals, and step.
- Inspecting captures and an async continuation after a real suspension. Explain
  supported generated storage without promising an FPAS-specific IDE front end.
- Capturing CPU stacks, allocation/GC observations, and a controlled failure dump
  with selected installed .NET tools. Keep profiled and unprofiled timing separate.
- Tool installation/version requirements, relevant platform restrictions, output
  locations, and cleanup. Avoid depending on a public network service or UI in
  ordinary automated example checks.

Close D3 after the exact documented procedure has been exercised with the stated
tool/platform configuration. Record practical limits. The Rust DAP/JSONL debugger,
tracepoints, seeded scheduler, and native failure-report commands are not CLR
features and must not be copied as working instructions.

## Scope and maintenance

Keep these entries open until their procedures and examples are verified. Link
completed guidance from the owning handbook/build/tool article, then remove or
shorten the corresponding gap entry. Do not accumulate completion history here.

The Rust documentation for register execution, bytecode images, VM collection,
native built-in catalogs, and already absent tooling describes other
implementations. It does not create a missing CLR handbook chapter. Finer Rust
chapter splits alone are not gaps where CLR rules already have an owner.

## See also

- [Documentation hub](../README.md)
- [Language scope](../pascal/getting-started/overview.md)
- [Verification coverage](../../tests/coverage.md)
- [Development workflow](../build/development.md)
