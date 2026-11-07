using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Documentation;
using Fpas.Workspaces.Loading;

namespace Fpas.Workspaces.Queries;

public sealed record EditorHover(SourceSpan Span, string Signature, string? Documentation);
public sealed record EditorClassification(SourceSpan Span, string Kind, bool Declaration);
public sealed record EditorSignatures(string Project, IReadOnlyList<DeclarationMatch> Declarations, bool Complete, int Omitted);

public sealed partial class SourceQuery
{
    public EditorHover? Hover(SourceSelection selection, CancellationToken token = default)
    {
        var (project, snapshot, offset, position) = SelectPosition(selection);
        var occurrence = snapshot.OccurrenceAt(position.Span.Path, offset);
        if (occurrence is null) return null;
        var symbol = snapshot.Symbol(occurrence.Symbol);
        var definition = Definition(project, symbol);
        var site = snapshot.Operations.Where(operation => operation.Anchor is { } anchor && Paths.Equals(anchor.Path, position.Span.Path) &&
            anchor.Contains(offset) && operation.Signature is not null).OrderBy(operation => operation.Span.Length).FirstOrDefault();
        var signature = site?.Signature is { } callable ? Signature(symbol.Name, callable, snapshot) : symbol.Signature;
        var documentation = definition.Origin == "source" ? SourceDocumentation(definition) : symbol.Metadata is { } identity ?
            BoundDocumentation.Read(identity, projects[project].Input.Request, analysis.Context.SnapshotId, token) : null;
        return new(occurrence.Span, Bound(signature), documentation is null ? null : Bound(documentation));
    }

    public SourceSpan? TypeDefinition(SourceSelection selection)
    {
        var (project, snapshot, offset, position) = SelectPosition(selection);
        var occurrence = snapshot.OccurrenceAt(position.Span.Path, offset);
        var id = snapshot.TypeAt(position.Span.Path, offset)?.Type ?? (occurrence is null ? null : snapshot.Symbol(occurrence.Symbol).Type);
        var type = snapshot.Types.FirstOrDefault(item => item.Id == id);
        if (type?.Parameter is { } parameter) return snapshot.Symbol(parameter).Declaration;
        if (type?.Metadata is not { } identity) return null;
        var candidate = snapshot.Symbols.FirstOrDefault(item => item.Metadata is { } metadata && MetadataKey(metadata) == MetadataKey(identity)) ??
            new(type.Id + ":type-definition", "type", type.Display, type.Display, null, type.Id, identity, null);
        return Definition(project, candidate).Symbol.Declaration;
    }

    public IReadOnlyList<SymbolOccurrence> Highlights(SourceSelection selection, CancellationToken token = default)
    {
        var (project, snapshot, offset, position) = SelectPosition(selection);
        var occurrence = snapshot.OccurrenceAt(position.Span.Path, offset);
        if (occurrence is null) return [];
        var target = Canonical(project, snapshot.Symbol(occurrence.Symbol));
        return snapshot.Occurrences.Where(item =>
        {
            token.ThrowIfCancellationRequested();
            return Paths.Equals(item.Span.Path, position.Span.Path) && Canonical(project, snapshot.Symbol(item.Symbol)) == target;
        }).Take(1001).ToArray() is { Length: <= 1000 } matches ? matches :
            throw new ContextFailure("result_limit", "Document highlights exceed 1,000 locations.");
    }

    public IReadOnlyList<EditorClassification> Classifications(string file, CancellationToken token = default)
    {
        var (_, snapshot, _, position) = SelectPosition(new(file, 1, 1), requireChecked: false);
        var symbols = snapshot.Symbols.ToDictionary(item => item.Id, StringComparer.Ordinal);
        var matches = new List<EditorClassification>();
        foreach (var item in snapshot.Occurrences.Where(item => Paths.Equals(item.Span.Path, position.Span.Path)))
        {
            token.ThrowIfCancellationRequested();
            if (matches.Count >= 10_000) throw new ContextFailure("result_limit", "Semantic classification exceeds 10,000 tokens.");
            matches.Add(new(item.Span, symbols[item.Symbol].Kind, item.Role == "declaration"));
        }
        return matches.AsReadOnly();
    }

    public string EditorOwner(SourceSelection selection) => SelectPosition(selection, requireChecked: false).Project;

