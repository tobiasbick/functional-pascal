# FPAS distribution

The compiler distribution delivers one self-contained executable per supported target: `fpas.exe` for
Windows x64 (`win-x64`) and `fpas` for Linux x64 with glibc (`linux-x64`). Windows
and Ubuntu under WSL use their own native artifacts. Other operating systems and
architectures are outside this distribution contract.

The executable includes the .NET 10 runtime, compiler, workspace and language-server
dependencies, Core reference metadata, native debugger shim and matching project
SDK package. It can compile explicit sources without an installed .NET runtime,
compiler checkout, network access or adjacent support files. Compilation still
needs the target's managed reference assemblies. Project builds require the .NET
10 SDK, framework reference packs and the project's dependencies; running generated
applications requires their own .NET runtime and application dependencies.

## Build and deliver

Run [dist-small.ps1](../../dist-small.ps1) on Windows x64 or
[dist-small.sh](../../dist-small.sh) on Linux x64 to distribute the compiler.
Install the .NET 10 SDK before running either script. The scripts package the
Release SDK with the [native CUI assets](#native-cui-assets). They publish the CLI
with its matching package embedded. Builds use the
default SDK parallelism and separate `.temp-data/dist/windows` or
`.temp-data/dist/unix` artifacts. Invocation works from another directory.

Copy only `bin/fpas.exe` or `bin/fpas` to the destination. On Linux retain executable
permissions. No companion DLL, helper executable or runtimeconfig file is needed.
On Windows the first start extracts the compiler and its .NET runtime files
under `fpas-runtime/` beside the executable. On demand, FPAS creates
`fpas-assets/` within that directory for SDK and debugger assets that need real files.
Copying these directories to another installation is unnecessary.
The scripts preserve unrelated files in `bin/`. They write the matching SDK and
GUI binding packages to `sdk/feed/` and evict those versions from the active
NuGet cache. Before the eviction they stop the .NET build servers, whose reused
MSBuild nodes keep the cached build tasks loaded. A build server of another
program, such as an editor, can still hold them on Windows; close it and run
the script again.

The compiler payload uses compressed, self-contained .NET single-file
deployment without trimming. Native AOT requires trimming and does not support the dynamic code used
by the complete protocol dependency graph, including Newtonsoft.Json and
OmniSharp/StreamJsonRpc. The release profile therefore preserves those dependencies
and their existing behavior. It uses ordinary in-memory managed bundling and native
runtime extraction, without `IncludeAllContentForSelfExtract` compatibility mode.
See Microsoft's [Native AOT restrictions](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/)
and [single-file deployment](https://learn.microsoft.com/en-us/dotnet/core/deploying/single-file/overview).

Optional compiler PDBs remain in the build artifacts, separate from the delivered
file. Generated applications retain their own [symbol contract](../architecture/debug-information.md).

## Distribute the compiler and applications

Run [dist.ps1](../../dist.ps1) on Windows x64 or [dist.sh](../../dist.sh) on Linux
x64. The script builds the compiler distribution first. It then publishes the
two widget galleries, FPas-Sweeper and FPas-Amp with Native AOT. Both galleries
use their existing projects under `examples/ui/`.

| Application | Windows output | Linux output |
| --- | --- | --- |
| GUI widget gallery | `bin/GuiWidgetGallery.exe` | `bin/GuiWidgetGallery` |
| TUI widget gallery | `bin/TuiWidgetGallery.exe` | `bin/TuiWidgetGallery` |
| FPas-Sweeper | `bin/FpasSweeper.exe` | `bin/FpasSweeper` |
| FPas-Amp | `bin/FpasAmp.exe` | `bin/FpasAmp` |

The full distribution includes the CUI notices in `bin/licenses/cui/`.
For FPas-Amp on Windows, the portable launcher includes the AOT application and
its dependencies. On Linux, copy the shared libraries and the `tools/`,
`licenses/` and `share/` directories with `bin/FpasAmp`.
Radio on Linux requires `ffmpeg` on PATH. See the
[FPas-Amp distribution details](../../apps/fpas-amp/doc/desktop.md#windows-portable-build).

## Windows portable launcher

The Windows distribution wraps that compiler payload in a native launcher
with an embedded archive. [FPas-Amp](../../apps/fpas-amp/doc/desktop.md#windows-portable-build)
uses the same launcher with an AOT desktop payload.
At launch, the wrapper creates a content-addressed directory under
`<executable-name>-runtime/` beside the executable, verifies each payload
file against its embedded SHA-256, and restores missing or damaged files.
Concurrent launches coordinate extraction, and a completed file is moved
into place atomically. The child receives the caller's working directory,
arguments and standard streams, and the launcher returns its exit code.

The [Windows packaging script](../../src/Fpas.Deployment.WindowsPortable/Publish.ps1)
builds the archive and manifest. The [launcher project](../../src/Fpas.Deployment.WindowsPortable/Fpas.WindowsPortable.csproj)
embeds those resources using the selected console or GUI subsystem.
Pass `-Title` to set the GUI error dialog title. Either `-Title` or `-Icon`
selects the GUI subsystem. If you pass only `-Icon`, the dialog uses the
launcher executable's base name.
[Program.cs](../../src/Fpas.Deployment.WindowsPortable/Program.cs) starts the
bundled executable, and [Payload.cs](../../src/Fpas.Deployment.WindowsPortable/Payload.cs)
handles verification and extraction. The launcher requires a writeable
installation directory for first start and repair.

## Export the matching SDK

From any directory, export the embedded package to a local NuGet feed:

```sh
fpas sdk export ./feed
fpas init project Hello --sdk-feed ./feed
fpas build Hello/Hello.fpasproj
fpas run Hello/Hello.fpasproj --no-build
```

Export prints the package's absolute path. Repeating it preserves identical
content. A different package with the same filename fails without overwriting it;
`--force` explicitly replaces it. After replacing a development package, remove
that version's `fpas.net.sdk` cache entry before restoring consumers. Other package
versions remain independent. The [standalone SDK walkthrough](standalone-sdk.md)
describes a mixed-language consumer and isolated feed/cache configuration.

The SDK includes its managed compiler and [effect library](../pascal/libraries/effects.md)
for ordinary `dotnet build`. A distributed
`fpas build` selects its own executable for compiler subprocesses. To select it
explicitly from MSBuild, pass `-p:FpasCompilerPath=<absolute-path-to-fpas>`.
Both managed and executable invocations use the same JSON response contract.
The SDK supplies its compiler SHA-256; a different compiler is rejected before
compilation. Incremental fingerprints include the selected executable and the
SDK's compiler metadata. Check/query also require matching compiler bytes.

Offline builds work when the .NET SDK/reference packs and any additional project
packages are already available. Export itself performs no restore or download.
Application outputs use the [publishing contract](publishing.md).

## Native CUI assets

Distribution builds compile CUI from the pinned upstream sources using the
[native build prerequisites](../ui/gui/native-build.md#prerequisites).
The embedded SDK package carries the native files for the distribution's RID.

| SDK package path | `win-x64` | `linux-x64` |
| --- | --- | --- |
| `tools/gui/runtimes/<rid>/native/` | `fpas_cui.dll` | `libfpas_cui.so` |
| `tools/gui/runtimes/<rid>/static/` | `fpas_cui.lib` | `libfpas_cui.a` |
| `tools/gui/licenses/` | Six CUI and bundled-component notices | Same notices |

The scripts set `FpasCuiDistributionRid` for both the SDK build and pack. Packing
also creates `Fpas.Ui.Gui.Cui.<version>.nupkg` with the managed binding, the same
native files and transitive build targets. Supply this package alongside the SDK
in the local feed for GUI projects. Use separate feeds for distributions built
for different RIDs at the same version. The compiler and terminal programs have
no CUI runtime dependency. Ordinary compiler and SDK development builds can
omit the distribution property. See the [packaged binding](../ui/gui/cui.md#packaged-binding)
for project references and RID selection.

The application-facing Fpas.Ui.Gui package depends on Fpas.Ui.Mvu, Fpas.Ui and the
matching Fpas.Ui.Gui.Cui package. Package the three FPAS library projects from the
same compiler build into that feed after preparing its local SDK:

```sh
dotnet pack src/Fpas.Ui.Mvu/Fpas.Ui.Mvu.fpasproj -c Release -o artifacts/feed
dotnet pack src/Fpas.Ui/Fpas.Ui.fpasproj -c Release -o artifacts/feed
dotnet pack src/Fpas.Ui.Gui/Fpas.Ui.Gui.fpasproj -c Release -o artifacts/feed
```

Set PackageVersion consistently when creating a versioned release. The
[GUI publishing acceptance](../../tests/Fpas.Ui.Gui.Cui.Tests/Packaging/GuiPublishingTests.cs)
builds a fresh feed with these packages and consumes Fpas.Ui.Gui without source
references or native GUI build tools. Terminal-only applications use Fpas.Ui.Tui,
Fpas.Ui and Fpas.Ui.Mvu independently of that native binding.

## Extraction and ownership

The Windows launcher sets the compiler payload's .NET extraction root to
the `dotnet` subdirectory of its own versioned `fpas-runtime/` directory.
On Linux, the .NET host uses `$HOME/.net` by default and honors
`DOTNET_BUNDLE_EXTRACT_BASE_DIR`. Use installation and cache directories
owned by the account running FPAS.

FPAS materializes only file-based assets needed by metadata readers, MSBuild,
the debugger or SDK export. In a Windows distribution in `bin/` it uses
`bin/fpas-runtime/fpas-assets/<SHA-256>/`. On Linux, the default
`fpas-assets/` is next to the executable. `FPAS_ASSET_CACHE` selects another
writable root using an absolute path.
Each asset has a SHA-256 directory; publication uses an atomic file move,
concurrent starts converge, and content is verified and repaired from the embedded
copy before reuse. These operations never download assets. Different versions
can coexist without replacing each other's files.

Old hash directories are retained until removed explicitly. The Windows
compiler and .NET runtime are both restored under the installation directory.

Both caches belong to the running account. They can be removed after its compiler,
debugger workers and MSBuild processes have exited; the next launch recreates
needed content. Removing a cache during active use is unsupported. `fpas env --report json`
reports the running compiler executable, materialized assembly and selected shim paths/hashes.
It still reports missing .NET SDK inputs when project tools are unavailable.

The [debugger](../architecture/debugger-core.md) starts workers and target brokers
through the same delivered executable. DBI/DAC come from the target's .NET runtime;
debugging performs no downloads. `fpas licenses` prints the bundled debugger and .NET runtime notices;
the exported SDK also contains `tools/compiler/debugger-notices.txt`.

## Acceptance

`LocalDeploymentTests` runs the platform's `dist-small` script twice with an
isolated package cache. It copies only the executable into a fresh directory,
hides the build checkout, removes .NET host search paths, and checks eight parallel
first compilations, JSON failures, concurrent SDK export, conflict handling and asset-cache
repair. It verifies executable-local hash directories, recreation after cache removal
and an explicit cache-root override. It builds through the matching SDK and explicit
executable override, rejects compiler mismatches, publishes and certificate-signs an external application,
and exercises check/query, real stdio LSP, batch
debugging and DAP using the delivered executable. See [build acceptance](../../tests/coverage.md#build-acceptance).

## Source files

<details>
<summary>Browse 18 source files</summary>

- [dist-small.ps1](../../dist-small.ps1)
- [dist-small.sh](../../dist-small.sh)
- [dist.ps1](../../dist.ps1)
- [dist.sh](../../dist.sh)

- `sdk/Fpas.NET.Sdk/`
  - [Fpas.NET.Sdk.csproj](../../sdk/Fpas.NET.Sdk/Fpas.NET.Sdk.csproj)
  - [GuiAssets.targets](../../sdk/Fpas.NET.Sdk/GuiAssets.targets)
- `src/Fpas.Cli/Distribution/`
  - [Distribution.targets](../../src/Fpas.Cli/Distribution/Distribution.targets)
  - [SdkCommand.cs](../../src/Fpas.Cli/Distribution/SdkCommand.cs)
- `src/Fpas.Cli/`
  - [Fpas.Cli.csproj](../../src/Fpas.Cli/Fpas.Cli.csproj)
- `src/Fpas.Cli/Projects/`
  - [ToolSdk.cs](../../src/Fpas.Cli/Projects/ToolSdk.cs)
- `src/Fpas.Deployment/`
  - [DistributionAssets.cs](../../src/Fpas.Deployment/DistributionAssets.cs)
  - [Fpas.Deployment.csproj](../../src/Fpas.Deployment/Fpas.Deployment.csproj)
- `tests/Fpas.IntegrationTests/Build/`
  - [LocalDeploymentTests.Debugging.cs](../../tests/Fpas.IntegrationTests/Build/LocalDeploymentTests.Debugging.cs)
  - [LocalDeploymentTests.GuiAssets.cs](../../tests/Fpas.IntegrationTests/Build/LocalDeploymentTests.GuiAssets.cs)
  - [LocalDeploymentTests.LanguageServer.cs](../../tests/Fpas.IntegrationTests/Build/LocalDeploymentTests.LanguageServer.cs)
  - [LocalDeploymentTests.Publishing.cs](../../tests/Fpas.IntegrationTests/Build/LocalDeploymentTests.Publishing.cs)
  - [LocalDeploymentTests.SingleFile.cs](../../tests/Fpas.IntegrationTests/Build/LocalDeploymentTests.SingleFile.cs)
  - [LocalDeploymentTests.cs](../../tests/Fpas.IntegrationTests/Build/LocalDeploymentTests.cs)

</details>
