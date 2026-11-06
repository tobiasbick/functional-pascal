# FPAS Project SDK

Fpas.NET.Sdk packages Sdk/Sdk.props, Sdk/Sdk.targets, the matching compiler/CLI,
Fpas.Core metadata attributes, and the MSBuild task. The task runs the compiler in
a separate process. `FpasCompilerPath` accepts either the packaged managed CLI or
an absolute path to the distributed single-file executable. The SDK checks the
compiler content hash before compilation; a different compiler is rejected.
The CLI includes the stdio language server, its protocol dependencies and the
passive project SDK resolver. Keep tools/compiler intact when extracting it;
`dotnet tools/compiler/Fpas.Cli.dll lsp` uses the same server as `fpas lsp`.
FPAS emits CIL directly. Standard .NET SDK targets handle references and managed
runtime assets; there is no generated C# compilation stage.

The SDK supports multiple FPAS sources per project, net10.0,
AnyCPU, Debug/Release, and executable or library output. Executables contain one
program plus optional units; libraries contain units only. ProjectReference and
PackageReference work in mixed C#/FPAS solutions. Portable PDBs and optional XML
documentation propagate through ordinary output/reference copying. Embedded
resources and IDE integration remain outside the supported build scope.
Records, unions, domains, and public aliases use Fpas.Core metadata attributes.
The SDK supplies and copies its matching Core assembly. See the
[CLR mapping](../../docs/specs/clr-mapping.md) for the metadata contracts.

Application publishing supports Native AOT, self-contained/single-file outputs,
native apphosts and framework-dependent DLLs on Windows x64 and Linux x64.
Standard SignAssembly/AssemblyOriginatorKeyFile, PublicSign and DelaySign
properties configure strong names. FpasPublishCertificate signs a complete
delivery with a PFX certificate; FpasVerifyPublish checks it against a pinned
public certificate. See [publishing](../../docs/build/publishing.md) for modes,
native prerequisites, password environment variables and verification commands.

IsTestProject=true builds a unit-based library with dependency/runtimeconfig
files for VSTest. Use native framework packages/attributes and `dotnet test` or
`fpas test`; no generated C# test entry point is needed. See [native testing](../../docs/pascal/tools/testing.md).
Microsoft.Testing.Platform self-hosted mode is outside this contract.

Build the toolchain, pack this project to sdk/feed, and restore the example
against that local source. NuGet package versions are immutable; use a fresh
version/cache entry after changing packaged tools. No public package is published.

See the [development workflow](../../docs/build/development.md#build-an-fpas-application)
for commands, the [MSBuild contract](../../docs/build/msbuild.md) for configuration,
the [standalone SDK walkthrough](../../docs/build/standalone-sdk.md) for an
independent FPAS/C# consumer, and
[build acceptance](../../tests/coverage.md#build-acceptance) for verification.
