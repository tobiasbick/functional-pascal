---
name: fpas-projects
description: Create or edit CLR FPAS .fpasproj projects and mixed .slnx solutions, source selection, dependencies, and SDK build configuration.
---

# FPAS CLR projects

Use MSBuild `.fpasproj` files and `.slnx` solutions. The local Fpas.NET.Sdk selects
sources and invokes the direct CIL compiler. Follow
[AGENTS.md](../../../AGENTS.md) and [CONTRIBUTING.md](../../../CONTRIBUTING.md).

## Read for the task

| Task | Reference |
| --- | --- |
| Project shape, sources, dependencies, and solutions | [Projects](../../../docs/pascal/program-structure/projects.md) |
| Project, library, test, and solution templates, build, and run | [Project commands](../../../docs/pascal/program-structure/project-commands.md) |
| Units, source identity, aliases, and visibility | [Program structure](../../../docs/pascal/program-structure/README.md) |
| Restore/build, warning policy, incremental state, and cleanup | [MSBuild contract](../../../docs/build/msbuild.md) |
| Bootstrap, toolchain configuration, SDK package versions/cache | [Development workflow](../../../docs/build/development.md) |
| Outside-checkout consumer, isolated feed/cache, and runnable deployment | [Standalone SDK walkthrough](../../../docs/build/standalone-sdk.md) |
| Assemblies, symbols, and runtime dependencies | [Compiled output](../../../docs/pascal/program-structure/compiled-programs.md) |
| Application apphosts, Native AOT, bundles and signing | [Publishing](../../../docs/build/publishing.md) |
| Build and cross-language verification | [Build acceptance](../../../tests/coverage.md#build-acceptance) |
| Native FPAS test projects and runners | [Testing](../../../docs/pascal/tools/testing.md) |

## Project workflow

1. Use a `fpas init` template when it fits the new project or solution. For
   existing projects, inspect the current SDK selection and source items.
   Confirm the configured feed has the selected SDK version. If testing a changed
   toolchain, package its matching SDK before restoring the consumer. When
   repacking the same version, evict it from the target NuGet cache before
   restoring. Use isolated caches for concurrent tests.
2. Set net10.0, AnyCPU, and OutputType Exe or Library. Executables contain exactly
   one program plus optional units; libraries contain units without a program.
   Each source header defines its identity, independent of filename/order.
3. Use the recursive `.fpas` Compile items or refine them with Include/Remove.
   Disable EnableDefaultFpasCompileItems when selecting all sources explicitly.
   Sources selected by a project compile into one assembly.
4. Declare ProjectReference, PackageReference, or Reference/HintPath on the
   consumer. Source uses aliases resolve those supplied references; they do not
   acquire packages or define project dependencies. The SDK supplies framework
   references and its matching Core dependency.
5. Put related projects in a `.slnx` with the
   [FPAS project-type mapping](../../../docs/build/msbuild.md#project-identity-and-solution-shape).
   ProjectReference edges define build order.
   Inspect the [mixed fixture](../../../tests/Fixtures/msbuild/Game.slnx) for a
   concrete layout. Public/private source declarations govern exported APIs.
6. Use explicit project or solution paths with `fpas build` or `dotnet build`.
   To resolve and launch a project executable, use `fpas run`. Select `--project`
   when a solution has several executables. Use `fpas test` for native test projects.

## Verification

Use [fpas-authoring](../fpas-authoring/SKILL.md) for source edits. Select relevant
xUnit SDK tests for source/reference changes, exported metadata, incremental
builds, Clean/Rebuild, and configuration isolation. Test compilation and execution
are separate steps; do not replace running toolchain binaries.

Use the [publishing contract](../../../docs/build/publishing.md) for application
apphosts, Native AOT, self-contained/single-file outputs and signing. For editor
work, use the [VS Code extension guide](../../../src/VSCode/README.md) to restore
and select the analysis context and verify project, test, or debug commands.
StandaloneSdkGuideTests verifies the isolated SDK walkthrough in Debug and Release.
