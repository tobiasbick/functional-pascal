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
and execute isolated FPAS/C# consumers. Project preparation and design-time
builds preserve staged documentation; stale-fixture cleanup runs with actual
output copying. Integration tests also build their own fixture projects;
this is part of the test, not a second toolchain build.

Choose relevant tests through the checked-in runner. For the handbook and agent
reference:

```sh
dotnet test tests/Fpas.IntegrationTests/Fpas.IntegrationTests.csproj --no-build --no-restore --configuration Debug --filter 'FullyQualifiedName~HandbookTests|FullyQualifiedName~AgentReferenceTests|FullyQualifiedName~LanguageSpecificationTests|FullyQualifiedName~DocExamplesBehaviorTests' --logger 'console;verbosity=normal'
```

AgentReferenceTests checks links, the entrypoint policy, and exact o200k_base
token counts. The [FPAS documentation checker](../../apps/doc-examples/) builds
through a fresh packaged SDK, compiles complete sources/fragments from Markdown
and supplies assemblies to LanguageSpecificationTests. Existing execution and
metadata assertions remain there. DocExamplesBehaviorTests verifies the actual
checker CLI, rejected inputs, expected failures and process cleanup.
HandbookTests checks topic hubs, related navigation, repository documentation
links/anchors, explicit syntax fragments, and current copied inputs.
Read [tests](../../tests/README.md) and the
[acceptance map](../../tests/coverage.md) when selecting
coverage for compiler, metadata, interop, async, or build changes.

Store local logs, scratch work, and performance data under `.temp-data/`.
Check whitespace with `git diff --check`. Keep generated bin/obj output out of Git.

### Native Windows CLI acceptance

After the Debug build, run the platform and CLI contracts from native Windows:

```powershell
dotnet test tests/Fpas.IntegrationTests/Fpas.IntegrationTests.csproj --no-build --no-restore --configuration Debug --filter 'FullyQualifiedName~.Projects.|FullyQualifiedName~.Processes.|FullyQualifiedName~.Testing.|FullyQualifiedName~.ApiDocumentation.|FullyQualifiedName~.Refactoring.|FullyQualifiedName~.Editing.|FullyQualifiedName~.Formatting.|FullyQualifiedName~.Diagnostics.|FullyQualifiedName~AnalysisSignalTests' --logger 'trx;LogFileName=windows-cli.trx' --results-directory .temp-data/windows-cli
```

The [acceptance map](../../tests/coverage.md#native-windows-cli-acceptance) describes
the observable contracts. Console tests use real ConPTY sessions on Windows 10
version 1809 or newer, with no visible window or external terminal dependency.
They check target exit before closing their console/job, so harness cleanup
cannot satisfy a CLI cleanup assertion. Linux retains SIGINT/process-group tests.

## Deploy the local Release compiler

Run [dist.ps1](../../dist.ps1) on Windows or [dist.sh](../../dist.sh) on Unix:

```powershell
.\dist.ps1
```

```sh
./dist.sh
```

Both scripts restore/build the CLI and project SDK together in Release, publish
the launcher and its dependencies to the ignored `bin/` directory, and pack the
matching SDK into `sdk/feed/`. They resolve paths relative to the script, so
invocation also works from another working directory. Build artifacts stay under
`.temp-data/dist/`, separate from ordinary toolchain/test builds.

Add that `bin/` directory to PATH once, then use `fpas --version`, `fpas --help`,
or `fpas compile --help` from any directory. The scripts do not change PATH.
Windows uses `bin/fpas.exe`; Unix uses `bin/fpas`. `dist.sh` is executable in Git
and makes the deployed launcher executable. Repeated deployment updates the
published files and preserves unrelated files in `bin/`.

The same launcher serves `fpas lsp`; its managed protocol dependencies and the
passive SDK resolver are included in both `bin/` and the matching SDK's
`tools/compiler/`. Packing rejects missing language-server assets. Run `fpas env`
to inspect the deployed server, then explicitly restore the consumer before
connecting a client. See the [language-server contract](../pascal/tools/language-server.md).

Building requires the .NET 10 SDK; running the deployed compiler requires the
.NET 10 runtime. Keep the published files together. This local managed deployment
does not close the [single-file/Native AOT distribution gaps](../future/dist-gap.md).

Check/query require the CLI and the project's SDK to contain the same compiler
binary. Dist builds both from the same artifacts directory and reads the package
version from the SDK project (currently `0.1.0-dev`). It removes only that version's
`fpas.net.sdk` entry from the active NuGet global-packages cache, so the next
consumer restore/SDK resolution selects the new package. Other versions/packages
remain cached. No separate SDK build, pack or cache-clear command is needed after
dist. Consumers must select this SDK version and have the local feed configured;
the checkout's examples already do.

## Build an FPAS application

After running dist, restore/build an application:

```sh
dotnet restore examples/hello/Hello.fpasproj
fpas build examples/hello/Hello.fpasproj --no-restore
fpas run examples/hello/Hello.fpasproj --no-build
```

The current SDK supports net10.0/AnyCPU and managed libraries/executables. It
resolves framework, project, package, and Core support references. Executables
run through dotnet; no native apphost or IDE extension is required.
The [project CLI](../pascal/program-structure/project-commands.md) defines templates,
source/solution inputs, selection, streams and structured reports. Ordinary
`dotnet build` remains the underlying MSBuild workflow for mixed .NET tools.

For manual SDK packaging outside dist, NuGet caches packages by identity/version.
After changing packaged tools, use a new version or remove only that SDK version
from the local cache. For example, pack with `-p:PackageVersion=0.1.0-dev.2` and
update the consumer's SDK version to match. To locate the cache, use
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
samplers. The FPAS CLI provides compilation, evaluated project checks/source queries,
[API documentation lookup](../pascal/tools/api-documentation.md) and
[batch and DAP debugging](../pascal/tools/debugging.md).
Use [fpas fmt](../pascal/tools/fmt-style.md) to format sources or check the fixed
style without changes. LSP and automatic IDE project integration remain separate
work.

For FPAS-authored tests, use native framework attributes and assertions in a test
library. [Native .NET testing](../pascal/tools/testing.md) describes `fpas init test`,
`fpas test` and ordinary `dotnet test`, with filters and structured native results.
