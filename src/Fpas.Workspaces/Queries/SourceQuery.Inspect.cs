using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Loading;
using System.Text;

namespace Fpas.Workspaces.Queries;

public sealed partial class SourceQuery
{
    public SourceQueryResult Inspect(SourceSelection selection, int limit = 20, CancellationToken cancellationToken = default)
    {
        if (limit is < 1 or > 100) throw new ContextFailure("invalid_argument", "Related declaration limit must be 1..100.");
        var selected = Execute("definition", selection, cancellationToken: cancellationToken);
        if (selected.Definition is not { } definition) return selected;
        var project = definition.Project ?? selected.Selection!.Project;
        var snapshot = projects[project].Snapshot;
        var symbol = definition.Symbol;
        var context = symbol.Context;
        var surface = Types(snapshot, (context?.SurfaceTypes ?? []).Append(symbol.Type));
        var checkedTypes = Execute("type", selection, cancellationToken: cancellationToken).Type?.Types ?? [];
        var types = surface.Concat(definition.Types).Concat(checkedTypes).DistinctBy(type => type.Id).OrderBy(type => type.Id, StringComparer.Ordinal).ToArray();
        var containing = new List<SemanticSymbol>();
        for (var parent = context?.Owner; parent is not null; parent = containing[^1].Context?.Owner)
        {
            cancellationToken.ThrowIfCancellationRequested(); containing.Add(snapshot.Symbol(parent));
        }
        var related = new Dictionary<string, DefinitionResult>(StringComparer.Ordinal);
        foreach (var id in context?.SurfaceSymbols ?? []) Add(Definition(project, snapshot.Symbol(id)));
        foreach (var type in surface)
        {
            cancellationToken.ThrowIfCancellationRequested();
            if (type.Parameter is { } parameter) { Add(Definition(project, snapshot.Symbol(parameter))); continue; }
            if (type.Metadata is not { } metadata) continue;
            var candidate = snapshot.Symbols.FirstOrDefault(candidate => candidate.Metadata is { } identity && MetadataKey(identity) == MetadataKey(metadata));
            candidate ??= new(type.Id + ":definition", "type", type.Display, type.Display, null, type.Id, metadata, null);
            Add(Definition(project, candidate));
        }
        var ordered = related.Values.OrderBy(item => item.Project, StringComparer.Ordinal).ThenBy(item => item.Symbol.Declaration?.Path, StringComparer.Ordinal)
            .ThenBy(item => item.Symbol.Declaration?.Offset).ThenBy(item => item.Symbol.Metadata?.GetKey() ?? item.Symbol.Id, StringComparer.Ordinal).ToArray();
        var sourceCount = ordered.Count(item => item.Origin == "source");
        var omittedByLimit = Math.Max(0, sourceCount - limit);
        var remaining = 65_536; var omittedText = 0;
        var source = Source(definition);
        var returnedSources = 0; var details = new List<RelatedDeclaration>();
        foreach (var item in ordered)
        {
            cancellationToken.ThrowIfCancellationRequested();
            if (item.Origin == "source" && returnedSources++ >= limit) continue;
            details.Add(new(item, Source(item)));
        }
        var owners = containing.Select(owner => new DeclarationMatch(project, owner.Id, owner.Kind, owner.Name, owner.Context!.QualifiedName,
            owner.Signature, owner.Context.Accessibility, owner.Declaration!)).ToArray();
        var inspection = new InspectResult(definition, Array.AsReadOnly(owners), Array.AsReadOnly(types), source,
            Array.AsReadOnly(details.ToArray()), sourceCount, Math.Min(sourceCount, limit), new(omittedText, omittedByLimit));
        return selected with { Definition = null, Inspect = inspection, Truncated = omittedText > 0 || omittedByLimit > 0 };

        string Identity(DefinitionResult item) => item.Project + ":" + (item.Origin == "metadata" ? item.Symbol.Metadata!.GetKey() : item.Symbol.Id);
        void Add(DefinitionResult item)
        {
            if (item.Origin != "implicit" && Identity(item) != Identity(definition)) related.TryAdd(Identity(item), item);
        }
        DeclarationSource Source(DefinitionResult item)
        {
            cancellationToken.ThrowIfCancellationRequested();
            if (item.Symbol.Context is not { } location || item.Project is null)
                return new(item.Project, new("metadata_only", null, null), new("metadata_only", null, null), new("metadata_only", null, null));
            var document = projects[item.Project].Snapshot.Documents.Single(document => Paths.Equals(document.Path, location.Span.Path));
            var declaration = Slice(location.Span, "unavailable");
            var documentation = Slice(location.Documentation, "no_documentation");
            var valid = projects[item.Project].Snapshot.Coverage;
            return new(item.Project, declaration, new(location.Body is null ? "no_body" : valid.Available(location.Body) ? "location" : "unavailable", location.Body, null), documentation);

            SourceSlice Slice(SourceSpan? span, string missing)
            {
                if (span is null) return new(missing, null, null);
                if (!projects[item.Project].Snapshot.Coverage.Available(span)) return new("unavailable", span, null);
                cancellationToken.ThrowIfCancellationRequested();
                var text = document.Text.AsSpan(span.Offset, span.Length);
                var bytes = Encoding.UTF8.GetByteCount(text);
                if (bytes > remaining) { omittedText++; return new("source_budget", span, null); }
                remaining -= bytes; return new("available", span, text.ToString());
            }
        }
    }
}
