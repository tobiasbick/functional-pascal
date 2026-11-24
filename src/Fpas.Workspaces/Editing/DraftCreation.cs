// [Documentation](../../../docs/pascal/tools/refactoring.md#plan-and-apply-a-checked-draft)
using Fpas.Compiler;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Snapshots;

namespace Fpas.Workspaces.Editing;

/// <summary>Describes a source to create in an evaluated FPAS project.</summary>
/// <param name="Path">Absolute destination path that must still be free.</param>
/// <param name="Project">Absolute path of its declared owning project.</param>
/// <param name="Text">Complete source text.</param>
public sealed record DraftCreation(string Path, string Project, string Text);

internal static class DraftAnalysis
{
    internal static WorkspaceAnalysis Analyze(WorkspaceContext context, IReadOnlyDictionary<string, string> replacements,
        IReadOnlyList<DraftCreation> creations, CancellationToken token)
    {
        var paths = WorkspaceTextSnapshot.Paths;
        if (creations.Count != 0 && context.Issues.FirstOrDefault() is { } issue)
            throw new ContextFailure("analysis_unavailable", "Cannot confirm draft ownership. " + issue.Message,
                issue.OutputTruncated, project: issue.InputProject ?? issue.Project, inputPath: issue.InputPath, nextAction: issue.NextAction);
        var originals = context.Texts.Documents.Select(document => document.Path).ToHashSet(paths);
        foreach (var path in replacements.Keys.Where(path => !originals.Contains(path)))
            throw new ContextFailure(context.Issues.Count == 0 ? "invalid_argument" : "analysis_unavailable",
                "A draft replacement must be an existing source in the evaluated context.", inputPath: path);
        foreach (var file in creations)
        {
            token.ThrowIfCancellationRequested();
            var owners = context.DraftOwners.GetValueOrDefault(file.Path) ?? [];
            if (owners.Count != 1 || !paths.Equals(owners[0], file.Project))
                throw new ContextFailure("invalid_argument", "The new source must have exactly the declared FPAS owner under the evaluated Compile rules.",
                    inputPath: file.Path, project: file.Project, candidates: owners);
        }
        var documents = new WorkspaceTextSnapshot(context.Texts.Documents.Select(document => replacements.TryGetValue(document.Path, out var text)
            ? new TextDocumentSnapshot(document.Path, text, document.Version) : document).Concat(creations.Select(file => new TextDocumentSnapshot(file.Path, file.Text))));
        if (creations.Count != 0)
        {
            var byProject = creations.ToLookup(file => file.Project, paths);
            var projects = context.Projects.Select(project => project with { Request = project.Request with
            {
                Sources = project.Request.Sources.Concat(byProject[project.Path].Select(file => new CompilationSource(file.Path, file.Text)))
                    .OrderBy(source => source.Path, StringComparer.Ordinal).ToArray()
            } });
            context = context.WithProjects(projects, documents);
        }
        return SourceGraphAnalysis.Analyze(context, documents, new(67_108_864), token);
    }
}
