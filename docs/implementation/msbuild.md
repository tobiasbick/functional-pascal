# Implemented MSBuild integration

The initial `Fpas.NET.Sdk` package builds real `.fpasproj` projects through
`dotnet build`. It uses the direct CIL compiler; no C# source is generated for
FPAS. This implements part of the [MSBuild plan](../build/msbuild.md), with the
limits below.

## Local SDK bootstrap

From the repository root with a stable .NET 10 SDK:

```sh
dotnet restore Fpas.slnx
dotnet build Fpas.slnx --no-restore --configuration Debug
dotnet pack sdk/Fpas.NET.Sdk/Fpas.NET.Sdk.csproj --no-build --configuration Debug --output sdk/feed
dotnet restore examples/hello/Hello.fpasproj
dotnet build examples/hello/Hello.fpasproj --no-restore
dotnet examples/hello/bin/Debug/net10.0/Hello.dll
```

The program prints `Hello from FPAS`. The root NuGet.Config includes the local
sdk/feed directory and nuget.org. No public FPAS SDK publication is required.
Fpas.slnx contains the implementation and test projects, not FPAS consumers:
the package must exist before those consumers can resolve their SDK.

NuGet caches packages by identity/version. Repacking `0.1.0-dev` does not replace
an already installed package in that cache. For subsequent toolchain changes,
pack with a new `-p:PackageVersion=0.1.0-dev.2` and update the consumer's SDK
version accordingly. Alternatively, while using the local development version,
remove only `fpas.net.sdk/0.1.0-dev` from the NuGet global-packages directory
before restoring. `dotnet nuget locals global-packages --list` identifies that
directory. Do not clear unrelated packages. Integration tests use a unique
package version per run so they always exercise the current toolchain.

## Project contract

[Hello.fpasproj](../../examples/hello/Hello.fpasproj) is the minimal executable:

```xml
<Project Sdk="Fpas.NET.Sdk/0.1.0-dev">
  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework>
    <OutputType>Exe</OutputType>
  </PropertyGroup>
</Project>
```

Supported today:

- One or more selected UTF-8 FPAS sources per project, net10.0, AnyCPU,
  Debug/Release, and OutputType Exe or Library. Executables require exactly one
  program plus optional units; libraries require units and no program.
- Default `**/*.fpas` Compile items with normal build-directory exclusions.
  Compile Remove works; set EnableDefaultFpasCompileItems=false to select inputs
  explicitly. Each file declares one program or unit; names must be unique.
- Framework, ProjectReference, PackageReference, and resolved assembly references
  supplied by the normal .NET SDK. `uses` binds this metadata and does not restore
  packages. Only APIs supported by the [compiler subset](first-program.md) bind.
- Restore, Build, Clean, Rebuild, target paths, managed dependency/runtime files,
  and transitive runtime copying. Executables run with `dotnet <assembly>.dll`.
- Mixed `.slnx` builds with the FPAS project-type mapping shown in the plan and
  [checked-in fixture](../../tests/Fixtures/msbuild/Game.slnx).
- Portable PDBs by default, or DebugType=none. Symbols propagate to outputs and
  referencing C# projects and participate in fingerprint validation and Clean.
- Optional [XML documentation](xml-documentation.md), including DocumentationFile
  custom paths, C# sidecar propagation, warning policy, and owned-output cleanup.

The SDK imports the .NET SDK's language-neutral targets, sets FPAS language
identity, and provides CoreCompile. Csc, Vbc, and Fsc are not part of its compile
path. The packaged tools include the matching CLI/compiler and MSBuild task.
MSBuild assemblies come from the host SDK, not a private bundled copy. The task
launches the CLI in a separate bounded process, passing a JSON response file
without shell argument interpretation.

Each configuration/framework keeps its own intermediate assembly and compiler
fingerprint. The fingerprint hashes the selected source/reference paths and
contents, compiler/task assets, runtime version, and supported build options.
Missing or changed intermediate assemblies or requested PDBs invalidate it. A no-change build
skips the compiler. Clean tracks the generated assembly, PDB, fingerprint, argument
file, and compiler runtime configuration through ordinary FileWrites.

Compiler diagnostics preserve FPAS codes, source paths, lines, and columns as
MSBuild errors. A compile failure makes the build fail; previous outputs are not
reported as successful new compilation. Build-adapter codes are FP5001 for an
unsupported configuration, FP5002 for I/O/process/result parsing errors, FP5003
for compiler failures without diagnostics or missing output, and FP5004 for
cancellation/time limit.

## Verification and remaining scope

`dotnet test Fpas.slnx --no-build --no-restore` packs the current SDK into an
isolated local feed. The mixed fixture builds Shared.csproj → Game.fpasproj →
Host.csproj and executes the C# host calling an FPAS function which calls C#.
Tests also exercise a consumed local NuGet package, transitive runtime assets,
Debug/Release, incremental source/reference changes, missing output, Compile
Remove, added/removed sources, source diagnostics, Clean, and Rebuild. FPAS
intermediate directories are checked for accidental C# generation.

The [expression/control-flow milestone](expressions-and-control-flow.md) adds an
actual Mandelbrot `.fpasproj` example, checked pixel by pixel, and public bool/double
signatures consumed by C#. The [multiple-source milestone](multiple-sources.md)
adds multi-unit projects, source visibility, and incremental unit-set tracking.
The [CLR milestone](clr-objects-and-resources.md) adds a StreamReader project,
object/nullable signatures consumed by C#, and resource/exception execution tests.

Other target frameworks/platforms, embedded resources, signing, publishing, native apphosts, and
IDE integration remain unimplemented. Explicit requests for supported-plan
features such as embedded resources or a non-AnyCPU platform fail rather
than silently producing those artifacts. [Records](records.md) and [unions](unions-and-matching.md) use Fpas.Core
metadata attributes. The SDK bundles the matching Core assembly, supplies it as
a reference, and copies it with runtime dependencies, including through C# hosts.
Domains remain pending. [Async entry points and portable debugging](overloads-interfaces-debugging-and-async.md)
are implemented.
