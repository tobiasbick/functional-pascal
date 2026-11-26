// [Documentation](../../../docs/architecture/workspaces.md#source-queries)
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;

namespace Fpas.Workspaces.Queries;

/// <summary>Selects a source position using one-based UTF-16 coordinates.</summary>
/// <param name="File">Source path in the context.</param>
/// <param name="Line">One-based line.</param>
/// <param name="Column">One-based UTF-16 column; tabs count as one.</param>
/// <param name="Project">Optional owning project path for a linked source.</param>
public sealed record SourceSelection(string File, int Line, int Column, string? Project = null);
/// <summary>Records the checked owner and exact span selected by a query.</summary>
/// <param name="Project">Owning project path, or null for a standalone source.</param>
/// <param name="Span">Selected source span.</param>
public sealed record QuerySelection(string? Project, SourceSpan Span);
/// <summary>Describes a resolved source, metadata or implicit declaration.</summary>
/// <param name="Project">Source owner, or null for external metadata.</param>
/// <param name="Origin">source, metadata or implicit.</param>
/// <param name="Symbol">Resolved symbol descriptor.</param>
/// <param name="Types">Self-contained types reachable from the declaration surface.</param>
public sealed record DefinitionResult(string? Project, string Origin, SemanticSymbol Symbol, IReadOnlyList<SemanticType> Types);
/// <summary>Describes the checked type of a source expression.</summary>
/// <param name="Type">Actual checked expression type.</param>
/// <param name="DeclaredType">Declared type, when separately available.</param>
/// <param name="Symbol">Symbol at the position, when present.</param>
/// <param name="Types">Types required to interpret the descriptors.</param>
public sealed record TypeResult(SemanticType Type, SemanticType? DeclaredType, SemanticSymbol? Symbol, IReadOnlyList<SemanticType> Types);
/// <summary>Identifies one checked reference in an owning compilation.</summary>
/// <param name="Project">Owning project path.</param>
/// <param name="Span">Reference token span.</param>
/// <param name="Role">Semantic occurrence role.</param>
public sealed record ReferenceResult(string Project, SourceSpan Span, string Role);
/// <summary>Contains a bounded reference search excluding declaration occurrences.</summary>
/// <param name="Definition">Canonical target declaration.</param>
/// <param name="References">Ordered checked references.</param>
/// <param name="ReturnedCount">Number of returned references.</param>
/// <param name="TotalCount">Number of matching references before pagination.</param>
/// <param name="Offset">Number of sorted references skipped.</param>
/// <param name="NextOffset">Next page offset, or null at the end.</param>
public sealed record ReferencesResult(DefinitionResult Definition, IReadOnlyList<ReferenceResult> References, int ReturnedCount, int TotalCount,
    int Offset, int? NextOffset);
/// <summary>Contains one source query's payload, scope and independent coverage/truncation states.</summary>
/// <param name="Status">ok or not_found.</param>
/// <param name="Reason">Explicit absence reason, when applicable.</param>
/// <param name="Selection">Resolved source selection, when applicable.</param>
/// <param name="Definition">Definition-query payload.</param>
/// <param name="Type">Type-query payload.</param>
/// <param name="Refs">Reference-query payload.</param>
/// <param name="Truncated">Whether pagination or text budgets omitted results.</param>
/// <param name="BinaryBoundaries">References outside editable source ownership.</param>
/// <param name="Find">Name-search payload.</param>
/// <param name="Inspect">Inspection payload.</param>
/// <param name="Searched">Project paths searched, when explicitly supplied.</param>
/// <param name="Callees">Outgoing-call payload.</param>
/// <param name="Callers">Incoming-call payload.</param>
/// <param name="Coverage">Checked semantic coverage independent of pagination.</param>
/// <param name="Complete">Compiler-completion payload.</param>
/// <param name="Signature">Compiler signature-help payload.</param>
public sealed record SourceQueryResult(string Status, string? Reason, QuerySelection? Selection, DefinitionResult? Definition,
    TypeResult? Type, ReferencesResult? Refs, bool Truncated, IReadOnlyList<string> BinaryBoundaries,
    FindResult? Find = null, InspectResult? Inspect = null, IReadOnlyList<string>? Searched = null,
    CalleesResult? Callees = null, CallersResult? Callers = null, WorkspaceSemanticCoverage? Coverage = null,
    EditorCompletion? Complete = null, EditorCallSignatures? Signature = null);

/// <summary>Queries detached checked facts without executing code or reading source files.</summary>
public sealed partial class SourceQuery
{
    private readonly WorkspaceAnalysis analysis;
    private readonly Dictionary<string, (ContextProject Input, SemanticSnapshot Snapshot)> projects;
    private readonly Dictionary<(string Project, string Symbol), DefinitionResult> definitions = [];
    private readonly WorkspaceSemanticCoverage coverage;
    private static StringComparer Paths => OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;

