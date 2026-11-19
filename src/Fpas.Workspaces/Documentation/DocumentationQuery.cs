// [Documentation](../../../docs/pascal/tools/api-documentation.md)
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;
using System.Text;

namespace Fpas.Workspaces.Documentation;

/// <summary>Searches checked FPAS declarations and public metadata without loading reference code.</summary>
public sealed class DocumentationQuery
{
    private readonly WorkspaceAnalysis analysis;
    private readonly SourceQuery source;
    private DocumentationInputs? inputs;
    private static StringComparer Paths => OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
    /// <summary>Creates documentation lookup over current captured analysis.</summary>
    /// <param name="analysis">Analysis with at least one available checked source region.</param>
    public DocumentationQuery(WorkspaceAnalysis analysis)
    { this.analysis = analysis; source = new(analysis); }

    /// <summary>Rejects empty names, unsupported match modes and invalid page bounds.</summary>
    /// <param name="search">Documentation filter to validate.</param>
    public static void Validate(DocumentationSearch search)
    {
        if (search.TypeArguments is { Count: > 0 } arguments && (!search.Usage || arguments.Any(string.IsNullOrWhiteSpace)))
            throw new ContextFailure("invalid_argument", "--type-arg requires --usage and a nonempty FPAS type.");
        if (search.Usage && (search.Match != "exact" || search.Name is not { Length: > 2 } || search.Name[1] != ':' || search.Name[0] is not ('M' or 'P' or 'F')))
            throw new ContextFailure("invalid_argument", "--usage requires an exact M:, P: or F: documentation ID from --name lookup.");
        if (search.Access is not null && (search.Access is not ("read" or "write") || !search.Usage || search.Name[0] == 'M'))
            throw new ContextFailure("invalid_argument", "--access read or write requires a property or field ID with --usage.");
        if (string.IsNullOrWhiteSpace(search.Name)) throw new ContextFailure("invalid_argument", "--name must not be empty.");
        if (search.Match is not ("exact" or "prefix" or "contains")) throw new ContextFailure("invalid_argument", "--match must be exact, prefix or contains.");
        if (search.Limit is < 1 or > 100 || search.Offset < 0) throw new ContextFailure("invalid_argument", "--limit must be 1..100 and --offset must be nonnegative.");
        if (search.Assembly is not null && string.IsNullOrWhiteSpace(search.Assembly)) throw new ContextFailure("invalid_argument", "--assembly must be an assembly's simple name.");
    }
    /// <summary>Captures XML identities and searches actual source/reference declarations.</summary>
    /// <param name="search">Literal name, owner filters and pagination.</param>
    /// <param name="token">Cancels traversal, metadata reads and documentation hashing.</param>
    /// <param name="metadataOnly">Whether to search captured reference images only, bounded to 1,000 matches.</param>
    /// <returns>A deterministic page sharing a 64-KiB documentation text budget.</returns>
    public DocumentationResult Search(DocumentationSearch search, CancellationToken token = default, bool metadataOnly = false)
    {
        Validate(search); token.ThrowIfCancellationRequested();
        var projects = analysis.Context.Projects.Where(project => search.Project is null || Paths.Equals(project.Path, Path.GetFullPath(search.Project)))
            .OrderBy(project => project.Path, StringComparer.Ordinal).ToArray();
        if (projects.Length == 0) throw new ContextFailure("invalid_argument", "--project must name an analyzed FPAS project in the context.");
        var references = projects.SelectMany(project => project.Request.References).Distinct(Paths).Order(StringComparer.Ordinal).ToArray();
        inputs = new(analysis.Context.SnapshotId, references, token);
        var xml = new XmlDocumentation(inputs, token);
        using var metadata = new MetadataCatalog();
        var candidates = metadataOnly || search.Usage ? [] : source.Documentation(search, token).ToList();
        var sourceProjects = projects.Select(project => project.Path).ToHashSet(Paths);
        var sourceOutputs = analysis.Context.DependencyOutputs.Where(output => sourceProjects.Contains(output.Project)).Select(output => output.Path).ToHashSet(Paths);
        foreach (var project in projects)
            foreach (var path in project.Request.References.Distinct(Paths).Where(path => !sourceOutputs.Contains(path)))
            {
                token.ThrowIfCancellationRequested();
                var image = metadataOnly || search.Usage ? project.Request.ReferenceImages?.FirstOrDefault(item => Paths.Equals(item.Path, path)) : null;
                if (metadataOnly && image is null) continue;
                foreach (var candidate in metadata.Search(project.Path, path, search, token, image, search.Usage ? project.Request : null))
                {
                    if (metadataOnly && candidates.Count == 1000)
                        throw new ContextFailure("documentation_limit", "Metadata search exceeds 1,000 matches. Narrow the name or project.");
                    candidates.Add(candidate);
                }
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
        return new(ordered.Length == 0 && analysis.IsComplete ? "not_found" : "ok", inputs.SnapshotId, page.AsReadOnly(), ordered.Length, search.Offset,
            next < ordered.Length ? (int)next : null, page.Count < ordered.Length || omitted != 0, omitted,
            Array.AsReadOnly(projects.Select(project => project.Path).ToArray()), Array.AsReadOnly(references), analysis.SemanticCoverage);
    }
    /// <summary>Rechecks XML contents and absence captured by the latest search.</summary>
    /// <param name="token">Cancels documentation hashing.</param>
    public void VerifyInputs(CancellationToken token = default)
    {
        if (inputs is null) throw new InvalidOperationException("Search must capture documentation inputs before verification.");
        inputs.Verify(token);
    }
}
