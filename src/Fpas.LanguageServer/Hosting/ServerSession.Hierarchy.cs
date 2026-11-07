using Fpas.Compiler.Analysis;
using Fpas.LanguageServer.Protocol;
using Fpas.Workspaces.Live;
using Fpas.Workspaces.Queries;
using Newtonsoft.Json.Linq;
using OmniSharp.Extensions.LanguageServer.Protocol.Models;

namespace Fpas.LanguageServer.Hosting;

internal sealed partial class ServerSession
{
    private object Hierarchy(string method, JObject parameters, LiveSnapshot snapshot, CancellationToken token)
    {
        var query = snapshot.Queries();
        if (method == "textDocument/prepareCallHierarchy")
        {
            var selection = Selection(parameters);
            var target = query.Execute("definition", selection, cancellationToken: token).Definition;
            if (target is { Origin: "source", Project: not null, Symbol.Kind: "routine" or "property" or "union_case", Symbol.Declaration: not null })
                return new[] { CallItem(snapshot, target.Project, target, null) };
            return Array.Empty<object>();
        }

        var data = Object(Object(parameters["item"], "item")["data"], "data");
        CheckItem(Object(data["stamp"], "stamp"), snapshot);
        var project = RequiredString(data, "project");
        var facts = Facts(snapshot, project);
        var bodyId = data.Value<string>("body");
        var body = bodyId is null ? null : facts.Bodies.FirstOrDefault(candidate => candidate.Id == bodyId)
            ?? throw Invalid("Unknown hierarchy body.");
        var symbol = data.Value<string>("symbol");
        if (symbol is not null && !facts.Symbols.Any(candidate => candidate.Id == symbol)) throw Invalid("Unknown hierarchy declaration.");
        var definition = symbol is null ? null : query.EditorDefinition(project, symbol);
        var span = definition?.Symbol.Declaration ?? body?.Span ?? throw Invalid("Unknown hierarchy declaration.");
        if (body is not null && body.Symbol is not null && body.Symbol != symbol) throw Invalid("Hierarchy body does not match the declaration.");
        var incoming = method == "callHierarchy/incomingCalls";
        if (incoming && body is { Symbol: null, Kind: not "lambda" }) return Array.Empty<object>();
        var result = query.Calls(incoming ? "callers" : "callees", new(span.Path, span.Line, span.Column, project),
            body: body is not null, limit: 1000, cancellationToken: token);
        var page = result.Callers?.Page ?? result.Callees?.Page;
        var sites = page?.Sites ?? [];
        var direct = sites.Where(site => site.Relationship == "invocation" && site.Site.Dispatch == "direct" && site.Site.UnknownReason is null);
        if (result.Coverage?.Complete != true || result.Truncated || (result.Callers?.Summary ?? result.Callees?.Summary)?.Unresolved.Count > 0)
            outgoing.Enqueue("window/logMessage", new LogMessageParams { Type = MessageType.Warning,
                Message = "FPAS call hierarchy shows proven direct calls. Some source regions or call targets could not be determined." },
                () => State == SessionState.Running);
        if (incoming)
            return direct.GroupBy(site => (site.Project, site.Owner.Id)).Select(group =>
            {
                var owner = group.First().Owner;
                var source = owner.Symbol is { } ownerSymbol ? query.EditorDefinition(group.Key.Project, ownerSymbol) : null;
                return new { from = CallItem(snapshot, group.Key.Project, source, source is null ? owner : null),
                    fromRanges = group.Select(site => SiteRange(snapshot, site)).ToArray() };
            }).ToArray();
        return direct.Where(site => site.Target is { Origin: "source", Project: not null, Symbol.Declaration: not null })
            .GroupBy(site => (site.Target!.Project!, site.Target.Symbol.Id)).Select(group => new
            {
                to = CallItem(snapshot, group.Key.Item1, group.First().Target, null),
                fromRanges = group.Select(site => SiteRange(snapshot, site)).ToArray()
            }).ToArray();
    }

    private object CallItem(LiveSnapshot snapshot, string project, DefinitionResult? definition, SourceBody? body)
    {
        var facts = Facts(snapshot, project);
        var symbol = definition?.Symbol;
        var name = symbol?.Name ?? (body?.Kind == "lambda" ? "lambda" : "program body");
        var selection = symbol?.Declaration ?? body!.Span;
        var range = symbol?.Context?.Span ?? body?.Span ?? selection;
        var document = facts.Documents.Single(source => PathComparer.Equals(source.Path, selection.Path));
        return new { name, kind = SymbolKind(symbol?.Kind ?? "routine"), uri = FileUris.Uri(selection.Path),
            range = FeatureProjection.Range(document, range), selectionRange = FeatureProjection.Range(document, selection),
            detail = snapshot.Analysis.IsComplete ? "Direct calls" : "Direct calls (partial coverage)",
            data = new { stamp = ItemStamp(snapshot), project, symbol = symbol?.Id, body = body?.Id } };
    }

    private static object SiteRange(LiveSnapshot snapshot, CallSiteResult site)
    {
        var span = site.Site.Anchor ?? site.Site.Span;
        return FeatureProjection.Range(Facts(snapshot, site.Project).Documents.Single(source => PathComparer.Equals(source.Path, span.Path)), span);
    }

    private static SemanticSnapshot Facts(LiveSnapshot snapshot, string project)
        => snapshot.Analysis.Projects.FirstOrDefault(item => PathComparer.Equals(item.Project, project))?.Result?.Snapshot
            ?? throw Invalid("The item's project has no checked snapshot.", -32803);
}
