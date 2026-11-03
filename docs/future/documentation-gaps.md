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
| D2 | Practical .NET API recipes | [Library guides](../pascal/libraries/README.md) | Open |
| D3 | Concrete debugging/profiling walkthroughs | [Debugging](../pascal/tools/debugging.md), [profiling](../pascal/tools/profiling.md) | Open |
| D4 | SDK use outside the checkout | [Development workflow](../build/development.md), [projects](../pascal/program-structure/projects.md) | Open |

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

## D2: Practical .NET API recipes

Existing guides cover collections, text/regex, files/streams, and JSON. An async
HTTP example exists under concurrency. Missing task-oriented guidance includes:

- Process creation, arguments, environment, exit status, bounded output capture,
  timeout/cancellation, and process/resource lifetime.
- Environment access, date/time and duration handling, numeric utilities, and
  random values through supported .NET APIs.
- HTTP request/response lifetime, cancellation, and supported stream/text APIs;
  further socket/server examples only where actual signatures and scope permit.
- Synchronization, Task coordination, and channels with explicit completion,
  fault/cancellation handling, and shared-state rules.

Write short recipes with explicit imports, actual CLR signatures, and meaningful
execution assertions. Prefer representative workflows over a copied API catalog.
Link full .NET references for overloads; select supported signatures rather than
claiming Span/ref-like or other excluded APIs work.

Close each recipe when its complete source is checked by the Markdown runner or
a linked SDK test, with controlled inputs and cleanup. Record a newly encountered
compiler limitation separately. Do not introduce a parallel Std wrapper library.

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

## D4: SDK use outside the checkout

The local bootstrap and examples inherit the repository's NuGet.Config. The
project/build contract describes dependency resolution and output files, but an
independent consumer still needs a complete walkthrough.

Document a consumer outside the source tree that:

- Resolves an explicitly versioned local Fpas.NET.Sdk package through an explicit
  NuGet source/configuration. Does not depend on an accidental prior cache entry.
- Builds an executable and a library with public units, plus an ordinary C#
  consumer or dependency through ProjectReference and a mixed solution.
- Runs with the needed .NET runtime, Core support assembly, and non-framework
  dependencies. Identifies the files to copy without requiring source/toolchain
  directories at runtime.
- Explains package-version/cache updates after compiler changes and the current
  net10.0/AnyCPU and symbol/documentation-output boundaries.

Close D4 after an isolated consumer with its own package configuration can restore,
build, and execute the examples using the packaged toolchain. Public package
publication, dotnet publish, AOT, signing, native apphosts, and IDE loaders remain
separate feature scope. This gap does not authorize implementing them.

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
