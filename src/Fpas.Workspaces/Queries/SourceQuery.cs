using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;

namespace Fpas.Workspaces.Queries;

public sealed record SourceSelection(string File, int Line, int Column, string? Project = null);
public sealed record QuerySelection(string Project, SourceSpan Span);
public sealed record DefinitionResult(string? Project, string Origin, SemanticSymbol Symbol, IReadOnlyList<SemanticType> Types);
public sealed record TypeResult(SemanticType Type, SemanticType? DeclaredType, SemanticSymbol? Symbol, IReadOnlyList<SemanticType> Types);
public sealed record ReferenceResult(string Project, SourceSpan Span, string Role);
public sealed record ReferencesResult(DefinitionResult Definition, IReadOnlyList<ReferenceResult> References, int ReturnedCount);
public sealed record SourceQueryResult(string Status, string? Reason, QuerySelection? Selection, DefinitionResult? Definition,
    TypeResult? Type, ReferencesResult? Refs, bool Truncated, IReadOnlyList<string> BinaryBoundaries,
    FindResult? Find = null, InspectResult? Inspect = null, IReadOnlyList<string>? Searched = null);

public sealed partial class SourceQuery
{
    private readonly WorkspaceAnalysis analysis;
    private readonly Dictionary<string, (ContextProject Input, SemanticSnapshot Snapshot)> projects;
    private readonly Dictionary<(string Project, string Symbol), DefinitionResult> definitions = [];
    private static StringComparer Paths => OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;

    public SourceQuery(WorkspaceAnalysis analysis)
    {
        if (!analysis.Success || !analysis.IsComplete) throw new ContextFailure("analysis_failed", "Queries require successful analysis of the complete selected scope.");
        this.analysis = analysis;
        projects = analysis.Context.Projects.ToDictionary(project => project.Path,
            project => (project, analysis.Projects.Single(result => Paths.Equals(result.Project, project.Path)).Result!.Snapshot!), Paths);
    }

    public SourceQueryResult Execute(string operation, SourceSelection selection, int limit = 100, CancellationToken cancellationToken = default)
    {
        if (operation is not ("definition" or "type" or "refs")) throw new ContextFailure("invalid_argument", "Unknown source-query operation.");
        if (limit is < 1 or > 1000) throw new ContextFailure("invalid_argument", "Reference limit must be 1..1000.");
        cancellationToken.ThrowIfCancellationRequested();
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
        var occurrence = owner.Snapshot.OccurrenceAt(file, offset);
        var position = new QuerySelection(owner.Input.Path, new(file, offset, 0, selection.Line, selection.Column));
        var boundaries = BinaryBoundaries();
        if (operation == "type")
        {
            var expression = owner.Snapshot.TypeAt(file, offset);
            if (expression is null) return Missing(occurrence is null ? "no_type" : "not_typed");
            position = position with { Span = expression.Span };
            var selected = occurrence is null ? null : owner.Snapshot.Symbol(occurrence.Symbol);
            var type = owner.Snapshot.Types.Single(type => type.Id == expression.Type);
            var declared = expression.DeclaredType is null ? null : owner.Snapshot.Types.Single(type => type.Id == expression.DeclaredType);
            return new("ok", null, position, null, new(type, declared, selected, Types(owner.Snapshot,
                (selected?.Context?.SurfaceTypes ?? []).Concat([type.Id, declared?.Id, selected?.Type]))), null, false, boundaries);
        }
        if (occurrence is null) return Missing("no_symbol");
        position = position with { Span = occurrence.Span };
        var definition = Definition(owner.Input.Path, owner.Snapshot.Symbol(occurrence.Symbol));
        if (operation == "definition")
        {
            if (definition.Origin == "implicit") return Missing(definition.Symbol.Kind == "namespace" ? "no_definition" : "implicit_declaration");
            return new("ok", null, position, definition, null, null, false, boundaries);
        }
        var canonical = Canonical(owner.Input.Path, owner.Snapshot.Symbol(occurrence.Symbol));
        var references = new List<ReferenceResult>();
        foreach (var (project, (_, snapshot)) in projects.OrderBy(project => project.Key, StringComparer.Ordinal))
        {
            var identities = new Dictionary<string, string>(StringComparer.Ordinal);
            foreach (var symbol in snapshot.Symbols)
            {
                cancellationToken.ThrowIfCancellationRequested(); identities[symbol.Id] = Canonical(project, symbol);
            }
            foreach (var reference in snapshot.Occurrences)
            {
                cancellationToken.ThrowIfCancellationRequested();
                if (reference.Role != "declaration" && identities[reference.Symbol] == canonical) references.Add(new(project, reference.Span, reference.Role));
            }
        }
        var ordered = references.Distinct().OrderBy(reference => reference.Project, StringComparer.Ordinal).ThenBy(reference => reference.Span.Path, StringComparer.Ordinal)
            .ThenBy(reference => reference.Span.Offset).ThenBy(reference => reference.Role, StringComparer.Ordinal).ToArray();
        var returned = Array.AsReadOnly(ordered.Take(limit).ToArray());
        return new("ok", null, position, null, null, new(definition, returned, returned.Count), ordered.Length > limit, boundaries);

        SourceQueryResult Missing(string reason) => new("not_found", reason, position, null, null, null, false, boundaries);
    }

