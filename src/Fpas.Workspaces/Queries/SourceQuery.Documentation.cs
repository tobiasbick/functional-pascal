using Fpas.Workspaces.Documentation;

namespace Fpas.Workspaces.Queries;

public sealed partial class SourceQuery
{
    internal IEnumerable<DocumentationCandidate> Documentation(DocumentationSearch search, CancellationToken token)
    {
        var kinds = DeclarationKinds.Where(kind => kind is not ("local" or "parameter" or "type_parameter")).ToArray();
        var matches = new Dictionary<(string Project, string Symbol), DeclarationMatch>();
        void Collect(NameSearch input, Func<DeclarationMatch, bool>? include = null)
        {
            for (int offset = 0; ;)
            {
                token.ThrowIfCancellationRequested();
                var found = Find(input with { Offset = offset, Limit = 1000 }, token).Find!;
                foreach (var match in found.Matches)
                    if (include is null || include(match)) matches.TryAdd((match.Project, match.Symbol), match);
                if (found.NextOffset is not { } next) break;
                offset = next;
            }
        }
        Collect(new(search.Name, search.Match, kinds, search.Project));
        if (search.Match == "exact")
            foreach (var owner in matches.Values.Where(item => item.Kind is "container" or "record" or "interface" or "variant" or "domain").ToArray())
                Collect(new(owner.QualifiedName + ".", "prefix", kinds, owner.Project), member => projects[member.Project].Snapshot.Symbol(member.Symbol).Context?.Owner == owner.Symbol);
        foreach (var match in matches.Values)
        {
            var snapshot = projects[match.Project].Snapshot;
            if (search.Assembly is not null && search.Assembly != snapshot.AssemblyName) continue;
            yield return new(match.Project, "source", match.QualifiedName,
                match.Span.Path + ":" + match.Span.Offset.ToString("D10", System.Globalization.CultureInfo.InvariantCulture), () =>
                {
                    token.ThrowIfCancellationRequested();
                    var inspection = Inspect(new(match.Span.Path, match.Span.Line, match.Span.Column, match.Project), 1, token).Inspect!;
                    var docs = inspection.Source.Documentation;
                    var content = docs.Text is null ? null : DocumentationMarkdown.Source(docs.Text);
                    return new(match.Project, "source", match.Kind, match.Name, match.QualifiedName, match.Signature, "fpas", snapshot.AssemblyName,
                        null, null, match.Span, new(docs.Availability, "markdown", docs.Span?.Path, content));
                });
        }
    }
}
