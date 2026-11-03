# Repository and project layout

**Status: initial direct compiler and MSBuild integration implemented.**
The C# projects and xUnit projects are grouped in one primary `Fpas.slnx` solution.
Compiler and CLI implement the [compiler subset](../implementation/first-program.md).
Tests cover diagnostics, emitted metadata, actual CLI/program execution, and
[mixed-project MSBuild builds](../implementation/msbuild.md).
Fpas.Core supplies record/union metadata attributes; the SDK packages Core, the compiler,
and the implemented task.
Examples build and run through the SDK; applications remain later work. The responsibilities
below describe the full intended implementation.

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
    Syntax/
    Binding/
    Lowering/
    Emission/
  Fpas.Cli/
    Fpas.Cli.csproj
    Program.cs
  Fpas.Build.Tasks/
    Fpas.Build.Tasks.csproj
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
  Fpas.IntegrationTests/
    Fpas.IntegrationTests.csproj
  Fixtures/
examples/
  README.md
apps/
  README.md
docs/
```

All listed projects exist. Fpas.Core contains the versioned record/union metadata
attributes; further metadata/helpers belong there only when an emitted feature
requires them. Common C#
settings live in Directory.Build.props; Directory.Packages.props pins the shared
xUnit/test dependencies. global.json selects a stable .NET 10 SDK.

The SDK packaging project includes props/targets, compiler/CLI and Core assets,
the task, and README. The [local bootstrap](../implementation/msbuild.md#local-sdk-bootstrap)
packs it into sdk/feed before restoring FPAS consumers. No SDK package is publicly
published. The SDK resolves and copies its matching Fpas.Core assembly for records and unions.

## Project responsibilities

| Project | Responsibility |
| --- | --- |
| `Fpas.Compiler` | C# library for source handling, lexing/parsing, name binding, type checking, lowering, and direct CIL/metadata/PDB generation; optional XML API documentation |
| `Fpas.Cli` | C# executable for arguments, compiler invocation, diagnostics, and exit status |
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

## Dependencies and solution

Fpas.slnx groups the C# implementation projects, SDK packaging, and xUnit projects.
ProjectReference edges define their build dependencies; solution folders may group
src, sdk, and tests without introducing separate solutions for each component.

- Fpas.Cli references Fpas.Compiler as a C# library.
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

Compiler logic and language diagnostics belong in Fpas.Compiler. The CLI handles
its invocation interface; the task translates structured results into MSBuild
diagnostics. Neither adapter implements a second binder, emitter, or error model.

The Game.slnx used in the MSBuild plan is a mixed-language consumer fixture,
distinct from the repository's Fpas.slnx. FPAS consumer projects require the built
SDK to be resolvable before restore/build. Establish that bootstrap before adding
selected consumer projects to the main solution.

## Tests and fixtures

Keep C# xUnit tests under tests, separate from production source. Unit tests cover
focused behavior such as tokenization, parsing, binding, type diagnostics, and
lowering. Add tests for a support library/task separately when their own behavior
warrants it; avoid empty test projects solely to mirror the source tree.

Integration tests must compile real FPAS, inspect emitted signatures and
diagnostics, and execute resulting programs as appropriate. Cover FPAS-to-C# and
C#-to-FPAS references and builds through the actual SDK/MSBuild task. Use bounded
processes and deterministic assertions for command output, exit status, and files.
The test harness is xUnit; fixture FPAS sources are inputs to that harness.

tests/Fixtures contains the required FPAS/C# sources, project files, and solutions,
organized by scenario. Isolate generated outputs in test work directories and
keep fixtures unchanged during tests. Planned implementation acceptance cases in
the language topics and MSBuild plan guide coverage. Implemented xUnit cases cover
the compiler subset and local SDK, including project/package reference execution,
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

The existing [Markdown programs](../pascal/examples.md) remain design examples.
Most source extraction, further examples, application selection, and migration of
old FPAS applications are later work. Hello builds through its `.fpasproj` or
the direct compiler. The [Units example](../../examples/units/README.md) compiles
a program and two units into one assembly. The apps directory still contains only its README
placeholder.

## Starting implementation

Extend the compiler library and CLI beyond the first-program subset, preserving
compile/execute integration coverage. Add Fpas.Core contents when emitted features
need them, and extend the implemented task/SDK and mixed-project coverage toward
the MSBuild plan. Add executable examples and apps once the required language and
build features work. Publishing and IDE/LSP/query tooling retain their separately
documented scope.
