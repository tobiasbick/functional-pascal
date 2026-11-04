using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Queries;

public sealed record OperationSummary(IReadOnlyDictionary<string, int> Indexed,
    IReadOnlyDictionary<string, int> Excluded, IReadOnlyDictionary<string, int> Unresolved);
public sealed record CallOperationPolicy(string Name, IReadOnlyList<string> Included, IReadOnlyList<string> Excluded);
public sealed record CallSiteResult(string Project, SourceBody Owner, SourceOperation Site,
    string Relationship, DefinitionResult? Target, IReadOnlyList<SemanticType> Types);
public sealed record CallSitePage(IReadOnlyList<CallSiteResult> Sites, int TotalCount, int ReturnedCount,
    int Offset, int? NextOffset, int InvocationCount, int CallableReferenceCount);
public sealed record CalleesResult(string SelectionMode, DefinitionResult? Target, MetadataIdentity? SelectedOperation,
    SourceBody? Body, string Availability, CallOperationPolicy OperationPolicy, OperationSummary? Summary, CallSitePage? Page);
public sealed record CallersResult(string SelectionMode, DefinitionResult? Target, MetadataIdentity? SelectedOperation,
    SourceBody? Body, CallOperationPolicy OperationPolicy, OperationSummary Summary, CallSitePage Page);
