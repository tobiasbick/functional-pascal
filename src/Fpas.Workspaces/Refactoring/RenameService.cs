using Fpas.Compiler;
using Fpas.Compiler.Analysis;
using Fpas.Compiler.Syntax;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;
using System.Text;

namespace Fpas.Workspaces.Refactoring;

public sealed record PreparedRename(RenamePlan Plan, IReadOnlyList<SourceChange> Changes);

public static class RenameService
{
    private static StringComparer Paths => OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
    public static async Task<PreparedRename> PrepareAsync(WorkspaceAnalysis analysis, SourceSelection selection, string newName, CancellationToken token = default)
    {
        var spelling = IdentifierFacts.Spell(newName); token.ThrowIfCancellationRequested();
        var before = new RenameBindings(analysis);
        var selected = before.Query.Execute("definition", selection, cancellationToken: token);
        if (selected.Definition is not { Origin: "source", Project: not null } definition || definition.Symbol.Declaration is null)
            throw new ContextFailure("rename_not_supported", "Select an explicit FPAS declaration or its checked reference.");
        var snapshots = analysis.Projects.ToDictionary(project => project.Project, project => project.Result!.Snapshot!, Paths);
        var definitions = new Dictionary<string, DefinitionResult>(StringComparer.Ordinal);
        foreach (var project in analysis.Projects)
            foreach (var symbol in project.Result!.Snapshot!.Symbols.Where(symbol => symbol.Declaration is not null))
            {
                token.ThrowIfCancellationRequested(); definitions[before.Key(project.Project, symbol.Id)] = before.Resolve(project.Project, symbol.Id);
            }
        var targets = new HashSet<string>(StringComparer.Ordinal) { before.Key(definition.Project, definition.Symbol.Id) };
        bool added;
        do
        {
            added = false;
            var spans = targets.Select(key => definitions[key].Symbol.Declaration!).Select(span => (span.Path, span.Offset)).ToHashSet();
            foreach (var (key, item) in definitions)
                if (spans.Contains((item.Symbol.Declaration!.Path, item.Symbol.Declaration.Offset))) added |= targets.Add(key);
            foreach (var project in analysis.Projects)
                foreach (var relation in project.Result!.Snapshot!.Relations)
                {
                    var implementation = before.Key(project.Project, relation.Implementation); var contract = before.Key(project.Project, relation.Contract);
                    if (!(targets.Contains(implementation) || targets.Contains(contract))) continue;
                    if (!definitions.ContainsKey(implementation) || !definitions.ContainsKey(contract))
                        throw new ContextFailure("rename_external_contract", "The selected member implements a contract outside the editable FPAS context.");
                    added |= targets.Add(implementation); added |= targets.Add(contract);
                }
        } while (added);
        var sources = analysis.Context.Projects.SelectMany(project => project.Request.Sources).GroupBy(source => source.Path, Paths)
            .Select(group => group.Select(source => source.Text).Distinct(StringComparer.Ordinal).Count() == 1 ? group.First() :
                throw new ContextFailure("edit_conflict", "Linked source owners have different contents: " + group.Key)).ToArray();
        if (sources.Sum(source => (long)Encoding.UTF8.GetByteCount(source.Text)) > 67_108_864)
            throw new ContextFailure("rename_limit", "Rename source inputs exceed 64 MiB.");
        var documents = sources.ToDictionary(source => source.Path, source => source.Text, Paths);
        var edits = new List<DiagnosticEdit>();
        foreach (var project in analysis.Projects)
            foreach (var occurrence in project.Result!.Snapshot!.Occurrences)
            {
                token.ThrowIfCancellationRequested();
                if (!targets.Contains(before.Key(project.Project, occurrence.Symbol))) continue;
                var span = occurrence.Span; var original = documents[span.Path].Substring(span.Offset, span.Length);
                if (original.TrimStart('@') == newName) continue;
                if (original.TrimStart('@') != Token(definition))
                    throw new ContextFailure("rename_not_supported", "A related source location has no matching identifier token.");
                edits.Add(new(new(span.Path, span.Line, span.Column, span.Length, Offset: span.Offset), original, spelling));
                if (edits.Count > 1000) throw new ContextFailure("rename_limit", "Rename exceeds 1,000 source edits.");
            }
        var editPlan = await DiagnosticFixPlan.CreateAsync(sources, edits.Count == 0 ? [] :
            [new(definition.Project, "rename", new("Rename identifier", DiagnosticApplicability.Automatic, edits))], token);
        var grouped = edits.DistinctBy(edit => (edit.Location.Path, edit.Location.Offset, edit.Location.Length, edit.NewText))
            .GroupBy(edit => edit.Location.Path, Paths).ToDictionary(group => group.Key, group => group.OrderBy(edit => edit.Location.Offset).ToArray(), Paths);
        int Map(string path, int offset) => offset + (grouped.TryGetValue(path, out var changes) ? changes.Where(edit => edit.Location.Offset < offset)
            .Sum(edit => edit.NewText.Length - edit.Location.Length) : 0);
        if (editPlan.Changes.Count != 0)
        {
            using var overlay = new RenameOverlay();
            var texts = editPlan.Changes.ToDictionary(change => change.Path, change => new UTF8Encoding(false, true).GetString(change.Replacement), Paths);
            var changed = overlay.Analyze(analysis.Context, texts, token);
            var expected = new RenameBindings(analysis, Map).Occurrences(token); var actual = new RenameBindings(changed).Occurrences(token);
            if (expected.Count != actual.Count || expected.Any(pair => !actual.TryGetValue(pair.Key, out var key) || key != pair.Value) ||
                !new RenameBindings(analysis, Map).Relations(token).SetEquals(new RenameBindings(changed).Relations(token)))
                throw new ContextFailure("rename_conflict", "The rename changes a source binding, overload selection or declaration relationship.");
        }
        var files = editPlan.Files.Select(file => new RenameFile(file.Path, file.OriginalHash, file.ReplacementHash, grouped[file.Path])).ToArray();
        var descriptors = targets.Select(key => definitions[key]).Select(item => new RenameTarget(item.Project!, item.Symbol.Kind, Token(item),
            item.Symbol.Declaration!, Public(item))).OrderBy(item => item.Project, StringComparer.Ordinal).ThenBy(item => item.Declaration.Path, StringComparer.Ordinal)
            .ThenBy(item => item.Declaration.Offset).ToArray();
        var plan = new RenamePlan(1, "rename", analysis.Context.Path, analysis.Context.Configuration, analysis.Context.SnapshotId,
            selection with { File = Path.GetFullPath(selection.File), Project = selection.Project is null ? null : Path.GetFullPath(selection.Project) }, newName,
            descriptors, files, analysis.Projects.Select(project => project.Project).Order(StringComparer.Ordinal).ToArray(), before.Query.Boundaries());
        plan = plan with { Id = RenamePlanJson.Identity(plan) };
        if (RenamePlanJson.Serialize(plan).Length > RenamePlanJson.MaximumBytes) throw new ContextFailure("rename_limit", "Rename plan exceeds 4 MiB.");
        return new(plan, editPlan.Changes);

        string Token(DefinitionResult item) => documents[item.Symbol.Declaration!.Path].Substring(item.Symbol.Declaration.Offset, item.Symbol.Declaration.Length).TrimStart('@');
        bool Public(DefinitionResult item)
        {
            var symbol = item.Symbol; var seenPublic = false;
            while (true)
            {
                if (symbol.Context?.Accessibility == "private") return false;
                seenPublic |= symbol.Context?.Accessibility == "public";
                if (symbol.Context?.Owner is not { } owner) break;
                symbol = snapshots[item.Project!].Symbol(owner);
            }
            return seenPublic || item.Symbol.Kind == "container";
        }
    }
}
