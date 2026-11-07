using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Documentation;

public sealed record DocumentationSearch(string Name, string Match = "exact", string? Project = null,
    string? Assembly = null, int Limit = 20, int Offset = 0);
public sealed record DocumentationContent(string Availability, string Format, string? Path, string? Content);
public sealed record DocumentationEntry(string Project, string Origin, string Kind, string Name, string QualifiedName,
    string Signature, string SignatureFormat, string Assembly, string? AssemblyPath, string? DocumentationId,
    SourceSpan? Span, DocumentationContent Documentation)
{
    [System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
    public string? Identity { get; init; }
}
public sealed record DocumentationResult(string Status, string SnapshotId, IReadOnlyList<DocumentationEntry> Entries,
    int TotalCount, int Offset, int? NextOffset, bool Truncated, int OmittedDocumentationCount,
    IReadOnlyList<string> SearchedProjects, IReadOnlyList<string> SearchedAssemblies, Fpas.Workspaces.Analysis.WorkspaceSemanticCoverage Coverage);

internal sealed record DocumentationCandidate(string Project, string Origin, string QualifiedName,
    string SortKey, Func<DocumentationEntry> Read);
