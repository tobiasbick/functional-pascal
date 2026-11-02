# MSBuild and solution integration

**Status: partially implemented.** The local SDK builds multi-source net10.0
programs/libraries and mixed C#/FPAS solutions using direct CIL output. See the
[implemented milestone](../implementation/msbuild.md) for bootstrap instructions,
verified behavior, and exact limits. Portable PDBs and [XML documentation](../implementation/xml-documentation.md)
are implemented. This document also defines wider goals, including public language
types which remain pending.

## Initial goal and scope

Provide `.fpasproj` projects alongside `.csproj` and `.fsproj` projects in a
`.slnx` solution. With the local FPAS compiler and SDK resolved, the normal entry
points are `dotnet restore`, `dotnet build`, and `dotnet clean`. Rebuild uses the
ordinary MSBuild Rebuild target.

The complete initial integration goal includes:

- A single explicit TargetFramework per FPAS project, Debug/Release, AnyCPU,
  library output, and executable output.
- Standard ProjectReference, PackageReference, and assembly reference inputs.
- FPAS-to-C# and C#-to-FPAS references in one dependency graph.
- Framework reference assemblies, package assets, incremental compilation,
  source diagnostics, portable PDBs, optional XML API documentation, and normal
  managed application host files.
- A reusable FPAS Project SDK rather than per-project compiler shell scripts.

Use a .NET SDK with `.slnx` support. The first validation baseline is .NET 10 SDK
and net10.0 examples; this does not make a framework version a language keyword
or permanently require that every FPAS application target net10.0.

Visual Studio project-system/CPS integration, LSP, VS Code integration, source
queries, formatter/debugger integration, and automatic IDE project creation are
outside this step. Building an FPAS project must not require an IDE extension.
Multi-targeting FPAS projects, custom solution-management commands, publishing,
AOT, signing, and deployment profiles are separate later work. Ordinary package
consumption is included; a public compiler/SDK publishing workflow is not.

## Project identity and solution shape

Use `.fpasproj` for the new MSBuild project format. It is an XML MSBuild project,
not a source manifest that happens to have a similar filename.

Assign the FPAS project type this stable identity for the planned integration:

```text
Name: FPAS
Extension: .fpasproj
TypeId: {5D4B15AB-1024-4803-9015-4F069B13DE5A}
```

This is an FPAS identifier defined by this plan, not an already registered Visual
Studio project factory. The solution format permits its own extension/type mapping:

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

Initially a checked-in solution may declare its FPAS mapping explicitly. Do not
assume that registering a solution type also makes every `dotnet sln` project-add
workflow or IDE project loader understand the new language. Validate such
additional workflows separately; they are not needed to build this solution.

## FPAS Project SDK

The SDK identifier is `Fpas.NET.Sdk`. It supplies Sdk.props, Sdk.targets, and
compiler invocation; a compiler-support reference is added when emitted features
need one. The following illustrates the full goal, including XML documentation
which is not yet implemented. Use the milestone's minimal project to build today:

