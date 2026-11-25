// [Documentation](../../../docs/architecture/workspaces.md#semantic-lint)
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;

namespace Fpas.Workspaces.Lint;

internal static class ShallowModules
{
    internal static IEnumerable<LintFinding> Find(string project, SemanticSnapshot snapshot, int maxInterfaceList, CancellationToken token)
    {
        var implementations = snapshot.Relations.Select(relation => relation.Implementation).ToHashSet(StringComparer.Ordinal);
        var routines = snapshot.Symbols.Where(symbol => symbol is { Kind: "routine", Declaration: not null, Context.Owner: not null }
            && !implementations.Contains(symbol.Id)).ToLookup(symbol => symbol.Context!.Owner!, StringComparer.Ordinal);
        var records = snapshot.Symbols.Where(symbol => symbol is { Kind: "record", Declaration: not null, Context.Owner: not null })
            .ToLookup(symbol => symbol.Context!.Owner!, StringComparer.Ordinal);
        foreach (var unit in snapshot.Symbols.Where(symbol => symbol is { Kind: "container", Declaration: not null }))
        {
            token.ThrowIfCancellationRequested();
            var shown = routines[unit.Id].Where(IsPublic).Select(routine => new[] { routine }).ToList();
            var total = routines[unit.Id].Count();
            foreach (var record in records[unit.Id])
            {
                total += routines[record.Id].Count();
                if (!IsPublic(record)) continue;
                var methods = routines[record.Id].Where(IsPublic).ToArray();
                if (methods.Length != 0) shown.Add(methods);
            }

            var exposed = shown.Sum(group => group.Length);
            if (exposed <= maxInterfaceList || exposed < total - exposed ||
                (shown.Count <= maxInterfaceList && shown.All(group => group.Length <= maxInterfaceList))) continue;

            yield return new("shallow-module", $"Unit '{unit.Name}' exposes {exposed} of its {total} routines through a wide interface.",
                project, unit.Declaration!, shown.SelectMany(group => group).OrderBy(routine => routine.Declaration!.Offset)
                    .Select(routine => LintFinding.Note($"Public routine '{routine.Context!.QualifiedName}'.", routine.Declaration!)).ToArray(),
                $"declaration:{unit.Name}");
        }
    }

    internal static IEnumerable<LintFinding> FindConsumers(string project, IReadOnlyList<(SemanticSymbol Target, SourceSpan Use)> references,
        IReadOnlyDictionary<string, SemanticSnapshot> snapshots, SourceQuery bindings, IReadOnlySet<string> testProjects, int maxExports, CancellationToken token)
    {
        var wideUses = new Dictionary<(string Caller, string Project, string Unit), Dictionary<string, (SemanticSymbol Export, SourceSpan Use)>>();
        foreach (var (target, use) in references)
        {
            token.ThrowIfCancellationRequested();
            if (target.Kind is "container" or "namespace" or "local" or "parameter" or "type_parameter" ||
                (target.Declaration is { } local && WorkspacePaths.Comparer.Equals(local.Path, use.Path))) continue;
            if (bindings.ResolveSource(project, target) is not { } definition || testProjects.Contains(definition.Project) ||
                WorkspacePaths.Comparer.Equals(use.Path, definition.Symbol.Declaration!.Path)) continue;

            var snapshot = snapshots[definition.Project];
            var export = definition.Symbol;
            while (export.Context?.Owner is { } owner && snapshot.Symbol(owner).Kind != "container")
                export = snapshot.Symbol(owner);
            if (!IsPublic(export) || export.Context?.Owner is not { } unitId) continue;

            var key = (use.Path, definition.Project, unitId);
            if (!wideUses.TryGetValue(key, out var learned)) wideUses.Add(key, learned = new(StringComparer.Ordinal));
            learned.TryAdd(export.Id, (export, use));
        }

        foreach (var (key, learned) in wideUses)
        {
            token.ThrowIfCancellationRequested();
            if (learned.Count <= maxExports) continue;
            var unit = snapshots[key.Project].Symbol(key.Unit);
            var uses = learned.Values.OrderBy(value => value.Use.Offset).ThenBy(value => value.Export.Context!.QualifiedName, StringComparer.Ordinal).ToArray();
            var notes = uses.Select(value => LintFinding.Note($"Uses '{value.Export.Context!.QualifiedName}' here.", value.Use))
                .Prepend(LintFinding.Note($"Unit '{unit.Name}' is declared here.", unit.Declaration!)).ToArray();
            yield return new("shallow-module", $"This source depends on {learned.Count} distinct exports from unit '{unit.Name}'. The limit is {maxExports}.",
                project, uses[0].Use, notes, $"consumer:{LintFinding.Relative(project, key.Caller)}::{LintFinding.Relative(project, key.Project)}::{unit.Name}");
        }
    }

    private static bool IsPublic(SemanticSymbol symbol) => symbol.Context?.Accessibility == "public";
}
