// [Documentation](../../../docs/architecture/workspaces.md#semantic-lint)
using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Analysis;

internal static class ImportUsage
{
    internal static IEnumerable<SemanticSymbol> UnusedClrAliases(SemanticSnapshot snapshot, CancellationToken token)
    {
        var used = new HashSet<string>(StringComparer.Ordinal);
        foreach (var occurrence in snapshot.Occurrences)
        {
            token.ThrowIfCancellationRequested();
            if (occurrence.Role != "declaration") used.Add(occurrence.Symbol);
        }

        foreach (var import in snapshot.Imports)
        {
            token.ThrowIfCancellationRequested();
            if (import.IsClr && !used.Contains(import.Alias)) yield return snapshot.Symbol(import.Alias);
        }
    }
}