```xml
<Project Sdk="Fpas.NET.Sdk/0.1.0">
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

Versions here identify the proposed project; no public FPAS SDK is published.
The implemented development bootstrap uses the repository's sdk/feed local
package source and version 0.1.0-dev. Missing SDK/compiler
components must fail clearly instead of falling back to a global unrelated tool.

The [repository layout plan](../architecture/project-layout.md) places the C#
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

## Compiler and build contract

| Interface | Required behavior |
| --- | --- |
| Restore | Resolve framework, PackageReference, ProjectReference, and compiler-support assets through standard restore infrastructure |
| Build | Resolve dependencies and compile changed FPAS inputs to the selected output |
| CoreCompile | Invoke the FPAS compiler with all selected sources, resolved references, and options; emit direct CIL/metadata/PDB and XML documentation when enabled |
| Clean / Rebuild | Remove this project's generated artifacts; rebuild without stale compiler results |
| GetTargetFrameworks | Expose framework information for standard project-reference negotiation, even for one target |
| GetTargetPath | Return the actual produced assembly with metadata expected by reference-resolution targets |
| Output items/properties | Expose TargetPath, intermediate/output locations, and generated-file tracking through standard MSBuild conventions |

Reuse standard targets instead of independently reimplementing the restore graph.
The exact imported-target order is verified during implementation because the
base SDK may otherwise select language-specific targets by project extension.

Compiler inputs include the selected source list, resolved compile-time reference
assemblies, package/project references, compiler-support identity, configuration,
target framework, nullable settings defined by the language, and output/debug
options. Bind reference metadata, not whichever implementation assemblies happen
to be loaded into the MSBuild task process. Response files or structured task
arguments must preserve paths and arguments without shell interpretation.

A library has units and no program. An executable compilation has exactly one
program, with normal/async entry behavior from the language specification.
Produce an implementation assembly and PDB plus standard runtime/dependency
files needed by executable output. A framework-dependent `.dll` runnable through
`dotnet` is enough initially; a native apphost is not required.

Separate reference assemblies are optional initially. Until the compiler can
emit them, expose the real implementation assembly and do not advertise missing
reference-assembly paths. C# consumers must receive the actual metadata defined
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
[comments and API documentation](../pascal/language/comments.md#xml-export-contract).
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

## Diagnostics, incremental work, and failure

Report compiler diagnostics as MSBuild errors/warnings with stable codes, source
paths, and line/column information. Preserve structured compiler information;
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

## Implementation sequence and acceptance

1. Add the Project SDK props/targets and a compiler invocation task/driver. Prove
   SDK resolution from a clean development checkout and deterministic source and
   reference selection. Validate the example `.slnx` and its FPAS mapping.
2. Compile real minimal FPAS library and executable sources through CoreCompile.
   Inspect IL to establish direct FPAS output, and execute the produced program.
   A dummy target that only creates a file is not an end-to-end compiler proof.
3. Build the Shared.csproj -> Game.fpasproj -> Host.csproj fixture. The C# host
   must call the FPAS library; FPAS must call the C# library. Check transitive
   dependencies and public record/union/domain metadata without manual DLL copying.
   Enable XML documentation, parse the sidecar, and verify documentation IDs and
   parameter names against emitted metadata. Check that the C# reference receives
   the matching XML alongside the assembly; no IDE is needed for this check.
4. Add a consumed NuGet API to the FPAS fixture. Verify restore, reference binding,
   runtime asset resolution, and execution from the normal output directory.
5. Exercise Debug/Release, no-change builds, source/reference additions and removals,
   compiler-option/version changes, Clean, Rebuild, and parallel solution builds.
   Compare with clean output and verify useful diagnostics/nonzero error exits.
   Include comment-only edits, missing XML outputs, custom documentation paths,
   and switching generation on/off without leaving stale documentation.

The intended verification entry points, once implemented, include:

```sh
dotnet restore Game.slnx
dotnet build Game.slnx --no-restore
dotnet build Game.slnx --no-restore -c Release
dotnet clean Game.slnx
dotnet msbuild Game.slnx -restore -t:Rebuild
```

The implemented mixed-project tests run these build/clean/rebuild entry points;
the broader acceptance cases above remain the goal. No Visual Studio/VS Code
extension or language server may be required for them.

## References

- [Solution XML schema](https://github.com/microsoft/vs-solutionpersistence/blob/main/src/Microsoft.VisualStudio.SolutionPersistence/Serializer/Xml/Slnx.xsd)
- [MSBuild project SDKs](https://learn.microsoft.com/en-us/visualstudio/msbuild/how-to-use-project-sdk)
- [MSBuild project-reference targets](https://github.com/dotnet/msbuild/blob/main/src/Tasks/Microsoft.Common.CurrentVersion.targets)
