// [Documentation](../../../docs/architecture/workspaces.md#source-queries)
using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Queries;

/// <remarks>Checked editing shares canonical definition and binary-boundary mappings with source queries.</remarks>
public sealed partial class SourceQuery
{
    internal DefinitionResult Resolve(string project, string symbol) => Definition(project, projects[project].Snapshot.Symbol(symbol));
    internal string[] Boundaries() => BinaryBoundaries();
}
