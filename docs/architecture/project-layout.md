# Repository and project layout

The C# projects and xUnit projects are grouped in one primary `Fpas.slnx` solution.
Compiler and CLI follow the [direct compilation pipeline](compiler.md).
Tests cover diagnostics, emitted metadata, actual CLI/program execution, and
[mixed-project MSBuild builds](../../tests/coverage.md#build-acceptance).
Fpas.Core supplies record/variant metadata attributes; the SDK packages Core, the compiler,
and the implemented task.
Fpas.Deployment owns file-based access to embedded distribution assets, their
content-addressed cache beside the executable, and shared CMS delivery signatures.
The CLI's Distribution directory owns bundling and offline SDK export;
the [distribution reference](../build/distribution.md) owns deployment.
Examples and applications build and run through the SDK. The responsibilities
below describe their boundaries.

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
    Testing/
    Documentation/
    Refactoring/
    Toolchain/
    LanguageServer/
    Distribution/
    Profiling/
    Terminal/
    Reporting/
  Fpas.LanguageServer/
    Hosting/
    Protocol/
    Transport/
  Fpas.Workspaces/
    Analysis/
    Editing/
    Refactoring/
    Formatting/
    Snapshots/
    Live/
    Loading/
    Processes/
    Projects/
    Queries/
    Solutions/
    Testing/
    Documentation/
  Fpas.Processes/
    Terminal/
  Fpas.Deployment/
  Fpas.SdkResolver/
  Fpas.Debugging/
    Fpas.Debugging.csproj
    Core/
    Native/
    Symbols/
    Values/
    Evaluation/
    Hosting/
    Batch/
    Failures/
    Dap/
  Fpas.Build.Tasks/
    Fpas.Build.Tasks.csproj
    Analysis/
    Documentation/
    Diagnostics/
    Incremental/
  Fpas.Core/
    Fpas.Core.csproj
  Fpas.Ui.Gui.Cui/
    Build/
    Interop/
    Text/
    native/
  Fpas.Ui.Gui/
    Application.fpas
    Internal/
  Fpas.Ui.Mvu/
  Fpas.Ui/
  Fpas.Ui.Tui/
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
    Testing/
    ApiDocumentation/
    Editing/
    Refactoring/
    Toolchain/
  Fpas.Ui.Gui.Cui.Tests/
  Fixtures/
examples/
  README.md
  basics/
  types/
  functions/
  interop/
  io/
  numerics/
  ui/
    gui/
    tui/
apps/
  README.md
third-party/
  cui/
docs/
```

All listed projects exist. Fpas.Core contains the versioned record/variant metadata
attributes; further metadata/helpers belong there only when an emitted feature
requires them. Common C#
settings live in Directory.Build.props; Directory.Packages.props pins the shared
xUnit/test dependencies. global.json selects a stable .NET 10 SDK.

The SDK packaging project includes props/targets, compiler/CLI and Core assets,
the task, and README. The [local bootstrap](../build/development.md#build-an-fpas-application)
packs it into sdk/feed before restoring FPAS consumers. No SDK package is publicly
published. The SDK resolves and copies its matching Fpas.Core assembly for records and variants.

## Project responsibilities

| Project | Responsibility |
| --- | --- |
| `Fpas.Compiler` | C# library for source handling, lexing/parsing, name binding, type checking, lowering, and direct CIL/metadata/PDB generation; optional XML API documentation |
| `Fpas.Cli` | Command binding, compiler invocation, diagnostics, debugger startup and exit status |
| `Fpas.Debugging` | Shared native CLR debugger core, supervised workers, passive inspection, batch/failure reports and DAP server |
| `Fpas.Workspaces` | Evaluated MSBuild/solution contexts, explicit preparation policy, immutable text snapshots, checked edits and compiler-backed analysis/refactoring services |
| `Fpas.LanguageServer` | Stdio LSP lifecycle, bounded protocol transport, document synchronization and diagnostics over the shared live workspace |
| `Fpas.Processes` | Linux process-group/subreaper, Windows jobs and private native PTY/ConPTY ownership |
| `Fpas.Build.Tasks` | Thin C# MSBuild adapter for inputs, compiler process invocation, structured diagnostics, and declared outputs |
| `Fpas.NET.Sdk` | MSBuild props/targets and the SDK packaging project, selecting the matching task/compiler/support assets |
| `Fpas.Core` | Small managed support library for emitted FPAS features, as defined by the CLR mapping |
| `Fpas.Ui.Gui.Cui` | Low-level CUI binding, UI-thread native lifetime, UTF-8 editing buffers and native build integration |
| `Fpas.Ui.Gui` | FPAS owner loop adapting shared MVU transitions and validated widget trees to CUI |
| `Fpas.Ui.Mvu` | FPAS model/mailbox runtime and background-effect lifetime shared by UI hosts |
| `Fpas.Ui` | Host-independent FPAS widget data, cached subviews and validation profiles |
| `Fpas.Ui.Tui` | FPAS terminal owner loop, shared-widget adapter, native cell layout and rendering; depends on MVU and Ui without CUI |
| `Fpas.Compiler.Tests` | C# xUnit unit tests for compiler components and their observable rules |
| `Fpas.IntegrationTests` | C# xUnit tests for the actual compiler executable, generated programs, .NET interop, and complete MSBuild builds |
| `Fpas.Ui.Gui.Cui.Tests` | Native GUI/interop tests and the packaged-SDK FPAS GUI fixture |

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

- Fpas.Cli references Fpas.Compiler, Fpas.Workspaces, Fpas.LanguageServer and Fpas.Debugging as C# libraries.
- Fpas.LanguageServer references Fpas.Workspaces, MIT-licensed OmniSharp protocol
  types and StreamJsonRpc. Protocol dependencies stay outside the workspace and
  compiler libraries.
- Fpas.Workspaces references the compiler, process ownership and official
  SolutionPersistence library; MSBuild remains in owned SDK processes.
  Fpas.SdkResolver runs inside those processes and resolves package SDKs from the
  installed cache without downloads. Its MSBuild framework reference is supplied
  by that host and is not shipped with the CLI.
- Fpas.Debugging references process ownership and packaged DbgShim assets; it has
  no dependency on Fpas.Compiler or Fpas.Core. Native interfaces, symbols, values,
  evaluation, supervision and frontends have separate topic directories.
- Fpas.Build.Tasks launches the matching Fpas.Cli in a separate process. Compiler
  dependencies stay outside the MSBuild host process. Resolve the compiler from
  the selected SDK's assets, with normal .NET host invocation where needed.
- Fpas.NET.Sdk delivers the task and compiler together and resolves the matching
  Fpas.Core reference/runtime asset when required. Build/package ordering must
  account for those inputs without creating a dependency cycle.
- Generated FPAS assemblies reference Fpas.Core only for the language support
  they need. Fpas.Core has no dependency on compiler or MSBuild implementation.
- Fpas.Ui.Gui.Cui owns the native CUI dependency without referencing the compiler,
  CLI or TUI. GUI consumers opt in through CLR references. Unmodified upstream
  source and notices live under `third-party/cui/`.
- The UI family uses the `Fpas.Ui` namespace prefix with matching project,
  assembly and package names. `Fpas.Ui.Mvu` and the shared `Fpas.Ui` widget
  library remain independent assemblies. Both hosts reference them; only
  `Fpas.Ui.Gui` references `Fpas.Ui.Gui.Cui`.
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
keep fixtures unchanged during tests. The [coverage map](../../tests/coverage.md)
links language and build contracts to their tests. xUnit cases cover
the compiler and local SDK, including project/package reference execution,
incremental builds, and Clean/Rebuild. Test build targets obtain actual reference
paths; the checked-in Hello source is copied into the integration-test output.

## Examples and applications

The two consumer directories have distinct purposes:

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
The [Units example](../../examples/basics/units/README.md) compiles a program and two
units into one assembly. The [doc-examples application](../../apps/doc-examples/)
checks Markdown sources and explicit contexts and supplies compiled assemblies
to behavior tests.

## Extending the implementation

Preserve compile/execute integration coverage when extending the compiler or SDK.
Add Fpas.Core contents only when emitted features need them. Follow the owning
language/build contracts and their tracked gaps. Add executable examples/apps
using implemented features. Application delivery follows the
[publishing contract](../build/publishing.md). The [VS Code extension](../../src/VSCode/README.md)
lives in `src/VSCode`. Repository commands belong in the
[development workflow](../build/development.md).
