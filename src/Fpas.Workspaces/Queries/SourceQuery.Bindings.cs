using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Queries;

public sealed partial class SourceQuery
{
    internal DefinitionResult Resolve(string project, string symbol) => Definition(project, projects[project].Snapshot.Symbol(symbol));
    internal string[] Boundaries() => BinaryBoundaries();
}
