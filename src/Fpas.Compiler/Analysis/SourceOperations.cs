// [Documentation](../../../docs/architecture/compiler.md#semantic-snapshots)
namespace Fpas.Compiler.Analysis;

/// <summary>A routine, lambda, or initializer body that contains operations.</summary>
/// <param name="Id">The body id.</param>
/// <param name="Kind">The body category, such as routine, program_entry, or an initializer kind.</param>
/// <param name="Span">The whole body range.</param>
/// <param name="Regions">The source ranges evaluated by this body: the body itself, then any contract conditions.</param>
/// <param name="Symbol">The symbol id of the owning routine, when named.</param>
/// <param name="Parent">The id of the enclosing body, when nested.</param>
public sealed record SourceBody(string Id, string Kind, SourceSpan Span, IReadOnlyList<SourceSpan> Regions,
    string? Symbol, string? Parent);

/// <summary>The receiver of a delegate invocation.</summary>
/// <param name="Span">The receiver expression.</param>
/// <param name="Symbol">The symbol id the receiver names, when it is a name.</param>
/// <param name="Type">The type id of the receiver.</param>
public sealed record OperationReceiver(SourceSpan Span, string? Symbol, string Type);

/// <summary>A checked call, construction, accessor, or operator use inside a body.</summary>
/// <param name="Id">The operation id.</param>
/// <param name="Body">The id of the containing body.</param>
/// <param name="Span">The whole operation expression.</param>
/// <param name="Anchor">The name or operator token that identifies the operation, when one.</param>
/// <param name="Kind">The operation category, such as invoke, construct, property_get, or delegate_invoke.</param>
/// <param name="Phase">The evaluation phase in the body, such as body or a contract clause.</param>
/// <param name="Target">The symbol id of the called member, when known.</param>
/// <param name="TargetBody">The body id of a called source lambda or routine, when known.</param>
/// <param name="Operation">The CLR identity of the called member, when available.</param>
/// <param name="Dispatch">How the target is selected, such as direct, virtual, constrained, or delegate.</param>
/// <param name="UnknownReason">Why the runtime target is not statically known, when it is not.</param>
/// <param name="Signature">The checked signature of the target.</param>
/// <param name="OwnerType">The type id of the member owner.</param>
/// <param name="ResultType">The type id of the operation result.</param>
/// <param name="Receiver">The receiver of a delegate invocation.</param>
public sealed record SourceOperation(string Id, string Body, SourceSpan Span, SourceSpan? Anchor,
    string Kind, string Phase, string? Target, string? TargetBody, MetadataIdentity? Operation,
    string Dispatch, string? UnknownReason, CallableSignature? Signature,
    string? OwnerType, string? ResultType, OperationReceiver? Receiver)
{
    internal IReadOnlyList<(int Offset, string Label)> Labels { get; init; } = [];

    /// <summary>Gets proposed parameter labels for positional arguments of this call. The resulting overload still needs verification.</summary>
    /// <returns>UTF-16 insertion offsets and escaped parameter names in written order.</returns>
    public IReadOnlyList<(int Offset, string Label)> SuggestedArgumentLabels() => Labels;

    /// <summary>Compares dispatch and the selected CLR member signature and definition token across analyses of the same references.</summary>
    /// <param name="other">The operation checked after a source edit.</param>
    /// <returns>True for matching dispatch and metadata. Source declaration bindings and generic substitutions require separate checks.</returns>
    public bool HasSameDispatch(SourceOperation other) => Kind == other.Kind && Dispatch == other.Dispatch &&
        Operation?.DefinitionToken == other.Operation?.DefinitionToken &&
        Operation?.GetKey(includeAssemblyHash: false) == other.Operation?.GetKey(includeAssemblyHash: false);

    internal bool[]? InertArguments { get; init; }

    /// <summary>Checks whether omitting this evaluated argument preserves side effects and exceptions.</summary>
    /// <param name="parameterIndex">Index in the selected signature, including defaulted slots.</param>
    /// <returns>True only for a proven inert bound value, including its conversion.</returns>
    public bool CanOmitArgument(int parameterIndex) => InertArguments is { } slots &&
        (uint)parameterIndex < (uint)slots.Length && slots[parameterIndex];
}

/// <summary>An operation the snapshot deliberately does not model, such as an implicit dispose.</summary>
/// <param name="Body">The id of the containing body.</param>
/// <param name="Kind">The excluded protocol, such as using_protocol.</param>
/// <param name="Span">The source range of the excluded construct.</param>
public sealed record ExcludedOperation(string Body, string Kind, SourceSpan Span);
