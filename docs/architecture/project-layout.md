# Repository and project layout

The C# projects and xUnit projects are grouped in one primary `Fpas.slnx` solution.
Compiler and CLI follow the [direct compilation pipeline](compiler.md).
Tests cover diagnostics, emitted metadata, actual CLI/program execution, and
[mixed-project MSBuild builds](../../tests/coverage.md#build-acceptance).
Fpas.Core supplies record/union metadata attributes; the SDK packages Core, the compiler,
and the implemented task.
Examples build and run through the SDK; applications remain later work. The responsibilities
below describe their responsibilities and boundaries.

## Repository tree

```text
Fpas.slnx
global.json
Directory.Build.props
Directory.Packages.props
NuGet.Config
src/
  Fpas.Compiler/
    Fpas.Compiler.csproj
    Formatting/
    Analysis/
    Syntax/
      Parsing/
    Diagnostics/
    Binding/
      Bodies/
      Bound/
      Declarations/
      Flow/
      References/
      Semantics/
      Symbols/
      Types/
    Lowering/
      Async/
    Emission/
      Bodies/
      Debugging/
      Metadata/
      StateMachines/
  Fpas.Cli/
    Fpas.Cli.csproj
    Program.cs
    CliApplication.cs
    Compilation/
    Analysis/
    Debugging/
    Diagnostics/
    Projects/
    Formatting/
  Fpas.Workspaces/
    Analysis/
    Formatting/
    Loading/
    Processes/
    Projects/
    Queries/
    Solutions/
  Fpas.Processes/
  Fpas.Debugging/
    Fpas.Debugging.csproj
    Processes/
    Protocol/
    Sessions/
    Tools/
  Fpas.Build.Tasks/
    Fpas.Build.Tasks.csproj
    Analysis/
    Documentation/
    Diagnostics/
    Incremental/
  Fpas.Core/
    Fpas.Core.csproj
sdk/
  Fpas.NET.Sdk/
    Fpas.NET.Sdk.csproj
    Sdk/
      Sdk.props
      Sdk.targets
  feed/
    README.md
tests/
  Fpas.Compiler.Tests/
    Fpas.Compiler.Tests.csproj
    ControlFlow/
    Diagnostics/
    Metadata/
    Routines/
    Support/
    Types/
  Fpas.IntegrationTests/
    Fpas.IntegrationTests.csproj
    Build/
    Cli/
    Debugging/
    Documentation/
    Diagnostics/
    Execution/
    Processes/
    Workspaces/
    Projects/
    Support/
  Fixtures/
examples/
  README.md
apps/
  README.md
integrations/
  neovim/
    lua/
      fpas_debug.lua
docs/
```

All listed projects exist. Fpas.Core contains the versioned record/union metadata
attributes; further metadata/helpers belong there only when an emitted feature
requires them. Common C#
settings live in Directory.Build.props; Directory.Packages.props pins the shared
xUnit/test dependencies. global.json selects a stable .NET 10 SDK.

The SDK packaging project includes props/targets, compiler/CLI and Core assets,
the task, and README. The [local bootstrap](../build/development.md#build-an-fpas-application)
packs it into sdk/feed before restoring FPAS consumers. No SDK package is publicly
published. The SDK resolves and copies its matching Fpas.Core assembly for records and unions.

## Project responsibilities

| Project | Responsibility |
| --- | --- |
| `Fpas.Compiler` | C# library for source handling, lexing/parsing, name binding, type checking, lowering, and direct CIL/metadata/PDB generation; optional XML API documentation |
| `Fpas.Cli` | Command binding, compiler invocation, diagnostics, MCP debugger startup and exit status |
| `Fpas.Debugging` | Official SDK MCP hosting/tools, thin DAP session client and owned external NetCoreDbg processes |
| `Fpas.Workspaces` | Evaluated MSBuild/solution contexts, immutable input capture, dependency preparation and compiler-backed project analysis |
| `Fpas.Processes` | Shared Linux process-group/subreaper and Windows job ownership for debugger/build processes |
| `Fpas.Build.Tasks` | Thin C# MSBuild adapter for inputs, compiler process invocation, structured diagnostics, and declared outputs |
| `Fpas.NET.Sdk` | MSBuild props/targets and the SDK packaging project, selecting the matching task/compiler/support assets |
| `Fpas.Core` | Small managed support library for emitted FPAS features, as defined by the CLR mapping |
| `Fpas.Compiler.Tests` | C# xUnit unit tests for compiler components and their observable rules |
| `Fpas.IntegrationTests` | C# xUnit tests for the actual compiler executable, generated programs, .NET interop, and complete MSBuild builds |

Use folders and namespaces within Fpas.Compiler for Syntax, Binding, Lowering,
and Emission. Split a compiler component into another project when independent
reuse or a distinct dependency boundary warrants it. CLI and MSBuild code share
the same compiler implementation through the compiler executable/library.
The emitted program targets the CLR directly; C# is the implementation language,
not an intermediate source-generation stage. See [CLR mapping](../specs/clr-mapping.md).

Within each phase, directories and namespaces identify the owning concern.
Syntax owns the syntax model; Syntax.Parsing owns parser implementations.
Binding separates metadata references, declarations, body binding, types,
symbols, flow analysis, and typed bound operations. Bound-tree traversal belongs
with that shared representation, so binding does not depend on lowering.
Lowering.Async produces resumable plans. Emission owns their association with
generated machines, field layouts, CIL bodies, metadata, and portable symbols.
Keep partial implementations of one component together in their topic directory.

CLI compilation/publication code lives in Compilation. Build-task documentation
outputs and incremental fingerprints have their own topic directories. Public
compiler APIs, Core metadata attributes, and the MSBuild task retain the CLR
identities required by their owning contracts.

## Dependencies and solution

Fpas.slnx groups the C# implementation projects, SDK packaging, and xUnit projects.
ProjectReference edges define their build dependencies; solution folders may group
src, sdk, and tests without introducing separate solutions for each component.

- Fpas.Cli references Fpas.Compiler, Fpas.Workspaces and Fpas.Debugging as C# libraries.
- Fpas.Workspaces references the compiler, process ownership and official
  SolutionPersistence library; MSBuild remains in owned SDK processes.
- Fpas.Debugging uses the official MCP SDK and hosting packages; it has no
  dependency on Fpas.Compiler or Fpas.Core. NetCoreDbg is installed separately.
- Fpas.Build.Tasks launches the matching Fpas.Cli in a separate process. Compiler
  dependencies stay outside the MSBuild host process. Resolve the compiler from
  the selected SDK's assets, with normal .NET host invocation where needed.
- Fpas.NET.Sdk delivers the task and compiler together and resolves the matching
  Fpas.Core reference/runtime asset when required. Build/package ordering must
  account for those inputs without creating a dependency cycle.
- Generated FPAS assemblies reference Fpas.Core only for the language support
  they need. Fpas.Core has no dependency on compiler or MSBuild implementation.
- Tests reference the component under test or invoke the actual built toolchain.
  Compiler components do not depend on tests, examples, or applications.

Formatting owns the token-preserving, parser-driven layout engine in
Fpas.Compiler. Its optional syntax layout facts are collected only for formatting;
normal compilation does not build a layout. Fpas.Workspaces.Formatting evaluates
source selection without analysis preparation, and Fpas.Cli.Formatting owns
options, reports and stale-checked source publication.

Compiler logic and language diagnostics belong in Fpas.Compiler. The CLI handles
its invocation interface; the task translates structured results into MSBuild
diagnostics. Neither adapter implements a second binder, emitter, or error model.

The solution used in the MSBuild tests is a mixed-language consumer fixture,
distinct from the repository's Fpas.slnx. FPAS consumer projects require the built
SDK to be resolvable before restore/build. Establish that bootstrap before adding
selected consumer projects to the main solution.

## Tests and fixtures

The [Neovim integration](../../integrations/neovim/README.md) is configuration for
an existing external DAP client. It is separate from the C# toolchain and emitted
programs; no editor/debugger binary or DAP proxy is packaged with FPAS.

Keep C# xUnit tests under tests, separate from production source. Unit tests cover
focused behavior such as tokenization, parsing, binding, type diagnostics, and
lowering. Add tests for a support library/task separately when their own behavior
warrants it; avoid empty test projects solely to mirror the source tree.

Compiler tests group types, routines, control flow, and metadata by topic.
Integration tests separate CLI behavior, generated-program execution,
documentation, and SDK builds. Shared helpers live in Support with matching
namespaces; tests do not call unrelated test classes for compilation setup.

Integration tests must compile real FPAS, inspect emitted signatures and
diagnostics, and execute resulting programs as appropriate. Cover FPAS-to-C# and
C#-to-FPAS references and builds through the actual SDK/MSBuild task. Use bounded
processes and deterministic assertions for command output, exit status, and files.
The test harness is xUnit; fixture FPAS sources are inputs to that harness.

tests/Fixtures contains the required FPAS/C# sources, project files, and solutions,
organized by scenario. Isolate generated outputs in test work directories and
keep fixtures unchanged during tests. Planned implementation acceptance cases in
the language topics and MSBuild plan guide coverage. Implemented xUnit cases cover
the compiler and local SDK, including project/package reference execution,
incremental builds, and Clean/Rebuild. Test build targets obtain actual reference
paths; the checked-in Hello source is copied into the integration-test output.

## Examples and applications

Both root directories follow the earlier FPAS repository's distinction:

- `examples/` contains small programs demonstrating a language feature, .NET API,
  or focused technique. Group them by topic. A standalone example may have its
  own `.fpasproj`, supporting units, and a short README. Interactive demos are
  allowed when clearly identified.
- `apps/` contains complete end-user applications written in FPAS. An application
  owns a directory with its README, source, `.fpasproj` project or projects,
  assets, and application-specific material. It may grow across multiple units
  and projects as its features require.

Both are consumers of the same compiler and .NET SDK contract. They use the new
language syntax and `.fpasproj` format. Test assertions and regression fixtures
belong under tests, including application workflow coverage driven by xUnit.
Automated runs select bounded noninteractive scenarios explicitly; interactive
examples/apps can be compiled without launching their UI.

Complete [Markdown programs](../pascal/examples/README.md) and the compact
[agent reference](../pascal/agents/README.md) are compiled directly from their
documentation and executed by integration tests. Companion units use explicit
project labels. Hello builds through its `.fpasproj` or the direct compiler.
The [Units example](../../examples/units/README.md) compiles a program and two
units into one assembly. The apps directory contains its README placeholder.

## Extending the implementation

Preserve compile/execute integration coverage when extending the compiler or SDK.
Add Fpas.Core contents only when emitted features need them. Follow the owning
language/build contracts and their tracked gaps. Add executable examples/apps
using implemented features. Publishing and IDE/LSP/query tooling retain their
separately documented scope. Repository commands belong in the
[development workflow](../build/development.md).
