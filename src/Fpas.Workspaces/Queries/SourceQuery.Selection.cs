using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Loading;

namespace Fpas.Workspaces.Queries;

public sealed partial class SourceQuery
{
    private (string Project, SemanticSnapshot Snapshot, int Offset, QuerySelection Position) SelectPosition(SourceSelection selection)
    {
        var file = Path.GetFullPath(selection.File);
        var owners = projects.Values.Where(project => project.Snapshot.Documents.Any(document => Paths.Equals(document.Path, file))).ToArray();
        if (selection.Project is not null) owners = owners.Where(project => Paths.Equals(project.Input.Path, Path.GetFullPath(selection.Project))).ToArray();
        if (owners.Length == 0) throw new ContextFailure("invalid_argument", "The file does not belong to the selected context/project: " + file);
        if (owners.Length > 1) throw new ContextFailure("ambiguous_project", "Select the file's owning compilation with --project.",
            candidates: owners.Select(owner => owner.Input.Path).Order(StringComparer.Ordinal).ToArray());
        var owner = owners[0]; var document = owner.Snapshot.Documents.Single(document => Paths.Equals(document.Path, file));
        int offset;
        try { offset = document.Offset(selection.Line, selection.Column); }
        catch (ArgumentOutOfRangeException exception) { throw new ContextFailure("invalid_argument", exception.Message); }
        return (owner.Input.Path, owner.Snapshot, offset, new(owner.Input.Path, new(file, offset, 0, selection.Line, selection.Column)));
    }
}
