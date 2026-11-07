using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Loading;

namespace Fpas.Workspaces.Queries;

public sealed partial class SourceQuery
{
    private (string Project, SemanticSnapshot Snapshot, int Offset, QuerySelection Position) SelectPosition(SourceSelection selection, bool requireChecked = true)
    {
        var file = Path.GetFullPath(selection.File);
        var owners = analysis.Context.Projects.Where(project => project.Request.Sources.Any(document => Paths.Equals(document.Path, file))).ToArray();
        if (selection.Project is not null) owners = owners.Where(project => Paths.Equals(project.Path, Path.GetFullPath(selection.Project))).ToArray();
        if (owners.Length == 0) throw new ContextFailure("invalid_argument", "The file does not belong to the selected context/project: " + file);
        if (owners.Length > 1) throw new ContextFailure("ambiguous_project", "Select the file's owning compilation with --project.",
            candidates: owners.Select(owner => owner.Path).Order(StringComparer.Ordinal).ToArray());
        if (!projects.TryGetValue(owners[0].Path, out var owner)) throw new ContextFailure("analysis_unavailable", "The selected project has unavailable semantic analysis.");
        var document = owner.Snapshot.Documents.Single(document => Paths.Equals(document.Path, file));
        int offset;
        try { offset = document.Offset(selection.Line, selection.Column); }
        catch (ArgumentOutOfRangeException exception) { throw new ContextFailure("invalid_argument", exception.Message); }
        if (requireChecked && !owner.Snapshot.Coverage.AvailableAt(file, offset)) throw new ContextFailure("analysis_unavailable", "The selected source region could not be checked in this snapshot.");
        return (owner.Input.Path, owner.Snapshot, offset, new(owner.Input.Path, new(file, offset, 0, selection.Line, selection.Column)));
    }
}
