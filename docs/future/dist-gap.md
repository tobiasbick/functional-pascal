# CLR distribution gaps

The release goal is one delivered `fpas.exe` on Windows and one `fpas` per
supported Unix platform/architecture, preferably Native AOT. Required compiler
DLLs, support files or helper executables beside it would not meet this goal.
This concerns the compiler tool; application outputs and .NET/MSBuild build
inputs retain their own requirements. No publish mechanism is selected yet.

## Open work

| Capability | Required work |
| --- | --- |
| Release workflow | Produce the single-executable artifact beyond the managed [local deployment](../build/development.md#deploy-the-local-release-compiler). Retain the `fpas` tool name. |
| Native AOT | Configure and verify publication; evaluate trimming, warnings, dependency compatibility and native build prerequisites. If AOT is unsuitable, evaluate another single-file deployment mode. |
| Platforms | Select supported OS/architecture targets and verify each target-specific artifact. |
| Assets | Define embedded/provisioned compiler, Core metadata and SDK assets, separate optional symbols, version selection, offline availability and cache ownership. |
| SDK invocation | Let the [SDK](../../sdk/Fpas.NET.Sdk/Sdk/Sdk.props) and [build task](../../src/Fpas.Build.Tasks/FpasCompile.cs) select/invoke the distributed compiler with a matching version. |
| Release verification | Compile, report diagnostics and use the matching project SDK from an isolated delivered executable, without a compiler checkout or adjacent compiler DLLs. |

## Concrete compatibility obstacles

- The [response reader](../../src/Fpas.Cli/Compilation/CompileResponseFile.cs)
  uses reflection-based deserialization. Supply an AOT-compatible contract.
  Compile/check/query diagnostics already use source-generated JSON contracts;
  evaluate the remaining serialization and dependency graph during publication.
- The compiler locates Core metadata with
  `typeof(Core.RecordAttribute).Assembly.Location`. A bundled/native executable
  needs another way to supply that managed metadata. Native-linking Core does
  not provide a reference assembly for compiling FPAS sources.
- Evaluate the complete CLI dependency graph, including workspace/MSBuild and MCP
  hosting dependencies, under the selected publish mode. Inspecting CIL emission
  alone cannot establish AOT compatibility.

Embedding managed assets and reading them from memory or materializing them in a
cache are options. The SDK's compiler launch and matching support/package assets
must work without requiring loose files beside the tool. Generated applications
may still need their own managed support assembly.

Project/solution builds still need the .NET SDK, framework references, package and
project dependencies, SDK props/targets and build tasks. Native AOT removes only
the compiler process's .NET runtime requirement.

## Scope and references

Application publishing belongs in [CLI gaps](cli-gap.md); language-server
functionality belongs in [editor gaps](editor-gap.md). No additional runner,
benchmark executable or source-standard-library distribution is required.

- [Native AOT restrictions](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/)
- [JSON source generation](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/source-generation)
- [Single-file API limitations](https://learn.microsoft.com/en-us/dotnet/core/deploying/single-file/overview)
