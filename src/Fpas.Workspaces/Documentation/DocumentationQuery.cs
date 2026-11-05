using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;
using System.Text;

namespace Fpas.Workspaces.Documentation;

public sealed class DocumentationQuery
{
    private readonly WorkspaceAnalysis analysis;
    private readonly SourceQuery source;
    private DocumentationInputs? inputs;
    private static StringComparer Paths => OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
    public DocumentationQuery(WorkspaceAnalysis analysis)
    { this.analysis = analysis; source = new(analysis); }

    public static void Validate(DocumentationSearch search)
    {
        if (string.IsNullOrWhiteSpace(search.Name)) throw new ContextFailure("invalid_argument", "--name must not be empty.");
        if (search.Match is not ("exact" or "prefix" or "contains")) throw new ContextFailure("invalid_argument", "--match must be exact, prefix or contains.");
        if (search.Limit is < 1 or > 100 || search.Offset < 0) throw new ContextFailure("invalid_argument", "--limit must be 1..100 and --offset must be nonnegative.");
        if (search.Assembly is not null && string.IsNullOrWhiteSpace(search.Assembly)) throw new ContextFailure("invalid_argument", "--assembly must be an assembly's simple name.");
    }
    public DocumentationResult Search(DocumentationSearch search, CancellationToken token = default)
    {
        Validate(search); token.ThrowIfCancellationRequested();
        var projects = analysis.Context.Projects.Where(project => search.Project is null || Paths.Equals(project.Path, Path.GetFullPath(search.Project)))
            .OrderBy(project => project.Path, StringComparer.Ordinal).ToArray();
        if (projects.Length == 0) throw new ContextFailure("invalid_argument", "--project must name an analyzed FPAS project in the context.");
        var references = projects.SelectMany(project => project.Request.References).Distinct(Paths).Order(StringComparer.Ordinal).ToArray();
        inputs = new(analysis.Context.SnapshotId, references, token);
        var xml = new XmlDocumentation(inputs, token);
        using var metadata = new MetadataCatalog();
        var candidates = source.Documentation(search, token).ToList();
        var sourceProjects = projects.Select(project => project.Path).ToHashSet(Paths);
        var sourceOutputs = analysis.Context.DependencyOutputs.Where(output => sourceProjects.Contains(output.Project)).Select(output => output.Path).ToHashSet(Paths);
        foreach (var project in projects)
            foreach (var path in project.Request.References.Distinct(Paths).Where(path => !sourceOutputs.Contains(path)))
            {
                token.ThrowIfCancellationRequested(); candidates.AddRange(metadata.Search(project.Path, path, search, token));
            }
        var ordered = candidates.OrderBy(item => item.Project, StringComparer.Ordinal).ThenBy(item => item.Origin == "source" ? 0 : 1)
            .ThenBy(item => item.QualifiedName, StringComparer.Ordinal).ThenBy(item => item.SortKey, StringComparer.Ordinal).ToArray();
        var page = new List<DocumentationEntry>(); var remaining = 65_536; var omitted = 0;
        foreach (var candidate in ordered.Skip(search.Offset).Take(search.Limit))
        {
            token.ThrowIfCancellationRequested(); var entry = candidate.Read();
            var docs = entry.Origin == "metadata" ? xml.Read(entry) : entry.Documentation;
            if (docs.Content is { } content)
            {
                var size = Encoding.UTF8.GetByteCount(content);
                if (size > remaining) { docs = docs with { Availability = "text_budget", Content = null }; omitted++; }
                else remaining -= size;
            }
            else if (docs.Availability == "source_budget") omitted++;
            page.Add(entry with { Documentation = docs });
        }
        var next = (long)search.Offset + page.Count;
        return new(ordered.Length == 0 ? "not_found" : "ok", inputs.SnapshotId, page.AsReadOnly(), ordered.Length, search.Offset,
            next < ordered.Length ? (int)next : null, page.Count < ordered.Length || omitted != 0, omitted,
            Array.AsReadOnly(projects.Select(project => project.Path).ToArray()), Array.AsReadOnly(references));
    }
    public void VerifyInputs(CancellationToken token = default)
    {
        if (inputs is null) throw new InvalidOperationException("Search must capture documentation inputs before verification.");
        inputs.Verify(token);
    }
}
