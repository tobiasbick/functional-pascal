using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Snapshots;

namespace Fpas.Workspaces.Refactoring;

internal static class RenameOverlay
{
    internal static WorkspaceAnalysis Analyze(WorkspaceContext context, IReadOnlyDictionary<string, string> texts, CancellationToken token)
    {
        var documents = new WorkspaceTextSnapshot(context.Texts.Documents.Select(document => texts.TryGetValue(document.Path, out var text)
            ? new TextDocumentSnapshot(document.Path, text, document.Version) : document));
        var result = SourceGraphAnalysis.Analyze(context, documents, new(67_108_864), token);
        if (!result.Success) throw new ContextFailure("rename_conflict", "Renamed sources fail dependency or semantic validation: " +
            string.Join("; ", result.Projects.SelectMany(project => project.Result?.Diagnostics ?? []).Take(10).Select(diagnostic => diagnostic.Code + ": " + diagnostic.Message)));
        return result;
    }
}
