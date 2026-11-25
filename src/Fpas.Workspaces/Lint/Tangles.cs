// [Documentation](../../../docs/architecture/workspaces.md#semantic-lint)
using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Lint;

internal static class Tangles
{
    internal static IEnumerable<LintFinding> Find(string project, SemanticSnapshot snapshot,
        IReadOnlyList<(SemanticSymbol Target, SourceSpan Use)> references, int maxLocalGroup, CancellationToken token)
    {
        var importAliases = snapshot.Imports.Select(import => import.Alias).ToHashSet(StringComparer.Ordinal);
        var entities = snapshot.Symbols.Where(symbol =>
            symbol is { Declaration: not null, Context.Owner: { } owner, Kind: "routine" or "record" or "variant" or "interface" or "domain" or "global" or "alias" } &&
            snapshot.Symbol(owner).Kind == "container" && !importAliases.Contains(symbol.Id))
            .ToDictionary(symbol => symbol.Id, StringComparer.Ordinal);
        var byFile = entities.Values.GroupBy(symbol => symbol.Declaration!.Path, StringComparer.Ordinal)
            .ToDictionary(group => group.Key, group => group.OrderBy(symbol => symbol.Context!.Span.Offset).ToArray(), StringComparer.Ordinal);
        var owningEntities = new Dictionary<string, string?>(StringComparer.Ordinal);
        var graph = new ArchitectureGraph();
        foreach (var uses in references.GroupBy(reference => reference.Use.Path, StringComparer.Ordinal))
        {
            if (!byFile.TryGetValue(uses.Key, out var scopes)) continue;
            var index = 0;
            foreach (var (target, use) in uses)
            {
                token.ThrowIfCancellationRequested();
                while (index < scopes.Length && End(scopes[index]) <= use.Offset) index++;
                if (index == scopes.Length) break;
                if (target.Declaration is not null && scopes[index].Context!.Span.Contains(use.Offset) && Owner(target) is { } entity)
                    graph.Add(scopes[index].Id, entity, use);
            }
        }

        foreach (var cycle in graph.Cycles(token))
        {
            var unitCount = cycle.Select(id => entities[id].Declaration!.Path).Distinct(StringComparer.Ordinal).Count();
            if (unitCount == 1 && cycle.Count <= maxLocalGroup) continue;
            var members = string.Join(", ", cycle.Select(Name).Order(StringComparer.Ordinal));
            var edges = graph.Edges(cycle).OrderBy(edge => Name(edge.From), StringComparer.Ordinal)
                .ThenBy(edge => Name(edge.To), StringComparer.Ordinal).ToArray();
            yield return new("tangle", $"{cycle.Count} declarations depend on each other across {unitCount} FPAS unit(s): {members}.",
                project, edges[0].Use, edges.Select(edge => LintFinding.Note($"{Name(edge.From)} depends on {Name(edge.To)}.", edge.Use)).ToArray(),
                string.Join(", ", cycle.Select(Identity).Order(StringComparer.Ordinal)),
                CycleEdges: edges.Select(edge => $"{Identity(edge.From)} -> {Identity(edge.To)}").Order(StringComparer.Ordinal).ToArray());
        }

        static int End(SemanticSymbol symbol) => symbol.Context!.Span.Offset + symbol.Context.Span.Length;
        string Name(string id) => entities[id].Context!.QualifiedName;
        string Identity(string id) => entities[id].Metadata?.GetKey(includeAssemblyHash: false) ?? Name(id);
        string? Owner(SemanticSymbol symbol)
        {
            if (owningEntities.TryGetValue(symbol.Id, out var existing)) return existing;
            var current = symbol;
            while (!entities.ContainsKey(current.Id) && current.Context?.Owner is { } owner)
                current = snapshot.Symbol(owner);
            var found = entities.ContainsKey(current.Id) ? current.Id : null;
            owningEntities.Add(symbol.Id, found);
            return found;
        }
    }
}