    public EditorCompletion Complete(SourceSelection selection, CancellationToken token = default)
    {
        var (project, _, offset, position) = SelectPosition(selection, requireChecked: false);
        return EditorAnalysis.Complete(projects[project].Input.Request, position.Span.Path, offset, token);
    }

    public EditorCallSignatures Signatures(SourceSelection selection, CancellationToken token = default)
    {
        var (project, _, offset, position) = SelectPosition(selection, requireChecked: false);
        return EditorAnalysis.Signatures(projects[project].Input.Request, position.Span.Path, offset, token);
    }

    public string? EditorDocumentation(SourceSelection selection, MetadataIdentity? identity, SourceSpan? declaration, CancellationToken token = default)
    {
        var (project, snapshot, _, _) = SelectPosition(selection, requireChecked: false);
        if (identity is not null)
        {
            var match = snapshot.Symbols.FirstOrDefault(item => item.Metadata is { } metadata && MetadataKey(metadata) == MetadataKey(identity));
            if (match is not null && Definition(project, match) is { Origin: "source" } target) return SourceDocumentation(target);
            var text = BoundDocumentation.Read(identity, projects[project].Input.Request, analysis.Context.SnapshotId, token);
            return text is null ? null : Bound(text);
        }
        if (declaration is null) return null;
        var symbol = snapshot.Symbols.FirstOrDefault(item => item.Declaration == declaration);
        return symbol is null ? null : SourceDocumentation(Definition(project, symbol));
    }

    public EditorSignatures PublicSignatures(string file, CancellationToken token = default, string? owningProject = null)
    {
        var (project, snapshot, _, position) = SelectPosition(new(file, 1, 1, owningProject), requireChecked: false);
        var symbols = snapshot.Symbols.ToDictionary(item => item.Id, StringComparer.Ordinal);
        var declarations = new List<DeclarationMatch>(); var omitted = 0;
        foreach (var symbol in symbols.Values.Where(item => item.Declaration is { } span && Paths.Equals(span.Path, position.Span.Path)))
        {
            token.ThrowIfCancellationRequested();
            if (symbol.Context is not { } context || !Public(symbol)) continue;
            if (declarations.Count >= 500) { omitted++; continue; }
            declarations.Add(new(project, symbol.Id, symbol.Kind, symbol.Name, context.QualifiedName,
                Bound(symbol.Signature), context.Accessibility, symbol.Declaration!));
        }
        return new(project, declarations.OrderBy(item => item.Span.Offset).ToArray(), snapshot.Coverage.IsComplete, omitted);

        bool Public(SemanticSymbol symbol)
        {
            if (symbol.Kind == "container") return true;
            if (symbol.Context?.Accessibility != "public") return false;
            for (var parent = symbol.Context.Owner; parent is not null; parent = symbols[parent].Context?.Owner)
                if (symbols[parent].Context?.Accessibility == "private") return false;
            return true;
        }
    }

    public SourceQueryResult EditorInspection(SourceSelection selection, CancellationToken token = default)
        => Inspect(selection, cancellationToken: token, includeBody: true);

    public DefinitionResult EditorDefinition(string project, string symbol) => Resolve(project, symbol);

    private string? SourceDocumentation(DefinitionResult definition)
    {
        if (definition.Project is not { } project || definition.Symbol.Context?.Documentation is not { } span) return null;
        var snapshot = projects[project].Snapshot;
        if (!snapshot.Coverage.Available(span)) return null;
        var document = snapshot.Documents.Single(item => Paths.Equals(item.Path, span.Path));
        var content = Bound(document.Text.Substring(span.Offset, span.Length));
        return string.Join('\n', content.Split('\n').Select(line => line.TrimStart().StartsWith("//", StringComparison.Ordinal) ? line.TrimStart()[2..].TrimStart() : line.TrimEnd()));
    }

    private static string Signature(string name, CallableSignature signature, SemanticSnapshot snapshot)
    {
        var types = snapshot.Types.ToDictionary(type => type.Id, type => type.Display, StringComparer.Ordinal);
        var parameters = signature.Parameters.Select(parameter => (parameter.Mode.Length == 0 ? "" : parameter.Mode + " ") +
            parameter.Name + ": " + types[parameter.Type]);
        return name + "(" + string.Join("; ", parameters) + "): " + types[signature.Result];
    }

    private static string Bound(string text) => text.Length <= 8192 ? text : text[..8191] + "…";
}
