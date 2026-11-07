using Fpas.Compiler.Analysis;
using Fpas.Compiler.Syntax;
using Fpas.LanguageServer.Protocol;
using Fpas.Workspaces.Live;
using Fpas.Workspaces.Queries;
using Newtonsoft.Json.Linq;

namespace Fpas.LanguageServer.Hosting;

internal sealed partial class ServerSession
{
    private object Completion(string method, JObject parameters, LiveSnapshot snapshot, CancellationToken token)
    {
        var query = snapshot.Queries();
        var resolving = method == "completionItem/resolve";
        var data = resolving ? Object(parameters["data"], "data") : parameters;
        if (resolving) CheckItem(Object(data["stamp"], "stamp"), snapshot);
        var selection = Selection(data);
        var completion = query.Complete(selection, token);
        if (!resolving)
        {
            var stamp = ItemStamp(snapshot);
            return new { isIncomplete = completion.Incomplete, items = completion.Items.Select((candidate, index) =>
                CompletionItem(candidate, completion, new { stamp, index, textDocument = data["textDocument"], position = data["position"] })).ToArray() };
        }
        var index = RequiredInteger(data, "index");
        if (index < 0 || index >= completion.Items.Count) throw Invalid("Unknown completion candidate.");
        var item = completion.Items[index];
        return CompletionItem(item, completion, data, query.EditorDocumentation(selection, item.Metadata, item.Declaration, token));
    }

    private static object CompletionItem(EditorCandidate item, EditorCompletion completion, object data, string? documentation = null)
    {
        string name;
        try { name = IdentifierFacts.Spell(item.Name); }
        catch (ArgumentException) { name = item.Name; }
        var kind = item.Kind switch
        {
            "routine" => 3, "field" => 5, "variable" or "parameter" => 6, "type" => 7,
            "namespace" => 9, "property" => 10, _ => 1
        };
        return new { label = item.Name, kind, detail = Limit(item.Signature),
            textEdit = new { range = FeatureProjection.Token(completion.Replacement), newText = name }, data,
            documentation = documentation is null ? null : new { kind = "markdown", value = SignatureMarkdown(Limit(item.Signature), documentation) } };
    }

    private static object? SignatureHelp(SourceSelection selection, LiveSnapshot snapshot, CancellationToken token)
    {
        var query = snapshot.Queries();
        var result = query.Signatures(selection, token);
        if (result.Items.Count == 0) return null;
        return new
        {
            signatures = result.Items.Select((signature, index) =>
            {
                var documentation = index == result.ActiveSignature ? query.EditorDocumentation(selection, signature.Metadata, null, token) : null;
                return new { label = Limit(signature.Label),
                    parameters = signature.Parameters.Select(parameter => new { label = Limit(parameter) }).ToArray(),
                    activeParameter = signature.ActiveParameter,
                    documentation = documentation is null ? null : new { kind = "markdown", value = SignatureMarkdown(Limit(signature.Label), documentation) } };
            }).ToArray(),
            activeSignature = result.ActiveSignature,
            activeParameter = result.ActiveSignature is { } selected ? result.Items[selected].ActiveParameter : null
        };
    }

    private static string Limit(string value) => value.Length <= 8192 ? value : value[..8191] + "…";
}
