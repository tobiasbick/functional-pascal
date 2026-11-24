// [Documentation](../../../docs/pascal/tools/language-server.md#navigation-and-edits)
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Documentation;
using Fpas.Workspaces.Loading;

namespace Fpas.Workspaces.Queries;

/// <summary>Contains bounded hover text for a checked identifier.</summary>
/// <param name="Span">Identifier token span.</param>
/// <param name="Signature">Checked signature, including actual call substitutions when available.</param>
/// <param name="Documentation">Source Markdown or escaped metadata text, when present.</param>
public sealed record EditorHover(SourceSpan Span, string Signature, string? Documentation);
/// <summary>Classifies one checked identifier occurrence.</summary>
/// <param name="Span">Identifier token span.</param>
/// <param name="Kind">Semantic symbol kind.</param>
/// <param name="Declaration">Whether the occurrence declares the symbol.</param>
public sealed record EditorClassification(SourceSpan Span, string Kind, bool Declaration);
/// <summary>Contains a bounded public-declaration view for one source owner.</summary>
/// <param name="Project">Owning project path.</param>
/// <param name="Declarations">Public source declarations in source order.</param>
/// <param name="Complete">Whether the owning semantic snapshot has complete coverage.</param>
/// <param name="Omitted">Declarations omitted by the 500-item limit.</param>
public sealed record EditorSignatures(string Project, IReadOnlyList<DeclarationMatch> Declarations, bool Complete, int Omitted);

/// <remarks>Editor views adapt detached compiler facts and preserve explicit absence and coverage states.</remarks>
public sealed partial class SourceQuery
{
    /// <summary>Gets the selected identifier's checked signature and available documentation.</summary>
    /// <param name="selection">Checked source position.</param>
    /// <param name="token">Cancels metadata documentation reads.</param>
    /// <returns>Bounded hover text, or null when no symbol occurs at the position.</returns>
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

    /// <summary>Locates the source declaration of a checked expression's type.</summary>
    /// <param name="selection">Checked source position.</param>
    /// <returns>A source type or type-parameter declaration, or null without editable source.</returns>
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

    /// <summary>Finds checked occurrences of the selected identity in the same physical document.</summary>
    /// <param name="selection">Checked symbol position.</param>
    /// <param name="token">Cancels occurrence traversal.</param>
    /// <returns>At most 1,000 occurrences; larger results fail explicitly.</returns>
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

    /// <summary>Classifies available checked identifier occurrences without requiring full-document coverage.</summary>
    /// <param name="file">Unambiguously owned source path.</param>
    /// <param name="token">Cancels occurrence traversal.</param>
    /// <returns>At most 10,000 classifications; larger results fail explicitly.</returns>
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

    /// <summary>Resolves one source owner even when the selected region is unchecked.</summary>
    /// <param name="selection">Source position and optional linked-file owner.</param>
    /// <returns>The analyzed owning project path.</returns>
    public string EditorOwner(SourceSelection selection) => SelectPosition(selection, requireChecked: false).Project;

    /// <summary>Computes compiler-owned completions using this snapshot's captured inputs.</summary>
    /// <param name="selection">Completion insertion position, possibly in incomplete syntax.</param>
    /// <param name="token">Cancels completion analysis.</param>
    /// <returns>Checked candidates and the exact replacement range.</returns>
    public EditorCompletion Complete(SourceSelection selection, CancellationToken token = default)
        => Assist(analysis, "complete", selection, token).Complete!;

    /// <summary>Computes signature-help candidates at a call position.</summary>
    /// <param name="selection">Source position in the argument list.</param>
    /// <param name="token">Cancels signature analysis.</param>
    /// <returns>Compiler-owned signatures and active-argument information.</returns>
    public EditorCallSignatures Signatures(SourceSelection selection, CancellationToken token = default)
        => Assist(analysis, "signature", selection, token).Signature!;

    /// <summary>Requests compiler editor assistance even if the selected source contains incomplete syntax.</summary>
    /// <param name="analysis">Workspace inputs and project availability.</param>
    /// <param name="operation">complete or signature.</param>
    /// <param name="selection">Source cursor and optional linked-file owner.</param>
    /// <param name="token">Cancels compiler assistance.</param>
    /// <returns>The editor result with its cursor and overall checked coverage.</returns>
    public static SourceQueryResult Assist(WorkspaceAnalysis analysis, string operation, SourceSelection selection, CancellationToken token = default)
    {
        var (project, source) = SelectOwner(analysis, selection);
        if (analysis.Projects.Any(item => Paths.Equals(item.Project, project.Path) && item.State == "blocked"))
            throw new ContextFailure("analysis_unavailable", "The selected project has unavailable analysis inputs.", project: project.Path);
        var document = new SourceDocument(source.Path, source.Text, token);
        var offset = Offset(document, selection);
        var result = new SourceQueryResult("ok", null, new(project.Path, document.Span(offset, 0)), null, null, null, false, [],
            Coverage: analysis.SemanticCoverage);
        return operation switch
        {
            "complete" => result with { Complete = EditorAnalysis.Complete(project.Request, source.Path, offset, token) },
            "signature" => result with { Signature = EditorAnalysis.Signatures(project.Request, source.Path, offset, token) },
            _ => throw new ContextFailure("invalid_argument", "Unknown editor query operation.")
        };
    }

    /// <summary>Reads documentation for an editor candidate's checked metadata or source identity.</summary>
    /// <param name="selection">Position determining the owning compilation.</param>
    /// <param name="identity">Optional metadata identity, preferred over a source span.</param>
    /// <param name="declaration">Optional source declaration span.</param>
    /// <param name="token">Cancels metadata documentation reads.</param>
    /// <returns>Bounded Markdown-compatible text, or null when absent.</returns>
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

    /// <summary>Lists explicit public declarations whose containing owners remain accessible.</summary>
    /// <param name="file">Source file path.</param>
    /// <param name="token">Cancels declaration traversal.</param>
    /// <param name="owningProject">Optional linked-file project selection.</param>
    /// <returns>A bounded source-order signature view with coverage and omission counts.</returns>
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

    /// <summary>Inspects a declaration including its selected body text.</summary>
    /// <param name="selection">Checked source position.</param>
    /// <param name="token">Cancels inspection.</param>
    /// <returns>The standard bounded inspection result with body text enabled.</returns>
    public SourceQueryResult EditorInspection(SourceSelection selection, CancellationToken token = default)
        => Inspect(selection, cancellationToken: token, includeBody: true);

    /// <summary>Resolves a snapshot-local symbol through checked source/export mappings.</summary>
    /// <param name="project">Analyzed project path.</param>
    /// <param name="symbol">Symbol identity from that project's semantic snapshot.</param>
    /// <returns>The canonical source, metadata or implicit definition.</returns>
    public DefinitionResult EditorDefinition(string project, string symbol) => Resolve(project, symbol);

    private string? SourceDocumentation(DefinitionResult definition)
    {
        if (definition.Project is not { } project || definition.Symbol.Context?.Documentation is not { } span) return null;
        var snapshot = projects[project].Snapshot;
        if (!snapshot.Coverage.Available(span)) return null;
        var document = snapshot.Documents.Single(item => Paths.Equals(item.Path, span.Path));
        var content = Bound(document.Text.Substring(span.Offset, span.Length));
        return DocumentationMarkdown.Source(content);
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
