// [Documentation](../../../docs/architecture/workspaces.md#semantic-lint)
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;

namespace Fpas.Workspaces.Lint;

internal sealed class NamespaceCycles(IReadOnlyDictionary<string, SemanticSnapshot> snapshots, SourceQuery bindings)
{
    private sealed record Unit(string Name, string[] Namespace);
    private sealed record Witness(string Project, string FromUnit, string ToUnit);

    private readonly Dictionary<string, Dictionary<string, Unit>> units = snapshots.ToDictionary(project => project.Key,
        project => project.Value.Symbols.Where(symbol => symbol is { Kind: "container", Declaration: not null })
            .ToDictionary(symbol => symbol.Declaration!.Path,
                symbol => new Unit(symbol.Name, symbol.Name.Split('.')[..^1]), WorkspacePaths.Comparer), WorkspacePaths.Comparer);
    private readonly ArchitectureGraph graph = new();
    private readonly Dictionary<(string From, string To), Witness> witnesses = [];

    internal void Add(string project, IReadOnlyList<(SemanticSymbol Target, SourceSpan Use)> references, CancellationToken token)
    {
        var sourceUnits = units[project];
        foreach (var (target, use) in references)
        {
            token.ThrowIfCancellationRequested();
            if (!sourceUnits.TryGetValue(use.Path, out var from) || from.Namespace.Length == 0 ||
                (target.Declaration is { } local && WorkspacePaths.Comparer.Equals(local.Path, use.Path))) continue;
            if (bindings.ResolveSource(project, target) is not { } declaration ||
                !units[declaration.Project].TryGetValue(declaration.Symbol.Declaration!.Path, out var to) ||
                Siblings(from.Namespace, to.Namespace) is not { } edge || !graph.Add(edge.From, edge.To, use)) continue;
            witnesses.Add(edge, new(project, from.Name, to.Name));
        }
    }

    internal IEnumerable<LintFinding> Find(CancellationToken token)
    {
        foreach (var cycle in graph.Cycles(token))
        {
            var names = string.Join(", ", cycle.Order(StringComparer.Ordinal));
            var edges = graph.Edges(cycle).ToArray();
            var first = edges[0];
            yield return new("namespace-cycle", $"Cyclic dependencies between sibling FPAS namespaces: {names}.",
                witnesses[(first.From, first.To)].Project, first.Use, edges.Select(edge =>
                {
                    var witness = witnesses[(edge.From, edge.To)];
                    return LintFinding.Note($"{edge.From} depends on {edge.To} through {witness.FromUnit} and {witness.ToUnit}.", edge.Use);
                }).ToArray(), names, CycleEdges: edges.Select(edge => $"{edge.From} -> {edge.To}").ToArray());
        }
    }

    private static (string From, string To)? Siblings(string[] from, string[] to)
    {
        var shared = 0;
        while (shared < from.Length && shared < to.Length && from[shared] == to[shared]) shared++;
        if (shared == from.Length || shared == to.Length) return null;
        return (string.Join('.', from.AsSpan(0, shared + 1)), string.Join('.', to.AsSpan(0, shared + 1)));
    }
}
