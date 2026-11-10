using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Loading;

namespace Fpas.Workspaces.Queries;

public sealed partial class SourceQuery
{
    private static readonly CallOperationPolicy OperationPolicy = new("explicit_source_operations_v1",
        Array.AsReadOnly(new[] { "invoke", "construct", "property_get", "property_set", "event_add", "event_remove", "operator", "callable_reference", "delegate_invoke" }),
        Array.AsReadOnly(new[] { "generated_infrastructure", "source_storage", "builtin_operators", "array_allocation", "attribute_construction", "using_protocol", "async_using_protocol", "foreach_protocol", "await_protocol" }));
    private sealed record CallTarget(QuerySelection Position, string Project, SemanticSnapshot Snapshot,
        DefinitionResult? Definition, SourceBody? Body, MetadataIdentity? Operation, string? Key, string? Failure);

    public SourceQueryResult Calls(string operation, SourceSelection selection, bool body = false, int limit = 100, int offset = 0,
        CancellationToken cancellationToken = default)
        => CallsCore(operation, selection, body, limit, offset, cancellationToken) with { Coverage = coverage };

    private SourceQueryResult CallsCore(string operation, SourceSelection selection, bool body, int limit, int offset, CancellationToken cancellationToken)
    {
        if (operation is not ("callees" or "callers")) throw new ContextFailure("invalid_argument", "Unknown call-query operation.");
        if (limit is < 1 or > 1000 || offset < 0) throw new ContextFailure("invalid_argument", "--limit must be 1..1000 and --offset must be nonnegative.");
        cancellationToken.ThrowIfCancellationRequested();
        var target = SelectCallTarget(selection, body, operation == "callers");
        var boundaries = BinaryBoundaries();
        if (target.Failure is { } reason) return new("not_found", reason, target.Position, null, null, null, false, boundaries);
        var index = CallsIndex(cancellationToken);
        var mode = body ? "body" : "target";
        if (operation == "callees")
        {
            var available = target.Body is not null;
            var availability = available ? "available" : target.Definition?.Origin == "metadata" ? "metadata_only" : "no_body";
            var sites = available ? index.Outgoing.GetValueOrDefault((target.Project, target.Body!.Id)) ?? [] : [];
            var page = available ? Page(sites, limit, offset, cancellationToken) : null;
            var summary = available ? Summary(sites.Select(site => site.Site), target.Snapshot.ExcludedOperations.Where(site => site.Body == target.Body!.Id)) : null;
            return new("ok", null, target.Position, null, null, null, page is not null && page.ReturnedCount < page.TotalCount, boundaries,
                Callees: new(mode, target.Definition, target.Operation, target.Body, availability, OperationPolicy, summary, page));
        }
        var matches = (index.Incoming.GetValueOrDefault(target.Key!) ?? []).Where(site => target.Operation is null ||
            site.Site.Operation is { } identity && MetadataKey(identity) == MetadataKey(target.Operation));
        var incoming = Page(matches, limit, offset, cancellationToken);
        return new("ok", null, target.Position, null, null, null, incoming.ReturnedCount < incoming.TotalCount, boundaries,
            Searched: Array.AsReadOnly(projects.Keys.Order(StringComparer.Ordinal).ToArray()),
            Callers: new(mode, target.Definition, target.Operation, target.Body, OperationPolicy, Summary(index.Operations, index.Excluded), incoming));
    }

    private CallTarget SelectCallTarget(SourceSelection selection, bool bodyMode, bool incoming)
    {
        var (project, snapshot, offset, position) = SelectPosition(selection);
        SourceBody? body = null; DefinitionResult? definition = null; MetadataIdentity? operation = null; string? key = null;
        if (bodyMode)
        {
            var bodies = snapshot.Bodies.ToDictionary(body => body.Id, StringComparer.Ordinal);
            body = snapshot.Bodies.Where(body => body.Regions.Any(region => Paths.Equals(region.Path, position.Span.Path) && region.Contains(offset)))
                .OrderBy(body => body.Regions.Where(region => region.Contains(offset)).Min(region => region.Length))
                .ThenByDescending(Depth).FirstOrDefault();
            if (body is null) return Missing("no_body");
            position = position with { Span = body.Span };
            if (body.Symbol is { } symbol) definition = Definition(project, snapshot.Symbol(symbol));
            if (incoming && body.Kind is not ("routine" or "lambda")) return Missing("not_callable_target");
            key = definition is null ? project + ":body:" + body.Id : Canonical(project, definition.Symbol);

            int Depth(SourceBody candidate)
            {
                var depth = 0;
                while (candidate.Parent is { } parent) { candidate = bodies[parent]; depth++; }
                return depth;
            }
        }
        else
        {
            var site = snapshot.Operations.Where(site => site.Anchor is { } anchor && Paths.Equals(anchor.Path, position.Span.Path) && anchor.Contains(offset))
                .OrderBy(site => site.Span.Length).ThenBy(site => site.Kind, StringComparer.Ordinal).FirstOrDefault();
            if (site?.Target is { } called)
            {
                definition = Definition(project, snapshot.Symbol(called));
                position = position with { Span = site.Anchor! };
                if (site.Kind is "construct" or "property_get" or "property_set" or "event_add" or "event_remove") operation = site.Operation;
            }
            else
            {
                var occurrence = snapshot.OccurrenceAt(position.Span.Path, offset);
                if (occurrence is null) return Missing("no_symbol");
                var symbol = snapshot.Symbol(occurrence.Symbol);
                if (symbol.Kind is not ("routine" or "property" or "variant_case")) return Missing("not_callable_target");
                definition = Definition(project, symbol); position = position with { Span = occurrence.Span };
            }
            var owner = definition.Project ?? project;
            snapshot = projects[owner].Snapshot; project = owner;
            if (!incoming && definition.Symbol.Context?.Body is { } declaredBody && !snapshot.Coverage.Available(declaredBody))
                throw new ContextFailure("analysis_unavailable", "The selected callable body could not be checked in this snapshot.");
            body = snapshot.Bodies.SingleOrDefault(body => body.Symbol == definition.Symbol.Id);
            key = Canonical(project, definition.Symbol);
        }
        return new(position, project, snapshot, definition, body, operation, key, null);

        CallTarget Missing(string reason) => new(position, project, snapshot, definition, body, operation, null, reason);
    }

    private static OperationSummary Summary(IEnumerable<SourceOperation> sites, IEnumerable<ExcludedOperation> excluded)
    {
        var all = sites.ToArray();
        static IReadOnlyDictionary<string, int> Count(IEnumerable<string> kinds) => new System.Collections.ObjectModel.ReadOnlyDictionary<string, int>(
            kinds.GroupBy(kind => kind).OrderBy(group => group.Key, StringComparer.Ordinal).ToDictionary(group => group.Key, group => group.Count(), StringComparer.Ordinal));
        return new(Count(all.Select(site => site.Kind)), Count(excluded.Select(site => site.Kind)), Count(all.Select(site => site.UnknownReason).OfType<string>()));
    }
}
