# MSBuild and solution integration

The local SDK builds net10.0/AnyCPU multi-source programs/libraries and mixed
C#/FPAS solutions with direct CIL output, portable PDBs, and optional XML API
documentation. Public records, unions, domains, and generics cross project
boundaries. Use the [development workflow](development.md#build-an-fpas-application)
for bootstrap commands and [build acceptance](../../tests/coverage.md#build-acceptance)
for verification and explicit scope limits.

## Scope

`.fpasproj` projects build alongside `.csproj` and `.fsproj` projects in a
`.slnx` solution. With the local FPAS compiler and SDK resolved, the normal entry
points are `dotnet restore`, `dotnet build`, and `dotnet clean`. Rebuild uses the
ordinary MSBuild Rebuild target.

The supported build scope is .NET 10/net10.0, one explicit TargetFramework,
AnyCPU, Debug/Release and library/executable output. Standard project, package
and assembly references form the dependency graph, including FPAS/C#/F# edges.
Building requires no IDE extension. Multi-targeting, IDE project loaders,
publishing/AOT/signing and compiler distribution remain in the
[toolchain gaps](../future/cli-gap.md).

## Project identity and solution shape

`.fpasproj` is an XML MSBuild project.

The FPAS project type has this stable identity:

```text
Name: FPAS
Extension: .fpasproj
TypeId: {5D4B15AB-1024-4803-9015-4F069B13DE5A}
```

This project type is not a registered Visual Studio project factory. A solution
declares the extension/type mapping explicitly:

```xml
<Solution>
  <Configurations>
    <BuildType Name="Debug" />
    <BuildType Name="Release" />
    <Platform Name="Any CPU" />
    <ProjectType Name="FPAS" Extension=".fpasproj"
                 TypeId="{5D4B15AB-1024-4803-9015-4F069B13DE5A}">
      <Platform Project="AnyCPU" />
    </ProjectType>
  </Configurations>
  <Project Path="Shared/Shared.csproj" />
  <Project Path="Game/Game.fpasproj" />
  <Project Path="Host/Host.csproj" />
</Solution>
```

In the fixture, Game references Shared, and Host references Game. ProjectReference
provides build edges; solution ordering is not a substitute for those references.
Do not label the FPAS project as C# to make it load. MSBuild can identify an
otherwise unknown project type as an MSBuild XML project.

`fpas init solution` writes this mapping. Other solution-management commands and
IDE project loaders need separate acceptance.

## FPAS Project SDK

The SDK identifier is `Fpas.NET.Sdk`. It supplies Sdk.props, Sdk.targets, and
compiler invocation; a compiler-support reference is added when emitted features
need one. The following project enables XML documentation:

```xml
<Project Sdk="Fpas.NET.Sdk/0.1.0-dev">
  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework>
    <OutputType>Library</OutputType>
    <GenerateDocumentationFile>true</GenerateDocumentationFile>
  </PropertyGroup>
  <ItemGroup>
    <ProjectReference Include="../Shared/Shared.csproj" />
    <PackageReference Include="Newtonsoft.Json" Version="13.0.3" />
  </ItemGroup>
</Project>
```

No public FPAS SDK is published.
The development bootstrap uses the repository's sdk/feed local
package source and version 0.1.0-dev. Missing SDK/compiler
components must fail clearly instead of falling back to a global unrelated tool.

The [repository layout](../architecture/project-layout.md) places the C#
compiler, CLI, and MSBuild task in src, SDK props/targets and packaging in sdk,
and xUnit unit/integration tests in tests, grouped by Fpas.slnx. The SDK selects
Fpas.Build.Tasks, which invokes Fpas.Cli as a separate compiler process. These
are implementation-project boundaries; FPAS applications consume the resulting
SDK through their own `.fpasproj` projects.

Reuse language-neutral .NET SDK/NuGet infrastructure for restore, framework
references, project-reference negotiation, and application assets where possible.
Supply FPAS language targets and replace the compile stage. Simply importing the
.NET SDK and renaming a C# project is not sufficient: no Csc/Vbc/Fsc compilation
or generated C# source belongs in the FPAS compile path.

Configure PackageReference-style restore explicitly for the custom extension,
including the appropriate RestoreProjectStyle property. Do not rely on NuGet
recognizing `.fpasproj` as a built-in language project.

The SDK sets the FPAS language identity and project-type properties consistently.
It includes `**/*.fpas` as Compile items exactly once, excluding bin/obj and other
standard build outputs. Disable conflicting inherited language source globs.
Standard Compile Include/Remove items may refine selection. Sources stay UTF-8;
their program/unit headers, not glob order, determine language identity.

Source-level `uses` imports do not acquire packages or establish build references.
MSBuild resolves references first; the compiler binds the resulting metadata.

### Native test projects

IsTestProject=true identifies a test project; OutputType still selects Exe or
Library for ordinary projects. FPAS VSTest projects contain units and compile as
Library. After importing the base SDK and test-package targets, the FPAS SDK
overrides their Exe default, sets GenerateProgramFile=false and requests standard
GenerateRuntimeConfigurationFiles/GenerateDependencyFile outputs. Core, framework
and package dependencies use normal copy/runtime asset handling. No generated
C# entry point or dummy FPAS program is involved. Analysis and compilation see
the same library target.

Imported test-package preparation hooks join FpasPrepareCompilationDependsOn,
including coverage path maps and adapter generation/validation targets. Build and
check execute the same dependency graph. Pre-compile hooks explicitly reachable
through that shared graph are accepted; unrelated custom Build/CoreCompile hooks
remain diagnosed rather than silently omitted by analysis.

Reference Microsoft.NET.Test.Sdk plus an actual framework and VSTest adapter.
`dotnet test` runs the generated assembly directly. The [testing guide](../pascal/tools/testing.md)
owns template versions and supported framework scenarios. The SDK diagnoses
IsTestingPlatformApplication=true as unsupported for FPAS test projects; the
Microsoft.Testing.Platform application mode is outside this VSTest contract.

## Compiler and build contract

| Interface | Required behavior |
| --- | --- |
| Restore | Resolve framework, PackageReference, ProjectReference, and compiler-support assets through standard restore infrastructure |
| Build | Resolve dependencies and compile changed FPAS inputs to the selected output |
| CoreCompile | Invoke the FPAS compiler with all selected sources, resolved references, and options; emit direct CIL/metadata/PDB and XML documentation when enabled |
| FpasPrepareCompilation | Shared compiler-input preparation for CoreCompile and analysis; generators use BeforeTargets or FpasPrepareCompilationDependsOn |
| FpasDescribeAnalysisContext / FpasExportAnalysisContext | Export versioned graph/evaluated source and resolved reference inputs for project analysis, without compiling the selected root |
| Clean / Rebuild | Remove this project's generated artifacts; rebuild without stale compiler results |
| GetTargetFrameworks | Expose framework information for standard project-reference negotiation, even for one target |
| GetTargetPath | Return the actual produced assembly with metadata expected by reference-resolution targets |
| Output items/properties | Expose TargetPath, intermediate/output locations, and generated-file tracking through standard MSBuild conventions |

Reuse standard targets instead of independently reimplementing the restore graph.
The imported-target order must select language-neutral targets explicitly;
the base SDK can otherwise select language-specific targets by project extension.

Compiler inputs include the selected source list, resolved compile-time reference
assemblies, package/project references, compiler-support identity, configuration,
target framework, nullable settings defined by the language, and output/debug
options. Optimize defaults to false for Debug and true for Release; it is
forwarded to the compiler and included in the compilation fingerprint. Portable
symbols remain available in either configuration. This controls the emitted
DebuggableAttribute/JIT policy, not a separate FPAS CIL optimization pass.
Bind reference metadata, not whichever implementation assemblies happen
to be loaded into the MSBuild task process. Response files or structured task
arguments must preserve paths and arguments without shell interpretation.

A library has units and no program. An executable compilation has exactly one
program, with normal/async entry behavior from the language specification.
Produce an implementation assembly and PDB plus standard runtime/dependency
files needed by executable output. A framework-dependent `.dll` runnable through
`dotnet` is supported; native apphosts have separate scope.

The SDK exposes the implementation assembly rather than a separate reference
assembly. C# consumers receive the actual metadata defined
in [CLR mapping](../specs/clr-mapping.md), including public language types.

Fpas.Core is an implementation dependency, not an implicit source namespace.
Resolve a compatible support assembly with the selected compiler/SDK; propagate
its runtime dependency to consuming applications through normal asset handling.
Do not require users to manually copy it or package dependencies beside outputs.

## Documentation output

Support the ordinary GenerateDocumentationFile and DocumentationFile properties.
The project example enables XML output. If neither property is supplied, generation
defaults to false. A nonempty DocumentationFile enables generation when
GenerateDocumentationFile is unset; an explicit false disables it. When enabled
without an explicit path, publish `$(TargetDir)$(TargetName).xml` beside the DLL.
Resolve an explicit relative path against the project directory. Compile tasks
must receive a normalized output path, independent of their working directory.

The compiler emits the standard XML described in
[comments and API documentation](../pascal/language/basics/comments.md#xml-export-contract).
Retain the SDK's ordinary separation of intermediate compiler output and final
output copying, and track every generated file. For a custom DocumentationFile,
also publish a conventionally named `$(TargetName).xml` beside TargetPath so
standard assembly-reference consumers can associate it with the DLL. Avoid a
duplicate copy when the requested and conventional paths coincide.

Expose/copy XML as a related assembly output through standard MSBuild reference
handling, including a C# or VB ProjectReference consumer. Do not turn it into an
assembly reference or require users to copy it manually. Documentation changes
cannot change assembly identity or require a runtime support dependency.

Include generation settings, requested paths, and source comment changes in
incremental tracking. A missing requested/generated sidecar invalidates an
otherwise up-to-date build. Clean/Rebuild and disabling generation remove stale
project-owned documentation artifacts. Do not remove unrelated XML files.
Documentation warnings follow ordinary build warning policy; failing to write a
requested output is a build error. These requirements specify compiler/build
output and do not require an editor or language-service implementation.

## SDK task and incremental state

Fpas.Build.Tasks launches the packaged CLI in a separate bounded .NET process
using a JSON response file. The compiler is not loaded into the MSBuild host.
The task uses the host SDK's MSBuild assemblies rather than bundling private
copies. Project/configuration/framework intermediates remain independent.

The compiler fingerprint hashes selected source/reference paths and contents,
compiler/task assets, runtime version, and relevant build/documentation/warning
options. Missing or altered requested assemblies, symbols, or documentation
invalidate the fingerprint. Cached documentation warnings are reported on
unchanged builds; suppressing recompilation must not hide those diagnostics.

XML output uses a separate intermediate file before publishing the requested
destination and conventional assembly-named sidecar. Standard DocFileItem and
FinalDocFile handling supplies documentation to reference consumers. FileWrites
tracks owned outputs; a successful build manifest records custom XML paths,
including paths outside bin/obj. Replacement, disabling generation, and Clean
remove those owned paths without wildcard XML deletion. Before CoreClean the
SDK explicitly removes manifest-owned custom paths which standard cleanup excludes.

## Diagnostics, incremental work, and failure

Report compiler diagnostics as MSBuild errors/warnings with stable codes, source
paths, and start/exclusive-end UTF-16 line/column information. Non-source errors
have no file or fabricated source position. Log explanation, help, related notes
and fix titles as messages instead of additional warning/error entries. Preserve
the full JSON payload when caching/replaying warnings. Preserve structured compiler information;
do not guess severity by searching human-readable text. Errors make Build fail
and `dotnet build` return nonzero. A failed compile must not publish partial new
assemblies or present stale output as the successful result of this invocation.

Track sources, source-list additions/removals, reference-list and content changes,
compiler/support versions, and relevant build properties. A no-change build skips
compilation; changing any semantic input invalidates it. Timestamp checks over
only currently selected files are insufficient to notice removed sources.
Use the appropriate standard dependency tracking plus a compiler-input fingerprint.

Parallel solution builds must respect project edges and isolate intermediate
state by project/configuration/framework. Do not use a shared mutable global
compiler cache without proper identity and concurrency handling. An incremental
rebuild must produce behavior equivalent to a clean build.

## Verification

The [build coverage map](../../tests/coverage.md#build-acceptance) owns executable
acceptance for SDK resolution, mixed consumers, package/runtime assets,
diagnostics, XML/PDB output, incremental invalidation and Clean/Rebuild parity.
Commands belong in the [development workflow](development.md).

## References

- [Solution XML schema](https://github.com/microsoft/vs-solutionpersistence/blob/main/src/Microsoft.VisualStudio.SolutionPersistence/Serializer/Xml/Slnx.xsd)
- [MSBuild project SDKs](https://learn.microsoft.com/en-us/visualstudio/msbuild/how-to-use-project-sdk)
- [MSBuild project-reference targets](https://github.com/dotnet/msbuild/blob/main/src/Tasks/Microsoft.Common.CurrentVersion.targets)
