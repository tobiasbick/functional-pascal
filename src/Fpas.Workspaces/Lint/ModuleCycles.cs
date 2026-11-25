// [Documentation](../../../docs/architecture/workspaces.md#semantic-lint)
using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Lint;

internal static class ModuleCycles
{
    internal static IEnumerable<LintFinding> Find(string project, SemanticSnapshot snapshot,
        IReadOnlyList<(SemanticSymbol Target, SourceSpan Use)> references, CancellationToken token)
    {
        var units = snapshot.Symbols.Where(symbol => symbol is { Kind: "container", Declaration: not null })
            .ToDictionary(symbol => symbol.Declaration!.Path, symbol => symbol.Name, StringComparer.Ordinal);
        var graph = new ArchitectureGraph();
        foreach (var (target, use) in references)
        {
            token.ThrowIfCancellationRequested();
            if (target.Declaration is { } declaration &&
                units.TryGetValue(use.Path, out var from) && units.TryGetValue(declaration.Path, out var to))
                graph.Add(from, to, use);
        }

        foreach (var cycle in graph.Cycles(token))
        {
            var names = string.Join(", ", cycle.Order(StringComparer.Ordinal));
            var edges = graph.Edges(cycle).ToArray();
            yield return new("module-cycle", $"Cyclic dependencies between FPAS units: {names}.", project, edges[0].Use,
                edges.Select(edge => LintFinding.Note($"{edge.From} depends on {edge.To}.", edge.Use)).ToArray(), names,
                CycleEdges: edges.Select(edge => $"{edge.From} -> {edge.To}").ToArray());
        }
    }
}
