# Single-file compiler distribution

FPAS delivers one self-contained executable per supported target: `fpas.exe` for
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

Run [dist.ps1](../../dist.ps1) on Windows x64 or [dist.sh](../../dist.sh) on Linux
x64, with the .NET 10 SDK installed. Each script builds and packages the Release
SDK, then publishes the CLI with its matching package embedded. Builds use the
default SDK parallelism and separate `.temp-data/dist/windows` or
`.temp-data/dist/unix` artifacts. Invocation works from another directory.

Copy only `bin/fpas.exe` or `bin/fpas` to the destination. On Linux retain executable
permissions. No companion DLL, helper executable or runtimeconfig file is needed.
The scripts preserve unrelated files in `bin/`; they also write the matching
package to `sdk/feed/` and evict only that SDK version from the active NuGet cache.

Publication uses compressed, self-contained .NET single-file deployment without
trimming. Native AOT requires trimming and does not support the dynamic code used
by the complete protocol dependency graph, including Newtonsoft.Json and
OmniSharp/StreamJsonRpc. The release profile therefore preserves those dependencies
and their existing behavior. It uses ordinary in-memory managed bundling and native
runtime extraction, without `IncludeAllContentForSelfExtract` compatibility mode.
See Microsoft's [Native AOT restrictions](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/)
and [single-file deployment](https://learn.microsoft.com/en-us/dotnet/core/deploying/single-file/overview).

Optional compiler PDBs remain in the build artifacts, separate from the delivered
file. Generated applications retain their own [symbol contract](../architecture/debug-information.md).

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

The SDK includes its managed compiler for ordinary `dotnet build`. A distributed
`fpas build` selects its own executable for compiler subprocesses. To select it
explicitly from MSBuild, pass `-p:FpasCompilerPath=<absolute-path-to-fpas>`.
Both managed and executable invocations use the same JSON response contract.
The SDK supplies its compiler SHA-256; a different compiler is rejected before
compilation. Incremental fingerprints include the selected executable and the
SDK's compiler metadata. Check/query also require matching compiler bytes.

Offline builds work when the .NET SDK/reference packs and any additional project
packages are already available. Export itself performs no restore or download.
Application publishing is governed separately by the [SDK scope](msbuild.md#scope).

## Extraction and ownership

The .NET host extracts bundled native runtime libraries to its per-bundle cache.
`DOTNET_BUNDLE_EXTRACT_BASE_DIR` selects a different root; defaults are
`%TEMP%/.net` on Windows and `$HOME/.net` on Linux. Use a directory owned by the
account running FPAS. The host coordinates concurrent first starts.

FPAS materializes only file-based assets needed by metadata readers, MSBuild,
the debugger or SDK export. Its default cache root is
`%LOCALAPPDATA%/fpas/assets` on Windows and `$XDG_CACHE_HOME/fpas/assets` (otherwise
`$HOME/.cache/fpas/assets`) on Linux. `FPAS_ASSET_CACHE` overrides it with an absolute
path. Each asset has a SHA-256 directory; publication uses an atomic file move,
concurrent starts converge, and content is verified and repaired from the embedded
copy before reuse. These operations never download assets. Different versions
can coexist without replacing each other's files.

Both caches belong to the running account. They can be removed after its compiler,
debugger workers and MSBuild processes have exited; the next launch recreates
needed content. Removing a cache during active use is unsupported. `fpas env --json`
reports the actual executable, materialized assembly and selected shim paths/hashes.
It still reports missing .NET SDK inputs when project tools are unavailable.

The [debugger](../architecture/debugger-core.md) starts workers and target brokers
through the same delivered executable. DBI/DAC come from the target's .NET runtime;
debugging performs no downloads. `fpas licenses` prints the bundled debugger and .NET runtime notices;
the exported SDK also contains `tools/compiler/debugger-notices.txt`.

## Acceptance

`LocalDeploymentTests` runs the actual platform distribution script twice with an
isolated package cache. It copies only the executable into a fresh directory,
hides the build checkout, removes .NET host search paths, and checks concurrent
compilation, JSON failures, concurrent SDK export, conflict handling and asset-cache
repair. It builds through the matching SDK and explicit executable override,
rejects compiler mismatches, and exercises check/query, real stdio LSP, batch
debugging and DAP using the delivered executable. See [build acceptance](../../tests/coverage.md#build-acceptance).
