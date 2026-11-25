// [Documentation](../../../docs/architecture/workspaces.md#semantic-lint)
using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Lint;

internal static class FieldLeaks
{
    internal static IEnumerable<LintFinding> Find(string project, SemanticSnapshot snapshot, CancellationToken token)
    {
        if (snapshot.PropertyReaches.Count == 0) yield break;
        var fields = snapshot.Symbols.Where(symbol => symbol is { Kind: "property", Declaration: not null, Context.Owner: not null })
            .ToLookup(symbol => symbol.Context!.Owner!, StringComparer.Ordinal);
        var methods = snapshot.Symbols.Where(symbol => symbol is { Kind: "routine", Context: { IsInstance: true, Owner: not null } })
            .ToLookup(symbol => symbol.Context!.Owner!, StringComparer.Ordinal);
        var uses = snapshot.Occurrences.Where(occurrence => occurrence.Role != "declaration")
            .ToLookup(occurrence => occurrence.Symbol, StringComparer.Ordinal);
        var reaches = snapshot.PropertyReaches.ToLookup(reach => reach.Property, StringComparer.Ordinal);
        var types = snapshot.Types.ToDictionary(type => type.Id, StringComparer.Ordinal);

        foreach (var record in snapshot.Symbols.Where(symbol => symbol is { Kind: "record", Context: not null }))
        {
            token.ThrowIfCancellationRequested();
            var state = fields[record.Id];
            var instanceMethods = methods[record.Id];
            if (!state.Any(field => field.Context!.Accessibility == "private") ||
                !state.Any(field => uses[field.Id].Any(use => instanceMethods.Any(method => Inside(method.Context!.Span, use.Span))))) continue;

            foreach (var field in state)
            {
                if (field.Context!.Accessibility != "public" || field.Type is null ||
                    types[field.Type] is not { IsReference: true, Callable: null } type || type.Metadata?.Type == "System.String") continue;
                var outside = reaches[field.Id].Select(reach => reach.Span).Where(use => !Inside(record.Context!.Span, use)).ToArray();
                if (outside.Length == 0) continue;

                var notes = outside.Select(use => LintFinding.Note($"Code reaches through '{field.Name}' here.", use))
                    .Prepend(LintFinding.Note("The record field is declared here.", field.Declaration!)).ToArray();
                yield return new("field-leak", $"Code outside '{record.Context!.QualifiedName}' works through its field '{field.Name}'.",
                    project, outside[0], notes, field.Context.QualifiedName);
            }
        }
    }

    private static bool Inside(SourceSpan outer, SourceSpan inner) => outer.Path == inner.Path && outer.Contains(inner.Offset);
}
