# Development workflow

Use a stable .NET 10 SDK satisfying global.json. Package versions are centralized
in Directory.Packages.props. Commands below run from the repository root.
Use the SDK's default build parallelism unless the user requests a limit.

## Build the compiler and tests

Restore, compile, and execute tests as separate steps:

```sh
dotnet restore Fpas.slnx
dotnet build Fpas.slnx --no-restore --configuration Debug -warnaserror
dotnet test Fpas.slnx --no-build --no-restore --configuration Debug
```

Debug is the development configuration. Use Release for release validation and
controlled performance measurements:

```sh
dotnet build Fpas.slnx --no-restore --configuration Release -warnaserror
dotnet test Fpas.slnx --no-build --no-restore --configuration Release
```

Do not rebuild toolchain binaries while tests are using them. Test projects copy
examples and documentation during their build; rebuild after changing those
inputs. Integration tests package the already built SDK with a unique version
and execute isolated FPAS/C# consumers. They also build their own fixture projects;
this is part of the test, not a second toolchain build.

Choose relevant tests through the checked-in runner. For the handbook and agent
reference:

```sh
dotnet test tests/Fpas.IntegrationTests/Fpas.IntegrationTests.csproj --no-build --no-restore --configuration Debug --filter 'FullyQualifiedName~HandbookTests|FullyQualifiedName~AgentReferenceTests|FullyQualifiedName~LanguageSpecificationTests' --logger 'console;verbosity=normal'
```

AgentReferenceTests checks links, the entrypoint policy, and exact o200k_base
token counts. LanguageSpecificationTests compiles complete programs/units from
the repository README and handbook and executes programs. Its assertions cover
each example's behavior.
HandbookTests checks topic hubs, related navigation, repository documentation
links/anchors, explicit syntax fragments, and current copied inputs.
Read [tests](../../tests/README.md) and the
[acceptance map](../../tests/coverage.md) when selecting
coverage for compiler, metadata, interop, async, or build changes.

Store local logs, scratch work, and performance data under `.temp-data/`.
Check whitespace with `git diff --check`. Keep generated bin/obj output out of Git.

## Build an FPAS application

The local SDK package must exist before a consumer can restore its SDK:

```sh
dotnet pack sdk/Fpas.NET.Sdk/Fpas.NET.Sdk.csproj --no-build --configuration Debug --output sdk/feed
dotnet restore examples/hello/Hello.fpasproj
dotnet build examples/hello/Hello.fpasproj --no-restore
dotnet examples/hello/bin/Debug/net10.0/Hello.dll
```

The current SDK supports net10.0/AnyCPU and managed libraries/executables. It
resolves framework, project, package, and Core support references. Executables
run through dotnet; no native apphost or IDE extension is required.

NuGet caches SDK packages by identity/version. After changing packaged tools,
use a new development package version or remove only that SDK version from the
local cache. For example, pack with `-p:PackageVersion=0.1.0-dev.2` and update
the consumer's SDK version to match. To locate the cache, use
`dotnet nuget locals global-packages --list`; remove only the selected
`fpas.net.sdk/<version>` directory when intentionally reusing that version.
Do not clear unrelated packages. See [MSBuild](msbuild.md) for the project contract.
For a consumer outside this checkout, follow the complete
[standalone SDK walkthrough](standalone-sdk.md). It includes an isolated local
feed/cache, FPAS and C# projects, and copying runnable output.

## Direct compiler invocation

Use compile help to inspect current arguments:

```sh
dotnet src/Fpas.Cli/bin/Debug/net10.0/Fpas.Cli.dll compile --help
```

Set FPAS_REFERENCE_DIR to the installed framework reference directory, with
layout `<dotnet-install>/packs/Microsoft.NETCore.App.Ref/<version>/ref/net10.0`.
Compile and execute one source:

```sh
dotnet src/Fpas.Cli/bin/Debug/net10.0/Fpas.Cli.dll compile examples/hello/main.fpas --output .temp-data/hello/Hello.dll --reference-dir "$FPAS_REFERENCE_DIR" --diagnostics json
dotnet .temp-data/hello/Hello.dll
```

Pass multiple sources in one invocation; use `--target library` for units-only
output. References are explicit DLLs/directories, not assemblies discovered in
the compiler's process. The CLI adds its matching Core metadata reference.
Direct deployments must supply their non-framework runtime dependencies; prefer
the project SDK for normal applications. Portable PDBs are emitted by default;
XML documentation is optional. See the
[CLI contract](../pascal/program-structure/cli.md) for stdin,
response files, output publication, and structured diagnostic/exit behavior.

## Compiler work and runtime inspection

Follow the [repository layout](../architecture/project-layout.md) for project
boundaries. Language rules live in the [handbook](../pascal/README.md) and
[grammar](../specs/syntax.md). Public representations follow
[CLR mapping](../specs/clr-mapping.md); dependency/build behavior follows
[MSBuild](msbuild.md). Read the affected [architecture](../architecture/README.md)
and [tests](../../tests/coverage.md) before changing or claiming support.
[Language gaps](../future/language-gaps.md) separates missing
baseline work from deferred extensions; record practical authoring limits in
[dogfooding gaps](../future/dogfooding-gaps.md).

Use supported .NET debuggers/profilers for generated assemblies. Portable symbols
provide source locations, locals, and closure/async metadata; see
[debug information](../architecture/debug-information.md).
Investigate actual program behavior before writing benchmark variants or external
samplers. The current FPAS CLI provides compilation; formatter, LSP/query tools,
and automatic IDE integration have their separately documented scope.
