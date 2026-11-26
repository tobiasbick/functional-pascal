// [Documentation](../../../docs/pascal/tools/refactoring.md#vs-code-refactorings)
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Analysis;

namespace Fpas.Workspaces.Refactoring;

/// <remarks>Import organization removes only proven unused CLR aliases while preserving comments and source imports.</remarks>
public static partial class CheckedRefactoring
{
    private static IReadOnlyList<string> OrganizeImports(Draft draft, SourceDocument source)
    {
        var syntax = draft.Syntax(source);
        var imports = syntax.Imports();
        if (imports is null) return ["The file has no imports."];
        var owners = draft.Owners(source.Path).ToArray();
        var notes = new List<string>();
        var items = imports.Items;
        var unused = items.Select(item => item.Alias).ToHashSet();
        foreach (var (_, snapshot) in owners)
            unused.IntersectWith(ImportUsage.UnusedClrAliases(snapshot, draft.Token)
                .Where(alias => Paths.Equals(alias.Declaration!.Path, source.Path)).Select(alias => alias.Declaration!.Offset));
        for (var index = 0; index < items.Count; index++)
        {
            var item = items[index];
            if (!unused.Contains(item.Alias)) continue;
            var prior = index == 0 ? syntax.ContainerHeaderEnd : items[index - 1].End;
            var lineEnd = source.Text.IndexOfAny(['\r', '\n'], item.End);
            if (lineEnd < 0) lineEnd = source.Text.Length;
            var next = index + 1 < items.Count ? items[index + 1].Start : source.Text.Length;
            if (CommentIn(prior, Math.Min(lineEnd, next)))
            {
                notes.Add("An unused import was retained to preserve an adjacent comment.");
                continue;
            }
            var end = item.End;
            if (source.Text.AsSpan(end, lineEnd - end).Trim().IsEmpty && lineEnd < source.Text.Length)
                end = lineEnd + LineEnding(source.Text, lineEnd).Length;
            Remove(item.Start, end);
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
