// [Documentation](../../../docs/pascal/tools/refactoring.md#vs-code-refactorings)
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Analysis;

namespace Fpas.Workspaces.Refactoring;

/// <remarks>Import organization removes only proven unused CLR aliases while preserving comments and source imports.</remarks>
public static partial class CheckedRefactoring
{
    private static IReadOnlyList<string> OrganizeImports(Draft draft, SourceDocument source)
    {
        var imports = draft.Syntax(source).Imports();
        if (imports is null) return ["The file has no import clause."];
        var owners = draft.Owners(source.Path).ToArray();
        var notes = new List<string>();
        var items = imports.Items;
        var unused = items.Select(item => item.Alias).ToHashSet();
        foreach (var (_, snapshot) in owners)
            unused.IntersectWith(ImportUsage.UnusedClrAliases(snapshot, draft.Token)
                .Where(alias => Paths.Equals(alias.Declaration!.Path, source.Path)).Select(alias => alias.Declaration!.Offset));
        var removable = items.Select(item => unused.Contains(item.Alias)).ToArray();
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
    }
}
