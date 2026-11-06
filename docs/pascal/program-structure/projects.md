# Projects and solutions

FPAS applications use MSBuild .fpasproj files and can share a .slnx solution
with C# projects. Restore resolves dependencies; the SDK invokes the direct
CIL compiler with selected sources and resolved reference assemblies.

Use [fpas init/build/run](project-commands.md) to create and execute projects and
solutions. `init solution` creates `.slnx`; there is no separate workspace format.

## Project shape

```xml
<Project Sdk="Fpas.NET.Sdk/0.1.0-dev">
  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework>
    <OutputType>Exe</OutputType>
  </PropertyGroup>
</Project>
```

The local SDK must be packaged before this project can restore. Follow the
[application workflow](../../build/development.md#build-an-fpas-application).
For projects outside the checkout, use the
[standalone SDK walkthrough](../../build/standalone-sdk.md) with an explicit
package source, SDK version, and independent cache.
The current build scope is one net10.0 target, AnyCPU, Debug/Release, and Exe
or Library output. A library contains units and no program; an executable has
exactly one program plus optional units. The program/unit header determines
source identity, not the source filename or Compile item order.

For [native .NET tests](../tools/testing.md), set IsTestProject=true and reference
the framework, VSTest adapter and Microsoft.NET.Test.Sdk packages. The SDK selects
Library output and emits deps/runtimeconfig files for the native test host.
Test libraries contain units and no program; `fpas init test` creates this shape.

## Sources and dependencies

The SDK selects .fpas sources recursively, excluding normal build directories.
Use ordinary Compile Include/Remove items to refine the set, or disable
EnableDefaultFpasCompileItems and select it explicitly. Sources are UTF-8.

Use ProjectReference for FPAS/C# dependencies, PackageReference for NuGet APIs,
and Reference/HintPath for an explicit assembly. The SDK supplies framework
reference assemblies and its matching Fpas.Core dependency. Uses statements
resolve names after these references exist; they do not acquire libraries.

The mixed example in the [MSBuild specification](../../build/msbuild.md) declares
the FPAS solution type and builds a C# library, an FPAS library and a C# host in
dependency order. ProjectReference edges define the graph; solution ordering
does not replace those edges. No IDE extension is required for command-line builds.

## Outputs and rebuilds

Builds produce ordinary assemblies, portable symbols, and application host files.
Optional XML documentation follows normal output/reference copying. No-change
builds skip compilation; source/reference lists and contents, compiler assets,
and relevant options participate in invalidation. Clean/Rebuild tracks owned
outputs. See [compiled programs](compiled-programs.md) and the
[build acceptance](../../../tests/coverage.md#build-acceptance) for verified details.

Application publication, Native AOT, apphosts and signing follow the
[publishing contract](../../build/publishing.md). Multi-targeting, additional
platform/framework coverage and IDE project loaders have separate scope. A requested
unsupported build feature must fail clearly rather than silently pretend to work.

## See also

- [Development commands](../../build/development.md)
- [Imports](imports.md)
- [Compiler CLI](cli.md)
- [MSBuild contract](../../build/msbuild.md)
