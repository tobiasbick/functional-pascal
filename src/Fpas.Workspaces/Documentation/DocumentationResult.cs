// [Documentation](../../../docs/pascal/tools/api-documentation.md)
using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Documentation;

/// <summary>Filters actual source and reference-metadata declarations for documentation lookup.</summary>
/// <param name="Name">Case-sensitive simple or qualified name.</param>
/// <param name="Match">exact, prefix or contains.</param>
/// <param name="Project">Optional analyzed owning project path.</param>
/// <param name="Assembly">Optional exact assembly simple name.</param>
/// <param name="Limit">Page size, 1..100.</param>
/// <param name="Offset">Nonnegative sorted-entry offset.</param>
/// <param name="Usage">Select an exact method, property or field XML ID in Name and generate a checked usage unit.</param>
/// <param name="TypeArguments">Concrete FPAS types for the declaring type and then the selected method, in declaration order.</param>
/// <param name="Access">Optional read or write selection for a property, indexer or field. The default is read.</param>
public sealed record DocumentationSearch(string Name, string Match = "exact", string? Project = null,
    string? Assembly = null, int Limit = 20, int Offset = 0, bool Usage = false, IReadOnlyList<string>? TypeArguments = null, string? Access = null);
/// <summary>Contains documentation text or an explicit absence/omission state.</summary>
/// <param name="Availability">Availability or missing/invalid-budget reason.</param>
/// <param name="Format">markdown or xml.</param>
/// <param name="Path">Owning source or adjacent XML path, when known.</param>
/// <param name="Content">Native documentation markup, or null when unavailable.</param>
public sealed record DocumentationContent(string Availability, string Format, string? Path, string? Content);
/// <summary>Describes one actual source or public CLR declaration and its documentation.</summary>
/// <param name="Project">Owning project path, or null for standalone references.</param>
/// <param name="Origin">source or metadata.</param>
/// <param name="Kind">Declaration kind.</param>
/// <param name="Name">Simple declaration name.</param>
/// <param name="QualifiedName">Qualified declaration name.</param>
/// <param name="Signature">Actual declaration signature.</param>
/// <param name="SignatureFormat">fpas or clr; CLR signatures do not certify FPAS call support.</param>
/// <param name="Assembly">Owning assembly's simple name.</param>
/// <param name="AssemblyPath">Metadata assembly path, or null for source.</param>
/// <param name="DocumentationId">Exact XML member ID for metadata, or null for source.</param>
/// <param name="Span">Source navigation span, when available.</param>
/// <param name="Documentation">Native documentation text or explicit absence state.</param>
public sealed record DocumentationEntry(string? Project, string Origin, string Kind, string Name, string QualifiedName,
    string Signature, string SignatureFormat, string Assembly, string? AssemblyPath, string? DocumentationId,
    SourceSpan? Span, DocumentationContent Documentation)
{
    /// <summary>Generated usage or an explanation for the selected method, when requested.</summary>
    public ApiUsage? Usage { get; init; }
    /// <summary>Gets captured metadata image/token identity, when lookup used a private image.</summary>
    [System.Text.Json.Serialization.JsonIgnore(Condition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull)]
    public string? Identity { get; init; }
}
/// <summary>Contains a documentation page with an identity including adjacent XML contents or absence.</summary>
/// <param name="Status">ok, or not_found when complete analysis proves absence.</param>
/// <param name="SnapshotId">Context and documentation-input identity for safe pagination.</param>
/// <param name="Entries">Returned declaration entries.</param>
/// <param name="TotalCount">Total checked matches before paging.</param>
/// <param name="Offset">Requested sorted offset.</param>
/// <param name="NextOffset">Next page offset, or null at the end.</param>
/// <param name="Truncated">Whether paging or text budgets omitted results.</param>
/// <param name="OmittedDocumentationCount">Blocks omitted by text budgets.</param>
/// <param name="SearchedProjects">Searched FPAS compilation paths.</param>
/// <param name="SearchedAssemblies">Actual reference assembly paths in the search scope.</param>
/// <param name="Coverage">Semantic coverage independent of paging and text budgets.</param>
public sealed record DocumentationResult(string Status, string SnapshotId, IReadOnlyList<DocumentationEntry> Entries,
    int TotalCount, int Offset, int? NextOffset, bool Truncated, int OmittedDocumentationCount,
    IReadOnlyList<string> SearchedProjects, IReadOnlyList<string> SearchedAssemblies, Fpas.Workspaces.Analysis.WorkspaceSemanticCoverage Coverage);

internal sealed record DocumentationCandidate(string? Project, string Origin, string QualifiedName,
    string SortKey, Func<DocumentationEntry> Read);
