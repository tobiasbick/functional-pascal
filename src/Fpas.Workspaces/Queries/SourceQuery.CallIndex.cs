using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Queries;

public sealed partial class SourceQuery
{
    private sealed record IndexedCall(string Project, SemanticSnapshot Snapshot, SourceBody Owner, SourceOperation Site, string? Target);
    private sealed record CallIndex(IReadOnlyDictionary<(string Project, string Body), IndexedCall[]> Outgoing,
        IReadOnlyDictionary<string, IndexedCall[]> Incoming, SourceOperation[] Operations, ExcludedOperation[] Excluded);
    private CallIndex? callIndex;

    private CallIndex CallsIndex(CancellationToken token)
    {
        if (callIndex is not null) return callIndex;
        var sites = new List<IndexedCall>(); var excluded = new List<ExcludedOperation>();
        foreach (var (project, (_, snapshot)) in projects.OrderBy(item => item.Key, StringComparer.Ordinal))
        {
            token.ThrowIfCancellationRequested();
            var owners = snapshot.Bodies.ToDictionary(body => body.Id, StringComparer.Ordinal);
            foreach (var site in snapshot.Operations)
            {
                token.ThrowIfCancellationRequested();
                var target = site.TargetBody is not null ? project + ":body:" + site.TargetBody : site.Target is { } symbol ? Canonical(project, snapshot.Symbol(symbol)) : null;
                sites.Add(new(project, snapshot, owners[site.Body], site, target));
            }
            excluded.AddRange(snapshot.ExcludedOperations);
        }
        var outgoing = sites.GroupBy(site => (site.Project, site.Site.Body)).ToDictionary(group => group.Key, group => group.ToArray());
        var incoming = sites.Where(site => site.Target is not null).GroupBy(site => site.Target!, StringComparer.Ordinal)
            .ToDictionary(group => group.Key, group => group.ToArray(), StringComparer.Ordinal);
        token.ThrowIfCancellationRequested();
        return callIndex = new(outgoing, incoming, sites.Select(site => site.Site).ToArray(), excluded.ToArray());
    }

    private CallSiteResult Site(IndexedCall item)
    {
        var (project, snapshot, owner, site, _) = item;
        var target = site.Target is null ? null : Definition(project, snapshot.Symbol(site.Target));
        var roots = (site.Signature?.Parameters.Select(parameter => parameter.Type) ?? []).Concat(site.Signature?.TypeParameters ?? [])
            .Concat(new[] { site.Signature?.Result, site.OwnerType, site.ResultType, site.Receiver?.Type });
        return new(project, owner, site, site.Kind == "callable_reference" ? "callable_reference" : "invocation", target, Types(snapshot, roots));
    }

    private CallSitePage Page(IEnumerable<IndexedCall> sites, int limit, int offset, CancellationToken token)
    {
        var ordered = sites.Select(site => { token.ThrowIfCancellationRequested(); return site; })
            .OrderBy(site => site.Project, StringComparer.Ordinal).ThenBy(site => site.Site.Span.Path, StringComparer.Ordinal)
            .ThenBy(site => site.Site.Span.Offset).ThenBy(site => site.Site.Kind, StringComparer.Ordinal)
            .ThenBy(site => site.Site.Operation?.GetKey() ?? site.Target, StringComparer.Ordinal).ToArray();
        var page = ordered.Skip(offset).Take(limit).Select(Site).ToArray(); var next = (long)offset + page.Length;
        var references = ordered.Count(site => site.Site.Kind == "callable_reference");
        return new(Array.AsReadOnly(page), ordered.Length, page.Length, offset, next < ordered.Length ? (int)next : null, ordered.Length - references, references);
    }
}
