using Fpas.Compiler.Analysis;
using Fpas.LanguageServer.Protocol;
using Fpas.Workspaces.Documentation;
using Fpas.Workspaces.Live;
using Fpas.Workspaces.Queries;
using Newtonsoft.Json.Linq;

namespace Fpas.LanguageServer.Hosting;

internal sealed partial class ServerSession
{
    private static readonly string[] EditorMethods = ["textDocument/hover", "textDocument/typeDefinition",
        "textDocument/documentHighlight", "textDocument/semanticTokens/full", "textDocument/prepareCallHierarchy",
        "callHierarchy/incomingCalls", "callHierarchy/outgoingCalls", "textDocument/codeAction", "codeAction/resolve",
        "textDocument/completion", "completionItem/resolve", "textDocument/signatureHelp",
        "fpas/publicSignatures", "fpas/inspect", "fpas/searchMetadata",
        "fpas/prepareRefactoring", "fpas/applyRefactoring", "fpas/discardRefactoring"];
    private static readonly string[] TokenTypes = ["namespace", "type", "class", "enum", "interface", "struct",
        "typeParameter", "parameter", "variable", "property", "enumMember", "event", "function", "method"];
    private string[] semanticTypes = [];
    private string[] semanticModifiers = [];
    private readonly string editorSession = Guid.NewGuid().ToString("N");
    private bool viewsPublished;

    private void EditorCapabilities(JObject server, JObject? client)
    {
        hierarchicalSymbols = (client?["documentSymbol"] as JObject)?.Value<bool>("hierarchicalDocumentSymbolSupport") == true;
        foreach (var feature in new[] { "documentSymbol", "foldingRange", "selectionRange", "hover", "typeDefinition",
            "documentHighlight", "callHierarchy" })
            server[feature + "Provider"] = true;
        if (renameSupported)
            server["codeActionProvider"] = new JObject { ["resolveProvider"] = !correctionCommand, ["codeActionKinds"] = new JArray("quickfix") };
        server["completionProvider"] = new JObject { ["resolveProvider"] = true, ["triggerCharacters"] = new JArray(".") };
        server["signatureHelpProvider"] = new JObject { ["triggerCharacters"] = new JArray("(", ","), ["retriggerCharacters"] = new JArray(",", ")") };
        if (client?["semanticTokens"] is JObject semantic && semantic["tokenTypes"] is JArray types &&
            (semantic["requests"] as JObject)?["full"] is { } full && (full.Type == JTokenType.Object || full.Type == JTokenType.Boolean && full.Value<bool>()))
        {
            semanticTypes = TokenTypes.Intersect(types.Values<string>().OfType<string>(), StringComparer.Ordinal).ToArray();
            semanticModifiers = (semantic["tokenModifiers"] as JArray)?.Values<string>().Contains("declaration") == true ? ["declaration"] : [];
            if (semanticTypes.Length != 0)
                server["semanticTokensProvider"] = new JObject { ["full"] = true, ["legend"] = new JObject
                    { ["tokenTypes"] = new JArray(semanticTypes), ["tokenModifiers"] = new JArray(semanticModifiers) } };
        }
        server["experimental"] = new JObject { ["fpasSourceViews"] = 1, ["fpasRefactoring"] = renameSupported ? 1 : 0 };
    }

    private object? EditorFeature(string method, JObject parameters, LiveSnapshot snapshot, CancellationToken token)
    {
        var query = snapshot.Queries();
        return method switch
        {
            "textDocument/hover" => Hover(query, Selection(parameters), token),
            "textDocument/typeDefinition" => query.TypeDefinition(Selection(parameters)) is { } target ? FeatureProjection.Location(target) : null,
            "textDocument/documentHighlight" => query.Highlights(Selection(parameters), token).Select(item => new
                { range = FeatureProjection.Token(item.Span), kind = item.Role == "declaration" ? 1 : item.Role == "write" ? 3 : 2 }).ToArray(),
            "textDocument/semanticTokens/full" => SemanticTokens(query, DocumentPath(parameters), token),
            "textDocument/prepareCallHierarchy" or "callHierarchy/incomingCalls" or "callHierarchy/outgoingCalls" =>
                Hierarchy(method, parameters, snapshot, token),
            "textDocument/codeAction" or "codeAction/resolve" => CodeActions(method, parameters, snapshot, token),
            "textDocument/completion" or "completionItem/resolve" => Completion(method, parameters, snapshot, token),
            "textDocument/signatureHelp" => SignatureHelp(Selection(parameters), snapshot, token),
            "fpas/prepareRefactoring" or "fpas/applyRefactoring" or "fpas/discardRefactoring" => Refactor(method, parameters, snapshot, token),
            _ => Views(method, parameters, snapshot, token)
        };
    }

