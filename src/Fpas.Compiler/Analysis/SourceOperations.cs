namespace Fpas.Compiler.Analysis;

public sealed record SourceBody(string Id, string Kind, SourceSpan Span, IReadOnlyList<SourceSpan> Regions,
    string? Symbol, string? Parent);
public sealed record OperationReceiver(SourceSpan Span, string? Symbol, string Type);

public sealed record SourceOperation(string Id, string Body, SourceSpan Span, SourceSpan? Anchor,
    string Kind, string Phase, string? Target, string? TargetBody, MetadataIdentity? Operation,
    string Dispatch, string? UnknownReason, CallableSignature? Signature,
    string? OwnerType, string? ResultType, OperationReceiver? Receiver);

public sealed record ExcludedOperation(string Body, string Kind, SourceSpan Span);
