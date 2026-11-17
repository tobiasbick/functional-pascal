using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Queries;

/// <summary>Counts indexed, excluded and unresolved operations by category.</summary>
/// <param name="Indexed">Indexed site counts by operation kind.</param>
/// <param name="Excluded">Excluded source protocol counts by kind.</param>
/// <param name="Unresolved">Counts by unresolved-target reason.</param>
public sealed record OperationSummary(IReadOnlyDictionary<string, int> Indexed,
    IReadOnlyDictionary<string, int> Excluded, IReadOnlyDictionary<string, int> Unresolved);
/// <summary>Names the source-operation contract used by a call index.</summary>
/// <param name="Name">Versioned operation-policy name.</param>
/// <param name="Included">Included explicit operation kinds.</param>
/// <param name="Excluded">Excluded generated or implicit protocols.</param>
public sealed record CallOperationPolicy(string Name, IReadOnlyList<string> Included, IReadOnlyList<string> Excluded);
/// <summary>Describes a checked invocation or callable-reference site.</summary>
/// <param name="Project">Owning project path.</param>
/// <param name="Owner">Innermost executable body owning the operation.</param>
/// <param name="Site">Original checked source operation.</param>
/// <param name="Relationship">invocation or callable_reference.</param>
/// <param name="Target">Resolved declaration, when known.</param>
/// <param name="Types">Types required by the site's checked signature and substitutions.</param>
public sealed record CallSiteResult(string Project, SourceBody Owner, SourceOperation Site,
    string Relationship, DefinitionResult? Target, IReadOnlyList<SemanticType> Types);
/// <summary>Contains a deterministic page and whole-result relationship counts.</summary>
/// <param name="Sites">Returned checked sites.</param>
/// <param name="TotalCount">Total matching sites before paging.</param>
/// <param name="ReturnedCount">Number of returned sites.</param>
/// <param name="Offset">Requested sorted offset.</param>
/// <param name="NextOffset">Next page offset, or null at the end.</param>
/// <param name="InvocationCount">Total matching invocation sites.</param>
/// <param name="CallableReferenceCount">Total matching callable-reference sites.</param>
public sealed record CallSitePage(IReadOnlyList<CallSiteResult> Sites, int TotalCount, int ReturnedCount,
    int Offset, int? NextOffset, int InvocationCount, int CallableReferenceCount);
/// <summary>Describes outgoing source operations or explicit body unavailability.</summary>
/// <param name="SelectionMode">target or body selection.</param>
/// <param name="Target">Selected declaration, when present.</param>
/// <param name="SelectedOperation">Exact constructor or accessor identity, when selected.</param>
/// <param name="Body">Selected executable body, when present.</param>
/// <param name="Availability">available, metadata_only or no_body.</param>
/// <param name="OperationPolicy">Included and excluded source-operation contract.</param>
/// <param name="Summary">Whole-body summary, or null without an available body.</param>
/// <param name="Page">Site page, or null without an available body.</param>
public sealed record CalleesResult(string SelectionMode, DefinitionResult? Target, MetadataIdentity? SelectedOperation,
    SourceBody? Body, string Availability, CallOperationPolicy OperationPolicy, OperationSummary? Summary, CallSitePage? Page);
/// <summary>Describes incoming sites across the checked FPAS context.</summary>
/// <param name="SelectionMode">target or body selection.</param>
/// <param name="Target">Selected declaration, when present.</param>
/// <param name="SelectedOperation">Exact constructor or accessor identity, when selected.</param>
/// <param name="Body">Selected callable body, when present.</param>
/// <param name="OperationPolicy">Included and excluded source-operation contract.</param>
/// <param name="Summary">Whole-context indexed/excluded/unresolved counts.</param>
/// <param name="Page">Matching incoming-site page.</param>
public sealed record CallersResult(string SelectionMode, DefinitionResult? Target, MetadataIdentity? SelectedOperation,
    SourceBody? Body, CallOperationPolicy OperationPolicy, OperationSummary Summary, CallSitePage Page);
