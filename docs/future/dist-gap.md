# CLR distribution gaps

This document records the missing CLR distribution workflow. It is a gap
inventory, not an implementation plan. It does not authorize implementation,
schedule work, select a packaging mechanism, or change FPAS syntax.

The distribution goal is one delivered `fpas.exe` on Windows, preferably built
with Native AOT. The corresponding Unix artifact is one `fpas` executable per
supported platform and architecture. Separate compiler DLLs, runtime configuration
files, support DLLs, or helper executables beside it would not meet this goal.

This goal concerns the distributed FPAS tool. It does not restrict the files in
compiled application outputs or remove the build dependencies described below.
Native AOT is preferred; the single-file goal remains if an AOT limitation requires
evaluating another deployment mode.

## Rust comparison and current CLR support

The Rust [Windows script](https://github.com/tobiasbick/functional-pascal/blob/0a4bc134ee71b5f922ebd0b1157ab55244bef657/dist.ps1)
and [Linux script](https://github.com/tobiasbick/functional-pascal/blob/0a4bc134ee71b5f922ebd0b1157ab55244bef657/dist.sh)
build `fpas`, `fpas-runner`, and `fpas-lsp`, and stage standard-library sources
under `bin/lib`. They do not build or copy the benchmark tool.

The CLR [CLI project](../../src/Fpas.Cli/Fpas.Cli.csproj) currently builds a managed
executable referencing the compiler and Fpas.Core projects. No Native AOT or
single-file publish configuration is present. Equivalent distribution entry
points for Windows and Linux are missing.

The existing [FPAS SDK package](../../sdk/Fpas.NET.Sdk/Fpas.NET.Sdk.csproj) contains
the managed compiler, its host/dependency files, Fpas.Core, MSBuild tasks, and SDK
props/targets. The [standalone SDK walkthrough](../build/standalone-sdk.md) verifies
managed deployment, not the proposed single-executable distribution.

The compiler reads reference assemblies with
[PEReader](../../src/Fpas.Compiler/Binding/References/ReferenceAssembly.cs)
and emits CIL/metadata through
[ManagedPEBuilder](../../src/Fpas.Compiler/Emission/Metadata/AssemblyEmitter.cs).
Writing a managed assembly to disk does not require executing newly generated
code inside the compiler. Source inspection found no fundamental Native AOT
blocker in that design. A successful native publish has not been demonstrated.

## Missing distribution capabilities

| Capability | CLR gap |
| --- | --- |
| Distribution entry points | No equivalent workflow producing the intended Windows or Linux release artifact from the current CLR sources. Script/helper implementation and naming remain open under repository dogfooding rules. |
| Executable identity | No distribution setting establishing `fpas.exe` or `fpas` as the published tool name. |
| Native AOT publication | No configured or verified Native AOT publish path for the CLI and compiler dependencies. Trimming, AOT warnings, and native build prerequisites still need evaluation. |
| Platform artifacts | No defined supported OS/architecture set or verified per-target outputs. Native AOT produces target-specific binaries, not one universal executable. |
| Single-file contents | No packaging contract that includes required FPAS assets while leaving only the executable in the delivered tool directory. Optional debug symbols need separate handling rather than becoming mandatory adjacent files. |
| Release verification | No evidence that an isolated distributed executable can compile sources, emit valid diagnostics, and work with the project SDK without relying on the compiler checkout or adjacent compiler DLLs. |

## Concrete AOT and single-file obstacles

The current [CLI implementation](../../src/Fpas.Cli/Compilation/CompilerCommand.cs)
and [response-file reader](../../src/Fpas.Cli/Compilation/CompileArguments.cs)
contain specific compatibility gaps:

- JSON diagnostics use reflection-based serialization of an anonymous report,
  and response files use reflection-based deserialization. An AOT-compatible
  JSON contract is missing. Source-generated metadata is a supported option.
- The CLI uses the non-generic `JsonStringEnumConverter`, which Native AOT does
  not support. An AOT-compatible representation/converter remains to be selected.
- The CLI obtains the compiler's Fpas.Core reference from
  `typeof(Core.RecordAttribute).Assembly.Location`. A bundled/native tool cannot
  assume that a separate managed assembly exists at that location.

Microsoft documents [Native AOT restrictions](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/),
[JSON source generation](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/source-generation),
and [single-file API limitations](https://learn.microsoft.com/en-us/dotnet/core/deploying/single-file/overview).
These provide available mechanisms, not proof that the current FPAS executable
already works in either publish mode.

Fpas.Core is a small metadata-contract assembly, not the former standard-library
catalog. Its managed metadata must remain available when compiling FPAS sources;
native-linking its implementation into the compiler does not supply that input.
Embedding the managed assembly and reading it from memory or materializing it
into a cache are possible approaches. Generated applications may also need the
matching managed support assembly in their own outputs. The single-file tool
goal does not remove those requirements.

## Project SDK and external build dependencies

The [SDK props](../../sdk/Fpas.NET.Sdk/Sdk/Sdk.props) select `Fpas.Cli.dll`, and the
[build task](../../src/Fpas.Build.Tasks/FpasCompile.cs) starts it through `dotnet`.
Support for invoking the native executable is missing. SDK packaging and compiler
selection also need to remain consistent with the distributed compiler version.

Building `.fpasproj` projects and mixed `.slnx` solutions still requires the .NET
SDK, framework reference assemblies, package/project references, and FPAS SDK
props/targets and build tasks. Native AOT can remove the compiler process's need
for an installed .NET runtime; it does not replace MSBuild or those build inputs.

The single-file distribution needs a way to make matching FPAS SDK/support assets
available without requiring users to copy loose files beside `fpas.exe`. Embedded
assets and package/cache provisioning are possible building blocks. Acquisition,
version selection, offline availability, and cache location remain open questions.

## Intentionally omitted and separate work

- `fpas-bench` and `--std-lib` remain intentionally omitted, as recorded in
  [CLI gaps](cli-gap.md). They are not distribution gaps.
- The old `lib` source tree and a replacement `Std.*` wrapper catalog are not
  distribution requirements. Fpas.Core remains relevant as described above.
- A separate `fpas-runner` is unnecessary because the compiler emits ordinary
  CLR programs. This follows from CLR output, independently of compiler AOT.
- A separate `fpas-lsp` executable is not the distribution goal. A future language
  server could run in another mode of the same executable. Its functionality is
  still missing and tracked in [editor gaps](editor-gap.md); no mode syntax is
  selected here.
- Native AOT for the compiler does not make generated programs native. Publishing
  user applications, including their own AOT compatibility, remains separate work
  in [CLI gaps](cli-gap.md) and outside the current [MSBuild scope](../build/msbuild.md).
