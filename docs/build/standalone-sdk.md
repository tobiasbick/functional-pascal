# Use the SDK outside the checkout

This walkthrough builds an independent consumer with the packaged compiler:
C# Shared → FPAS Messages → FPAS App and C# Host. The consumer has its own
NuGet configuration, feed, and initially empty package cache. It needs the
.NET 10 SDK for builds and the .NET 10 runtime to execute the resulting DLLs.
The current project scope is net10.0, AnyCPU, Debug/Release, and managed
executables/libraries. No public FPAS package is published.

## Prepare one versioned package

With a [single-file compiler distribution](distribution.md), export its embedded
matching package with `fpas sdk export ./feed`; no checkout is needed. The commands
below instead build a development package with an explicit walkthrough version.
When using the exported package, replace `0.1.0-local.1` in the consumer SDK
identifiers with the version in the exported package filename.

From the compiler checkout, restore and build the toolchain, then pack it:

```sh
dotnet restore Fpas.slnx
dotnet build Fpas.slnx --no-restore --configuration Debug -warnaserror
dotnet pack sdk/Fpas.NET.Sdk/Fpas.NET.Sdk.csproj --no-build --configuration Debug --output .temp-data/standalone-feed -p:PackageVersion=0.1.0-local.1
```

Create a separate directory outside the checkout, called `consumer` here.
Copy `.temp-data/standalone-feed/Fpas.NET.Sdk.0.1.0-local.1.nupkg` into its
`feed` subdirectory. This package includes the matching compiler, CLI, MSBuild
task, and Fpas.Core support assembly. The consumer needs no compiler checkout
or separately installed FPAS tool.

## Create the consumer files

Save each block under the indicated path relative to `consumer`. Create the
parent directories as needed. The integration test builds these exact blocks.

`NuGet.Config` selects only the local feed; the relative paths are resolved
from this configuration file. Framework references come from the installed SDK.
This example has no external PackageReference dependencies.

```xml file=NuGet.Config
<configuration>
  <packageSources>
    <clear />
    <add key="fpas-local" value="feed" />
  </packageSources>
  <config>
    <add key="globalPackagesFolder" value=".packages" />
  </config>
</configuration>
```

`Directory.Build.props` applies the same target and documentation settings to
all four projects:

```xml file=Directory.Build.props
<Project>
  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework>
    <PlatformTarget>AnyCPU</PlatformTarget>
    <Nullable>enable</Nullable>
    <ImplicitUsings>enable</ImplicitUsings>
    <GenerateDocumentationFile>true</GenerateDocumentationFile>
  </PropertyGroup>
</Project>
```

The solution declares the FPAS project type for command-line MSBuild. Its
ProjectReference edges define build order.

```xml file=Consumer.slnx
<Solution>
  <Configurations>
    <BuildType Name="Debug" />
    <BuildType Name="Release" />
    <Platform Name="Any CPU" />
    <ProjectType Name="FPAS" Extension=".fpasproj" TypeId="{5D4B15AB-1024-4803-9015-4F069B13DE5A}">
      <Platform Project="AnyCPU" />
    </ProjectType>
  </Configurations>
  <Project Path="src/Shared/Shared.csproj" />
  <Project Path="src/Messages/Messages.fpasproj" />
  <Project Path="src/App/App.fpasproj" />
  <Project Path="src/Host/Host.csproj" />
</Solution>
```

The C# dependency exposes an ordinary static CLR method:

```xml file=src/Shared/Shared.csproj
<Project Sdk="Microsoft.NET.Sdk" />
```

```csharp file=src/Shared/Words.cs
namespace Standalone.Shared;

/// <summary>Shared greeting text.</summary>
public static class Words
{
    /// <summary>Formats a name.</summary>
    public static string For(string name) => "Hello, " + name;
}
```

The FPAS library imports that dependency and exports a public record/function.
Both FPAS projects select the explicit package version.

```xml file=src/Messages/Messages.fpasproj
<Project Sdk="Fpas.NET.Sdk/0.1.0-local.1">
  <PropertyGroup>
    <OutputType>Library</OutputType>
  </PropertyGroup>
  <ItemGroup>
    <ProjectReference Include="../Shared/Shared.csproj" />
  </ItemGroup>
</Project>
```

```pascal file=src/Messages/Messages.fpas
// Greeting API.
unit Standalone.Messages;

uses Standalone.Shared as Shared;

// A greeting returned to callers.
public type Greeting = record
  // The formatted text.
  public Text: string;
end record;

// Creates a greeting through the C# dependency.
public function Create(Name: string): Greeting;
begin
  return new Greeting(Shared.Words.For(Name));
end function;

end unit;
```

