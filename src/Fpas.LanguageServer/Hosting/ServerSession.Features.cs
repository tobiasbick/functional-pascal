using Fpas.Compiler;
using Fpas.Compiler.Formatting;
using Fpas.Compiler.Syntax;
using Fpas.LanguageServer.Protocol;
using Fpas.Workspaces.Live;
using Fpas.Workspaces.Queries;
using Newtonsoft.Json.Linq;
using OmniSharp.Extensions.LanguageServer.Protocol.Models;

namespace Fpas.LanguageServer.Hosting;

internal sealed partial class ServerSession
{
    private HashSet<int> symbolKinds = Enumerable.Range(1, 18).ToHashSet();

    private sealed record FeatureInput(SourceSelection? Selection, string? Name = null, bool IncludeDeclaration = false);
    private FeatureInput ReadFeature(string method, JObject parameters)
    {
        if (method is "textDocument/rename" or "textDocument/prepareRename" && (!renameSupported || method == "textDocument/prepareRename" && !prepareSupported))
            throw Invalid("This client did not negotiate the required rename capabilities.", -32601);
        if (method == "workspace/symbol") return new(null, RequiredString(parameters, "query"));
        var selection = Selection(parameters);
        if (method == "textDocument/references") return new(selection, IncludeDeclaration: Boolean(Object(parameters["context"], "context"), "includeDeclaration"));
        if (method != "textDocument/rename") return new(selection);
        var name = RequiredString(parameters, "newName"); _ = IdentifierFacts.Spell(name); return new(selection, name);
    }

    private object? Feature(string method, FeatureInput input, LiveSnapshot snapshot, CancellationToken token)
    {
        var query = snapshot.Queries();
        if (method == "workspace/symbol")
        {
            var result = query.Find(new(input.Name!, Limit: 1000), token);
            CompleteSearch(result);
            return result.Find!.Matches.DistinctBy(match => (match.Span.Path, match.Span.Offset, match.Kind, match.Name)).Select(match =>
            {
                var symbol = FeatureProjection.Symbol(match);
                return symbolKinds.Contains((int)symbol.Kind) ? symbol : symbol with { Kind = (SymbolKind)(symbolKinds.Contains(13) ? 13 : symbolKinds.Min()) };
            }).ToArray();
        }
        var selection = input.Selection!;
        if (method == "textDocument/references")
        {
            var result = query.Execute("refs", selection, 1000, token); CompleteSearch(result);
            if (result.Refs is null) return Array.Empty<Location>();
            var spans = result.Refs.References.Select(reference => reference.Span);
            if (input.IncludeDeclaration && result.Refs.Definition.Symbol.Declaration is { } declaration) spans = spans.Append(declaration);
            var locations = spans.DistinctBy(span => (span.Path, span.Offset, span.Length)).Select(FeatureProjection.Location).ToArray();
            if (locations.Length > 1000) throw Invalid("Result exceeds 1,000 locations. Narrow the selected project context.", -32803);
            return locations;
        }
        var definition = query.Execute("definition", selection, cancellationToken: token);
        if (method == "textDocument/definition")
            return definition.Definition is { Origin: "source", Symbol.Declaration: { } span } ? FeatureProjection.Location(span) : null;
        if (definition.Definition is not { Origin: "source", Symbol.Declaration: not null } || definition.Selection is null)
        {
            if (method == "textDocument/prepareRename") return null;
            throw Invalid("rename_not_supported: Select an explicit checked FPAS identifier.", -32803);
        }
        if (method == "textDocument/prepareRename")
        {
            var span = definition.Selection.Span;
            var original = snapshot.Documents.Document(span.Path).Text.Substring(span.Offset, span.Length).TrimStart('@');
            _ = snapshot.Rename(selection, original, token);
            return new PlaceholderRange { Range = FeatureProjection.Token(span), Placeholder = original };
        }
        var prepared = snapshot.Rename(selection, input.Name!, token);
        return FeatureProjection.WorkspaceEdits(prepared.Edits);
    }

    private object Format(JObject parameters, RequestStamp stamp, CancellationToken token)
    {
        var path = DocumentPath(parameters); var options = Object(parameters["options"], "options");
        if (RequiredInteger(options, "tabSize") < 1) throw Invalid("tabSize must be positive.");
        _ = Boolean(options, "insertSpaces");
        var document = OpenDocument(path, stamp);
        var formatted = FpasFormatter.Format(new(path, document.Text), token);
        if (!formatted.Success) throw Invalid("format_failed: " + string.Join("; ", formatted.Diagnostics.Select(diagnostic => diagnostic.Code + ": " + diagnostic.Message)), -32803);
        return formatted.Text == document.Text ? Array.Empty<TextEdit>() : new[] { new TextEdit { Range = new(new(0, 0), FeatureProjection.End(document.Text)), NewText = formatted.Text! } };
    }

    private Fpas.Workspaces.Snapshots.TextDocumentSnapshot OpenDocument(string path, RequestStamp stamp)
    {
        lock (gate)
        {
            CheckRequest(stamp);
            if (desynchronized.Contains(path)) throw Invalid("Resynchronize this document before requesting its features.", -32803);
            return workspace!.OpenDocuments.Documents.FirstOrDefault(document => PathComparer.Equals(document.Path, path))
                ?? throw Invalid("Open the document before requesting its features.", -32803);
        }
    }

    private static void CompleteSearch(SourceQueryResult result)
    {
        if (result.Coverage?.Complete != true) throw Invalid("analysis_unavailable: Search coverage is incomplete. Repair unavailable source/project inputs before this search.", -32803);
        if (result.Truncated) throw Invalid("Result exceeds 1,000 items. Narrow the search or selected project context.", -32803);
    }
    private static JObject Object(JToken? value, string name) => value as JObject ?? throw Invalid("Supply object " + name + ".");
    private static bool Boolean(JObject value, string name) => value[name]?.Type == JTokenType.Boolean ? value.Value<bool>(name) : throw Invalid("Supply boolean " + name + ".");
    private static string DocumentPath(JObject parameters) => FileUris.Path(new Uri(RequiredString(Object(parameters["textDocument"], "textDocument"), "uri"), UriKind.RelativeOrAbsolute));
    private static SourceSelection Selection(JObject parameters)
    {
        var path = DocumentPath(parameters); var position = Object(parameters["position"], "position");
        var line = RequiredInteger(position, "line"); var character = RequiredInteger(position, "character");
        if (line < 0 || character < 0) throw Invalid("Positions must be nonnegative UTF-16 coordinates.");
        return new(path, checked(line + 1), checked(character + 1));
    }
}