    private static object? Hover(SourceQuery query, SourceSelection selection, CancellationToken token)
    {
        var hover = query.Hover(selection, token);
        return hover is null ? null : new { range = FeatureProjection.Token(hover.Span),
            contents = new { kind = "markdown", value = SignatureMarkdown(hover.Signature, hover.Documentation) } };
    }

    private static string SignatureMarkdown(string signature, string? documentation)
    {
        var content = "```fpas\n" + signature.Replace("`", "`\u200b", StringComparison.Ordinal) + "\n```";
        if (documentation is null) return content;
        return content + "\n\n" + string.Concat(documentation.Select(character =>
            "\\`*_{}[]<>()#+-.!|~&".Contains(character) ? "\\" + character : character.ToString()));
    }

    private object SemanticTokens(SourceQuery query, string path, CancellationToken token)
    {
        var data = new List<int>();
        if (semanticTypes.Length == 0) return new { data };
        var line = 1; var column = 1; var end = -1;
        foreach (var item in query.Classifications(path, token).OrderBy(item => item.Span.Offset).ThenByDescending(item => item.Declaration))
        {
            token.ThrowIfCancellationRequested();
            var span = item.Span;
            if (span.Offset < end || span.Length == 0) continue;
            var kind = item.Kind switch
            {
                "container" or "alias" => "namespace", "record" => "struct", "union" or "domain" => "enum",
                "interface" => "interface", "type" => "type", "type_parameter" => "typeParameter",
                "parameter" => "parameter", "field" or "property" => "property", "enum_member" or "union_case" => "enumMember",
                "routine" => "function", "method" => "method", "event" => "event", _ => "variable"
            };
            var type = Array.IndexOf(semanticTypes, kind);
            if (type < 0) continue;
            var delta = span.Line - line;
            data.AddRange([delta, delta == 0 ? span.Column - column : span.Column - 1, span.Length, type,
                item.Declaration && semanticModifiers.Length != 0 ? 1 : 0]);
            line = span.Line; column = span.Column; end = span.Offset + span.Length;
        }
        return new { data };
    }

    private object Views(string method, JObject parameters, LiveSnapshot snapshot, CancellationToken token)
    {
        if (RequiredInteger(parameters, "schemaVersion") != 1) throw Invalid("Use FPAS source view schema 1.");
        var selection = EditorSelection(parameters, snapshot, method == "fpas/inspect");
        var query = snapshot.Queries();
        object value = method switch
        {
            "fpas/publicSignatures" => query.PublicSignatures(selection.File, token, selection.Project),
            "fpas/inspect" => query.EditorInspection(selection, token),
            "fpas/searchMetadata" => SearchMetadata(parameters, selection, snapshot, token),
            _ => throw Invalid("Unknown editor feature.", -32601)
        };
        lock (gate) { CheckRequest(request.Value!); viewsPublished = true; }
        return new { schemaVersion = 1, snapshotId = snapshot.Analysis.Context.SnapshotId,
            version = snapshot.Documents.Document(selection.File).Version, revision = snapshot.Revision, value };
    }

    private static DocumentationResult SearchMetadata(JObject parameters, SourceSelection selection, LiveSnapshot snapshot, CancellationToken token)
    {
        var project = snapshot.Queries().EditorOwner(selection);
        var name = RequiredString(parameters, "query");
        if (name.Length is < 1 or > 128) throw Invalid("Supply a metadata name of at most 128 characters.");
        var docs = new DocumentationQuery(snapshot.Analysis);
        var result = docs.Search(new(name, "contains", project, Limit: 50), token, metadataOnly: true);
        docs.VerifyInputs(token);
        return result;
    }

    private static SourceSelection EditorSelection(JObject parameters, LiveSnapshot snapshot, bool position)
    {
        var selection = position ? Selection(parameters) : new SourceSelection(DocumentPath(parameters), 1, 1);
        var document = snapshot.Documents.Document(selection.File);
        if (document.Version != RequiredInteger(Object(parameters["textDocument"], "textDocument"), "version"))
            throw Invalid("The requested document version has changed.", -32801);
        var project = parameters["projectUri"] is null ? null : FileUris.Path(new Uri(RequiredString(parameters, "projectUri")));
        return selection with { Project = project };
    }

    private object ItemStamp(LiveSnapshot snapshot) => new
        { session = editorSession, revision = snapshot.Revision, snapshotId = snapshot.Analysis.Context.SnapshotId };

    private void CheckItem(JObject item, LiveSnapshot snapshot)
    {
        if (item.Value<string>("session") != editorSession || item["revision"]?.Type != JTokenType.Integer || item.Value<long>("revision") != snapshot.Revision ||
            item.Value<string>("snapshotId") != snapshot.Analysis.Context.SnapshotId)
            throw Invalid("The item belongs to a stale workspace snapshot.", -32801);
    }
}
