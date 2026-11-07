using Fpas.Compiler;
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Refactoring;
using Fpas.Workspaces.Snapshots;

namespace Fpas.Workspaces.Editing;

public sealed record CheckedImport(SourceSpan Trigger, DiagnosticFix Fix);

public static class CheckedImports
{
    public static IReadOnlyList<CheckedImport> Find(WorkspaceAnalysis analysis, string project, string path, CancellationToken token = default)
    {
        var paths = WorkspaceTextSnapshot.Paths;
        var input = analysis.Context.Projects.Single(item => paths.Equals(item.Path, project));
        var result = analysis.Projects.Single(item => paths.Equals(item.Project, project)).Result;
        if (analysis.Context.Issues.Count != 0 || result?.Snapshot is null) return [];
        var suggestions = new List<CheckedImport>();
        foreach (var diagnostic in result.Diagnostics.Where(item => item.Code == "FP3001" && item.Location is { Offset: not null } location &&
            paths.Equals(location.Path, path)).Take(16))
        {
            token.ThrowIfCancellationRequested();
            var import = EditorImports.Suggest(input.Request, path, diagnostic.Location!.Offset!.Value, token);
            if (import is null || !PreservesBindings(analysis, project, import, token)) continue;
            suggestions.Add(new(import.Receiver, new("Import " + import.QualifiedName + " as " + import.Alias, DiagnosticApplicability.Automatic, [import.Edit])));
        }
        return suggestions.AsReadOnly();
    }

    private static bool PreservesBindings(WorkspaceAnalysis analysis, string project, EditorImport import, CancellationToken token)
    {
        var paths = WorkspaceTextSnapshot.Paths;
        var path = import.Receiver.Path;
        var edit = import.Edit;
        var offset = edit.Location.Offset!.Value;
        var length = edit.NewText.Length;
        int OriginalOffset(string file, int position) => paths.Equals(file, path) && position >= offset + length ? position - length : position;
        var edited = TextEditSet.Create(analysis.Context.Texts, [edit], token).Documents.Single();
        var documents = new WorkspaceTextSnapshot(analysis.Context.Texts.Documents.Select(document => paths.Equals(document.Path, path)
            ? new TextDocumentSnapshot(path, edited.NewText, document.Version) : document));
        try
        {
            var updated = SourceGraphAnalysis.Analyze(analysis.Context, documents, new(67_108_864), token);
            if (!updated.Success || !updated.IsComplete) return false;
            var before = new RenameBindings(analysis, requireComplete: false);
            var after = new RenameBindings(updated, OriginalOffset);
            var snapshot = updated.Projects.Single(item => paths.Equals(item.Project, project)).Result!.Snapshot!;
            var receiver = snapshot.OccurrenceAt(path, import.Receiver.Offset + length);
            var member = snapshot.OccurrenceAt(path, import.Member.Offset + length);
            if (receiver is null || member is null || snapshot.Symbol(receiver.Symbol) is not { Kind: "alias", Name: var alias } || alias != import.Alias)
                return false;
            var target = after.Resolve(project, member.Symbol);
            if (import.AssemblyHash is { } hash)
            {
                if (target.Symbol.Metadata is not { } identity || identity.AssemblyHash != hash || identity.Type != import.QualifiedName) return false;
            }
            else if (target.Origin != "source" || target.Symbol.Context?.QualifiedName.StartsWith(import.QualifiedName + ".", StringComparison.Ordinal) != true)
                return false;
            var newOccurrences = after.Occurrences(token, span => !paths.Equals(span.Path, path) || span.Offset < offset || span.Offset >= offset + length);
            return before.Occurrences(token).All(pair => newOccurrences.TryGetValue(pair.Key, out var identity) && identity == pair.Value) &&
                before.Relations(token).IsSubsetOf(after.Relations(token));
        }
        catch (ContextFailure) { return false; }
    }
}
