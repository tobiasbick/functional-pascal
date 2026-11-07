using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Refactoring;

public static partial class CheckedRefactoring
{
    private static IReadOnlyList<string> OrganizeImports(Draft draft, SourceDocument source)
    {
        var imports = draft.Syntax(source).Imports();
        if (imports is null) return ["The file has no import clause."];
        var owners = draft.Owners(source.Path).ToArray();
        var notes = new List<string>();
        var items = imports.Items;
        var removable = items.Select(item => owners.All(owner => Unused(owner.Snapshot, item))).ToArray();
        if (removable.All(value => value) && !CommentIn(imports.Start, imports.End))
            Remove(imports.Start, imports.End);
        else
        {
            for (var first = 0; first < items.Count;)
            {
                if (!removable[first]) { first++; continue; }
                var last = first + 1;
                while (last < items.Count && removable[last]) last++;
                var start = last < items.Count || first == 0 ? items[first].Start : items[first - 1].End;
                var end = last < items.Count ? items[last].Start : items[last - 1].End;
                if (last < items.Count || first > 0)
                {
                    if (!CommentIn(start, end)) Remove(start, end);
                    else notes.Add("An unused import was retained to preserve an adjacent comment.");
                }
                first = last;
            }
        }
        if (draft.Edits.Count == 0) notes.Add("No safely removable CLR imports were found.");
        else notes.Add("Remaining aliases, source-unit imports and import order are preserved.");
        return notes.Distinct().ToArray();

        bool CommentIn(int start, int end) => imports.Comments.Any(comment => comment.Offset < end && comment.Offset + comment.Length > start);

        void Remove(int start, int end)
        {
            draft.Removed.Add(source.Span(start, end - start));
            draft.Edit(source, start, end, "");
        }

        bool Unused(SemanticSnapshot snapshot, RefactoringImport item)
        {
            draft.Token.ThrowIfCancellationRequested();
            var alias = snapshot.Occurrences.SingleOrDefault(occurrence => occurrence.Role == "declaration" &&
                In(occurrence.Span, source.Path, item.Alias, item.Alias + 1));
            if (alias is null || snapshot.Symbol(alias.Symbol).Kind != "alias") return false;
            var target = snapshot.Occurrences.FirstOrDefault(occurrence =>
                In(occurrence.Span, source.Path, item.Target, item.Target + 1) && occurrence.Role is "namespace" or "type");
            return target is not null && snapshot.Symbol(target.Symbol) is ({ Kind: "namespace" } or { Kind: "type", Metadata: not null }) &&
                !snapshot.Occurrences.Any(occurrence => occurrence.Role != "declaration" && occurrence.Symbol == alias.Symbol);
        }
    }
}
