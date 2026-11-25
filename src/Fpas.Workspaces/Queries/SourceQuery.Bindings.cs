// [Documentation](../../../docs/architecture/workspaces.md#source-queries)
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Loading;

namespace Fpas.Workspaces.Queries;

/// <remarks>Checked editing shares canonical definition and binary-boundary mappings with source queries.</remarks>
public sealed partial class SourceQuery
{
    private readonly Dictionary<(string Project, string Symbol), (string Project, SemanticSymbol Symbol)?> sourceDefinitions = [];
    private readonly Dictionary<string, ILookup<string, SymbolExport>> exportsByProject = new(Paths);

    internal DefinitionResult Resolve(string project, string symbol) => Definition(project, projects[project].Snapshot.Symbol(symbol));
    internal string[] Boundaries() => BinaryBoundaries();

    internal (string Project, SemanticSymbol Symbol)? ResolveSource(string project, SemanticSymbol symbol)
    {
        if (symbol.Declaration is not null) return (project, symbol);
        if (symbol.Metadata is not { } metadata) return null;
        if (sourceDefinitions.TryGetValue((project, symbol.Id), out var cached)) return cached;

        var source = projects[project];
        var key = MetadataKey(metadata);
        (string Project, SemanticSymbol Symbol)? match = null;
        foreach (var output in analysis.Context.DependencyOutputs)
        {
            if (!projects.TryGetValue(output.Project, out var candidate) || candidate.Snapshot.AssemblyName != metadata.Assembly ||
                !source.Input.Request.References.Contains(output.Path, Paths)) continue;
            if (!exportsByProject.TryGetValue(output.Project, out var exports))
                exportsByProject.Add(output.Project, exports = candidate.Snapshot.Exports.ToLookup(export => MetadataKey(export.Identity), StringComparer.Ordinal));
            foreach (var export in exports[key])
            {
                var declaration = candidate.Snapshot.Symbol(export.Symbol);
                if (declaration.Declaration is null) continue;
                if (match is { } previous && (previous.Project != output.Project || previous.Symbol.Id != declaration.Id))
                    throw new ContextFailure("context_failed", "Reference metadata maps to conflicting source declarations.");
                match = (output.Project, declaration);
            }
        }
        sourceDefinitions.Add((project, symbol.Id), match);
        return match;
    }
}