    /// <summary>Creates a query service when at least one checked semantic region is available.</summary>
    /// <param name="analysis">Current detached project analysis.</param>
    public SourceQuery(WorkspaceAnalysis analysis)
    {
        this.analysis = analysis;
        projects = analysis.Context.Projects.Where(project => analysis.Projects.Any(result => Paths.Equals(result.Project, project.Path) && result.Result is { Snapshot: not null, Availability: not "unavailable" }))
            .ToDictionary(project => project.Path, project => (project, analysis.Projects.Single(result => Paths.Equals(result.Project, project.Path)).Result!.Snapshot!), Paths);
        coverage = analysis.SemanticCoverage;
        if (projects.Count == 0) throw new ContextFailure("analysis_failed", "No checked semantic regions are available in the selected scope.");
    }

    /// <summary>Queries a definition, expression type or references at an exact source position.</summary>
    /// <param name="operation">definition, type or refs.</param>
    /// <param name="selection">Source position and optional linked-file owner.</param>
    /// <param name="limit">Maximum returned references, 1..1,000.</param>
    /// <param name="cancellationToken">Cancels semantic search.</param>
    /// <param name="pageOffset">Number of sorted references to skip.</param>
    /// <returns>Checked facts or explicit absence, with coverage and binary boundaries.</returns>
    public SourceQueryResult Execute(string operation, SourceSelection selection, int limit = 100, CancellationToken cancellationToken = default, int pageOffset = 0)
        => ExecuteCore(operation, selection, limit, cancellationToken, pageOffset) with { Coverage = coverage };

    private SourceQueryResult ExecuteCore(string operation, SourceSelection selection, int limit, CancellationToken cancellationToken, int pageOffset)
    {
        if (operation is not ("definition" or "type" or "refs")) throw new ContextFailure("invalid_argument", "Unknown source-query operation.");
        if (limit is < 1 or > 1000) throw new ContextFailure("invalid_argument", "Reference limit must be 1..1000.");
        if (pageOffset < 0) throw new ContextFailure("invalid_argument", "Reference offset must be nonnegative.");
        cancellationToken.ThrowIfCancellationRequested();
        var (selectedProject, selectedSnapshot, offset, selectedPosition) = SelectPosition(selection);
        var owner = projects[selectedProject];
        var occurrence = selectedSnapshot.OccurrenceAt(selectedPosition.Span.Path, offset);
        var file = selectedPosition.Span.Path;
        var position = selectedPosition;
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
        var symbol = owner.Snapshot.Symbol(occurrence.Symbol);
        var definition = Definition(owner.Input.Path, symbol);
        if (operation == "definition")
        {
            if (definition.Origin == "implicit") return Missing(definition.Symbol.Kind == "namespace" ? "no_definition" : "implicit_declaration");
            return new("ok", null, position, definition, null, null, false, boundaries);
        }
        return References(symbol, definition, position, limit, pageOffset, boundaries, cancellationToken);

        SourceQueryResult Missing(string reason) => new("not_found", reason, position, null, null, null, false, boundaries);
    }

    private SourceQueryResult References(SemanticSymbol symbol, DefinitionResult definition, QuerySelection position, int limit, int pageOffset,
        IReadOnlyList<string> boundaries, CancellationToken cancellationToken)
    {
        var canonical = Canonical(position.Project!, symbol);
        var references = new List<ReferenceResult>();
        foreach (var (project, (_, snapshot)) in projects.OrderBy(project => project.Key, StringComparer.Ordinal))
        {
            var identities = new Dictionary<string, string>(StringComparer.Ordinal);
            foreach (var candidate in snapshot.Symbols)
            {
                cancellationToken.ThrowIfCancellationRequested(); identities[candidate.Id] = Canonical(project, candidate);
            }
            foreach (var reference in snapshot.Occurrences)
            {
                cancellationToken.ThrowIfCancellationRequested();
                if (reference.Role != "declaration" && identities[reference.Symbol] == canonical) references.Add(new(project, reference.Span, reference.Role));
            }
        }
        var ordered = references.Distinct().OrderBy(reference => reference.Project, StringComparer.Ordinal).ThenBy(reference => reference.Span.Path, StringComparer.Ordinal)
            .ThenBy(reference => reference.Span.Offset).ThenBy(reference => reference.Role, StringComparer.Ordinal).ToArray();
        var returned = Array.AsReadOnly(ordered.Skip(pageOffset).Take(limit).ToArray());
        var more = ordered.Length - pageOffset > returned.Count;
        return new("ok", null, position, null, null,
            new(definition, returned, returned.Count, ordered.Length, pageOffset, more ? pageOffset + returned.Count : null), returned.Count < ordered.Length, boundaries);
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
        if (ResolveSource(project, symbol) is { } definition)
            return Save(definition.Project, projects[definition.Project].Snapshot, definition.Symbol, "source");
        if (symbol.Metadata is not null)
            return Save(null, source.Snapshot, symbol, "metadata");
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
