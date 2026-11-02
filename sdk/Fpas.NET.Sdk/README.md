# FPAS Project SDK

Fpas.NET.Sdk packages Sdk/Sdk.props, Sdk/Sdk.targets, the matching compiler/CLI,
Fpas.Core metadata attributes, and the MSBuild task. The task runs the compiler in a separate .NET process.
FPAS emits CIL directly. Standard .NET SDK targets handle references and managed
runtime assets; there is no generated C# compilation stage.

The initial implementation supports multiple FPAS sources per project, net10.0,
AnyCPU, Debug/Release, and executable or library output. Executables contain one
program plus optional units; libraries contain units only. ProjectReference and
PackageReference work in mixed C#/FPAS solutions. PDBs, XML documentation,
embedded resources, and IDE integration are pending.
Record/union declarations use Fpas.Core metadata attributes. The SDK supplies and
copies its matching Core assembly; outputs without records/unions have no emitted
Core assembly reference. See the [record milestone](../../docs/implementation/records.md)
and [unions/matching](../../docs/implementation/unions-and-matching.md).

Build the toolchain, pack this project to sdk/feed, and restore the example
against that local source. NuGet package versions are immutable; use a fresh
version/cache entry after changing packaged tools. No public package is published.

See the [implemented milestone](../../docs/implementation/msbuild.md) for commands
and limits, and the [MSBuild plan](../../docs/build/msbuild.md) for the full goal.
