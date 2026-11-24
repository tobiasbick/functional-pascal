// [Documentation](../../../docs/architecture/workspaces.md#source-queries)
using Fpas.Compiler;
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;

namespace Fpas.Workspaces.Queries;

/// <remarks>Source selections require unambiguous physical ownership and checked-region availability when requested.</remarks>
public sealed partial class SourceQuery
{
    private (string Project, SemanticSnapshot Snapshot, int Offset, QuerySelection Position) SelectPosition(SourceSelection selection, bool requireChecked = true)
    {
        var (project, source) = SelectOwner(analysis, selection);
        var file = source.Path;
        if (!projects.TryGetValue(project.Path, out var owner)) throw new ContextFailure("analysis_unavailable", "The selected project has unavailable semantic analysis.");
        var document = owner.Snapshot.Documents.Single(document => Paths.Equals(document.Path, file));
        var offset = Offset(document, selection);
        if (requireChecked && !owner.Snapshot.Coverage.AvailableAt(file, offset)) throw new ContextFailure("analysis_unavailable", "The selected source region could not be checked in this snapshot.");
        return (owner.Input.Path, owner.Snapshot, offset, new(owner.Input.Path, new(file, offset, 0, selection.Line, selection.Column)));
    }

    private static (ContextProject Project, CompilationSource Source) SelectOwner(WorkspaceAnalysis analysis, SourceSelection selection)
    {
        var file = Path.GetFullPath(selection.File);
        var selected = selection.Project is null ? null : Path.GetFullPath(selection.Project);
        var owners = analysis.Context.Projects.Where(project => selected is null || Paths.Equals(project.Path, selected))
            .Where(project => project.Request.Sources.Any(document => Paths.Equals(document.Path, file))).ToArray();
        if (owners.Length == 0) throw new ContextFailure("invalid_argument", "The file does not belong to the selected context/project: " + file);
        if (owners.Length > 1) throw new ContextFailure("ambiguous_project", "Select the file's owning compilation with --project.",
            candidates: owners.Select(owner => owner.Path).Order(StringComparer.Ordinal).ToArray());
        return (owners[0], owners[0].Request.Sources.Single(document => Paths.Equals(document.Path, file)));
    }

    private static int Offset(SourceDocument document, SourceSelection selection)
    {
        try { return document.Offset(selection.Line, selection.Column); }
        catch (ArgumentOutOfRangeException exception) { throw new ContextFailure("invalid_argument", exception.Message); }
    }
}
