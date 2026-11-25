// [Documentation](../../../docs/architecture/workspaces.md#semantic-lint)
using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Lint;

internal static class ArchitectureReferences
{
    internal static IEnumerable<(SemanticSymbol Target, SourceSpan Use)> BoundUses(SemanticSnapshot snapshot, CancellationToken token)
    {
        var imports = snapshot.Imports.ToDictionary(import => import.Alias, import => import.Target, StringComparer.Ordinal);
        var headers = snapshot.Imports.Select(import => snapshot.Symbol(import.Alias).Declaration!)
            .GroupBy(span => span.Path, StringComparer.Ordinal)
            .ToDictionary(group => group.Key, group => group.Max(span => span.Offset + span.Length), StringComparer.Ordinal);
        foreach (var occurrence in snapshot.Occurrences)
        {
            token.ThrowIfCancellationRequested();
            var use = occurrence.Span;
            if (occurrence.Role == "declaration" || use.Offset < headers.GetValueOrDefault(use.Path)) continue;
            var symbol = snapshot.Symbol(imports.GetValueOrDefault(occurrence.Symbol, occurrence.Symbol));
            if (symbol.Declaration is not null || symbol.Metadata is not null) yield return (symbol, use);
        }
    }
}