The FPAS executable references the library:

```xml file=src/App/App.fpasproj
<Project Sdk="Fpas.NET.Sdk/0.1.0-local.1">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
  </PropertyGroup>
  <ItemGroup>
    <ProjectReference Include="../Messages/Messages.fpasproj" />
  </ItemGroup>
</Project>
```

```pascal file=src/App/Main.fpas
program StandaloneApp;

uses Standalone.Messages as Messages, System.Console as Console;

begin
  Console.WriteLine(Messages.Create('FPAS').Text);
end;

end program;
```

The C# executable consumes the same FPAS API:

```xml file=src/Host/Host.csproj
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
  </PropertyGroup>
  <ItemGroup>
    <ProjectReference Include="../Messages/Messages.fpasproj" />
  </ItemGroup>
</Project>
```

```csharp file=src/Host/Program.cs
Console.WriteLine(Standalone.Messages.Create("C#").Text);
```

## Restore, build, and run

Start a shell in `consumer`. Select its fresh package cache explicitly so an
inherited NUGET_PACKAGES setting cannot select another cache during SDK resolution.
In PowerShell:

```powershell
$env:NUGET_PACKAGES = Join-Path (Get-Location) '.packages'
```

Or in Bash:

```sh
export NUGET_PACKAGES="$PWD/.packages"
```

Then run the same commands on Windows or Linux:

```sh
dotnet restore Consumer.slnx --configfile NuGet.Config -warnaserror
dotnet build Consumer.slnx --no-restore --configuration Debug -warnaserror
dotnet src/App/bin/Debug/net10.0/App.dll
dotnet src/Host/bin/Debug/net10.0/Host.dll
```

The programs print `Hello, FPAS` and `Hello, C#`, respectively. Restore extracts
the SDK to `.packages/fpas.net.sdk/0.1.0-local.1`; builds use its packaged tools.
Use `--configuration Release` and `bin/Release/net10.0` for Release output.
The toolchain package's Debug/Release configuration and the consumer's build
configuration are independent.
Warnings are errors during restore as well: a missing SDK can otherwise cause
NuGet to skip the FPAS projects while restoring only the C# projects.

## Copy runnable output

Copy the complete `src/App/bin/Debug/net10.0` directory to a separate deployment
directory. Do the same for Host if deploying the C# consumer. Each directory
contains:

| Files | Role |
| --- | --- |
| App.dll or Host.dll | Executable managed assembly |
| Matching .runtimeconfig.json and .deps.json | Runtime selection and dependency resolution |
| Messages.dll, Shared.dll, Fpas.Core.dll | FPAS library, C# dependency, and matching metadata support |
| Matching .pdb files | Optional portable symbols for debugging |
| Matching .xml files | Optional API documentation enabled in this example |

Keep any additional runtime assets supplied by dependencies when copying the
directory. XML files describe public APIs; they are not runtime dependencies.
PDBs must match their assemblies, and source debugging also needs the corresponding
sources. See [compiled programs](../pascal/program-structure/compiled-programs.md).

From the App deployment directory, `dotnet App.dll` prints `Hello, FPAS`.
From Host's directory, `dotnet Host.dll` prints `Hello, C#`. Only the installed
.NET runtime and copied output are required at this point; the source tree,
local feed, SDK cache, and FPAS compiler are build inputs. The acceptance test
renames the consumer directory before these runs and uses an empty package cache.

This walkthrough verifies managed DLL deployment. For apphosts, Native AOT,
self-contained/single-file output and signing, use [application publishing](publishing.md).
The [VS Code extension](../../src/VSCode/README.md) can analyze and build restored
project contexts with this toolchain.

## Update the packaged compiler

Treat each package identity/version as immutable. After changing the compiler
or build task, build the toolchain again, pack with a new version such as
`-p:PackageVersion=0.1.0-local.2`, and copy that package to the consumer's feed.
Update the Sdk version in both FPAS project files, restore the solution, and
rebuild with the selected configuration:

```sh
dotnet restore Consumer.slnx --configfile NuGet.Config -warnaserror
dotnet build Consumer.slnx --no-restore --configuration Debug --no-incremental -warnaserror
```

Overwriting a .nupkg with the same identity does not replace an already extracted
SDK in the package cache. A new version avoids reusing old tools without clearing
unrelated packages. Keep NUGET_PACKAGES set to the consumer's cache during updates.

## See also

- [Toolchain development](development.md)
- [Project and dependency contract](msbuild.md)
- [Projects and solutions](../pascal/program-structure/projects.md)
- [Build acceptance](../../tests/coverage.md#build-acceptance)
