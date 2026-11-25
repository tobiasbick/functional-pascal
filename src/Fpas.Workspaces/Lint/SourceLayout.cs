// [Documentation](../../../docs/architecture/workspaces.md#semantic-lint)
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;

namespace Fpas.Workspaces.Lint;

internal static class SourceLayout
{
    internal static IEnumerable<LintFinding> Find(WorkspaceAnalysis analysis, IReadOnlySet<string> testProjects,
        LintLimits limits, CancellationToken token)
    {
        var paths = WorkspacePaths.Comparer;
        var root = Path.GetDirectoryName(analysis.Context.Path)!;
        var files = analysis.Projects.OrderBy(project => testProjects.Contains(project.Project))
            .SelectMany(project => project.Result!.Snapshot!.Documents
            .Select(document => (Project: project.Project, Document: document))).DistinctBy(file => file.Document.Path, paths);
        var directories = new Dictionary<string, Dictionary<string, (string Project, SourceSpan Location)>>(paths);
        foreach (var (project, document) in files)
        {
            token.ThrowIfCancellationRequested();
            var location = document.Span(0, 0);
            var end = document.Text.Length;
            if (end > 0 && document.Text[end - 1] == '\n') end--;
            if (end > 0 && document.Text[end - 1] == '\r') end--;
            var lines = document.Span(end, 0).Line;
            if (lines > limits.MaxLines && !testProjects.Contains(project))
                yield return new("long-file", $"Source has {lines} lines. The limit is {limits.MaxLines}.", project, location, [],
                    LintFinding.Relative(analysis.Context.Path, document.Path));

            var entry = document.Path;
            var directory = Path.GetDirectoryName(entry);
            var relative = Path.GetRelativePath(root, entry);
            var linkedOutside = Path.IsPathRooted(relative) || relative.StartsWith(".." + Path.DirectorySeparatorChar, StringComparison.Ordinal);
            while (directory is not null)
            {
                if (!directories.TryGetValue(directory, out var members)) directories.Add(directory, members = new(paths));
                members.TryAdd(entry, (project, location));
                if (linkedOutside || paths.Equals(directory, root)) break;
                entry = directory;
                directory = Path.GetDirectoryName(entry);
            }
        }

        foreach (var (directory, members) in directories)
        {
            token.ThrowIfCancellationRequested();
            if (members.Count <= limits.MaxDirectoryEntries) continue;
            var ordered = members.OrderBy(member => member.Key, StringComparer.Ordinal).ToArray();
            var (project, location) = ordered[0].Value;
            var name = paths.Equals(directory, root) ? "." : Path.GetRelativePath(root, directory);
            yield return new("wide-directory", $"Directory '{name}' has {members.Count} FPAS source entries. The limit is {limits.MaxDirectoryEntries}.",
                project, location, ordered.Select(member => LintFinding.Note(
                    $"Source entry '{Path.GetFileName(member.Key)}'.", member.Value.Location)).ToArray(), LintFinding.Relative(analysis.Context.Path, directory));
        }
    }
}