    private string Canonical(string project, SemanticSymbol symbol)
    {
        if (symbol.Kind == "namespace") return "namespace:" + symbol.Name;
        var definition = Definition(project, symbol);
        return (definition.Project ?? "metadata") + ":" + (definition.Project is null && symbol.Metadata is not null ? symbol.Metadata.GetKey() : definition.Symbol.Id);
    }

    private DefinitionResult Definition(string project, SemanticSymbol symbol)
    {
        if (definitions.TryGetValue((project, symbol.Id), out var cached)) return cached;
        var source = projects[project];
        if (symbol.Declaration is not null) return Save(project, source.Snapshot, symbol, "source");
        if (symbol.Metadata is { } metadata)
        {
            var matches = new List<(string Project, SemanticSnapshot Snapshot, SemanticSymbol Symbol)>();
            foreach (var output in analysis.Context.DependencyOutputs.Where(output => projects.ContainsKey(output.Project) && source.Input.Request.References.Contains(output.Path, Paths)))
            {
                var candidate = projects[output.Project];
                if (candidate.Snapshot.AssemblyName != metadata.Assembly) continue;
                foreach (var export in candidate.Snapshot.Exports.Where(export => MetadataKey(export.Identity) == MetadataKey(metadata)))
                {
                    var declaration = candidate.Snapshot.Symbol(export.Symbol);
                    if (declaration.Declaration is not null) matches.Add((output.Project, candidate.Snapshot, declaration));
                }
            }
            var unique = matches.Distinct().ToArray();
            if (unique.Length > 1) throw new ContextFailure("context_failed", "Reference metadata maps to conflicting source declarations.");
            if (unique.Length == 1) return Save(unique[0].Project, unique[0].Snapshot, unique[0].Symbol, "source");
            return Save(null, source.Snapshot, symbol, "metadata");
        }
        return Save(project, source.Snapshot, symbol, "implicit");

        DefinitionResult Save(string? targetProject, SemanticSnapshot snapshot, SemanticSymbol target, string origin)
            => definitions[(project, symbol.Id)] = new(targetProject, origin, target, Types(snapshot, (target.Context?.SurfaceTypes ?? []).Append(target.Type)));
    }

    private static string MetadataKey(MetadataIdentity identity) => identity.GetKey(includeAssemblyHash: false);

    private static IReadOnlyList<SemanticType> Types(SemanticSnapshot snapshot, IEnumerable<string?> roots)
    {
        var all = snapshot.Types.ToDictionary(type => type.Id, StringComparer.Ordinal);
        var seen = new HashSet<string>(StringComparer.Ordinal); var pending = new Stack<string>(roots.OfType<string>());
        while (pending.TryPop(out var id))
        {
            if (!seen.Add(id)) continue;
            var type = all[id];
            if (type.Element is not null) pending.Push(type.Element);
            foreach (var argument in type.Arguments) pending.Push(argument);
            if (type.Callable is { } callable)
            {
                pending.Push(callable.Result);
                foreach (var parameter in callable.Parameters) pending.Push(parameter.Type);
                foreach (var parameter in callable.TypeParameters) pending.Push(parameter);
            }
        }
        return Array.AsReadOnly(seen.Order(StringComparer.Ordinal).Select(id => all[id]).ToArray());
    }
}
