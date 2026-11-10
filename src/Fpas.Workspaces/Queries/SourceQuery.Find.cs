namespace Fpas.Workspaces.Queries;

public sealed record NameSearch(string Name, string Match = "contains", IReadOnlyList<string>? Kinds = null,
    string? Project = null, int Limit = 100, int Offset = 0);
public sealed record DeclarationMatch(string Project, string Symbol, string Kind, string Name, string QualifiedName,
    string Signature, string? Accessibility, Fpas.Compiler.Analysis.SourceSpan Span);
public sealed record FindResult(IReadOnlyList<DeclarationMatch> Matches, int TotalCount, int ReturnedCount, int Offset, int? NextOffset);

public sealed partial class SourceQuery
{
    public static IReadOnlyList<string> DeclarationKinds { get; } = Array.AsReadOnly(new[]
    {
        "container", "record", "interface", "variant", "domain", "alias", "routine", "property", "global",
        "variant_case", "enum_member", "parameter", "type_parameter", "local"
    });

    public static void Validate(NameSearch search)
    {
        if (search.Name is null) throw new Loading.ContextFailure("invalid_argument", "Supply a name search.");
        if (search.Match is not ("exact" or "prefix" or "contains")) throw new Loading.ContextFailure("invalid_argument", "--match must be exact, prefix or contains.");
        if (search.Limit is < 1 or > 1000 || search.Offset < 0) throw new Loading.ContextFailure("invalid_argument", "--limit must be 1..1000 and --offset must be nonnegative.");
        if (search.Kinds?.Any(kind => !DeclarationKinds.Contains(kind, StringComparer.Ordinal)) == true)
            throw new Loading.ContextFailure("invalid_argument", "Unknown --kind; expected " + string.Join(", ", DeclarationKinds) + ".");
    }

    public SourceQueryResult Find(NameSearch search, CancellationToken cancellationToken = default)
        => FindCore(search, cancellationToken) with { Coverage = coverage };

    private SourceQueryResult FindCore(NameSearch search, CancellationToken cancellationToken)
    {
        Validate(search);
        var searched = projects.Keys.Where(project => search.Project is null || Paths.Equals(project, Path.GetFullPath(search.Project)))
            .Order(StringComparer.Ordinal).ToArray();
        if (searched.Length == 0) throw new Loading.ContextFailure(search.Project is not null && analysis.Projects.Any(project => Paths.Equals(project.Project, Path.GetFullPath(search.Project))) ? "analysis_unavailable" : "invalid_argument",
            "Select a project with checked semantic regions in the context.");
        var matches = new List<DeclarationMatch>();
        foreach (var project in searched)
            foreach (var symbol in projects[project].Snapshot.Symbols)
            {
                cancellationToken.ThrowIfCancellationRequested();
                if (symbol.Context is not { } context || symbol.Declaration is not { } span) continue;
                if (search.Kinds is { Count: > 0 } && !search.Kinds.Contains(symbol.Kind, StringComparer.Ordinal)) continue;
                if (!Matches(symbol.Name) && !Matches(context.QualifiedName)) continue;
                matches.Add(new(project, symbol.Id, symbol.Kind, symbol.Name, context.QualifiedName, symbol.Signature, context.Accessibility, span));
            }
        var ordered = matches.OrderBy(match => match.Project, StringComparer.Ordinal).ThenBy(match => match.Span.Path, StringComparer.Ordinal)
            .ThenBy(match => match.Span.Offset).ThenBy(match => match.Kind, StringComparer.Ordinal).ThenBy(match => match.Signature, StringComparer.Ordinal).ToArray();
        var page = ordered.Skip(search.Offset).Take(search.Limit).ToArray();
        var next = (long)search.Offset + page.Length;
        return new("ok", null, null, null, null, null, page.Length < ordered.Length, BinaryBoundaries(),
            new(Array.AsReadOnly(page), ordered.Length, page.Length, search.Offset, next < ordered.Length ? (int)next : null), Searched: Array.AsReadOnly(searched));

        bool Matches(string name) => search.Match switch
        {
            "exact" => name.Equals(search.Name, StringComparison.Ordinal),
            "prefix" => name.StartsWith(search.Name, StringComparison.Ordinal),
            _ => name.Contains(search.Name, StringComparison.Ordinal)
        };
    }

    private string[] BinaryBoundaries() => analysis.Context.Projects.SelectMany(project => project.Request.References).Distinct(Paths)
        .Where(path => !analysis.Context.DependencyOutputs.Any(output => Paths.Equals(output.Path, path) && projects.ContainsKey(output.Project)))
        .Order(StringComparer.Ordinal).ToArray();
}
