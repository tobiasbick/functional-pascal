namespace Fpas.Compiler.Binding;

internal sealed class ClosureSymbol(int id, RecordSymbol? recordContext = null, IReadOnlyList<GenericParameterSymbol>? parameters = null)
{
    internal IReadOnlyList<GenericParameterSymbol> GenericParameters { get; } = parameters ?? [];
    internal int Id { get; } = id;
    internal RecordSymbol? RecordContext { get; } = recordContext;
    internal List<CaptureSymbol> Captures { get; } = [];
    internal BoundRoutine Body { get; set; } = null!;
    internal List<BoundRoutine> NestedBodies { get; } = [];
    internal IEnumerable<BoundRoutine> Bodies => NestedBodies.Count == 0 ? [Body] : NestedBodies;
    internal TypeSymbol Type => new(TypeKind.Generated, Closure: this);
}

internal sealed class CaptureSymbol(object original, TypeSymbol type, BoundExpression value, bool mutable, ClosureSymbol owner)
{
    internal ClosureSymbol Owner { get; } = owner;
    internal object Original { get; } = original;
    internal TypeSymbol Type { get; } = type;
    internal BoundExpression Value { get; } = value;
    internal bool Mutable { get; } = mutable;
}

internal sealed record BoundCapture(CaptureSymbol Capture, TypeSymbol? FlowType = null) : BoundExpression(FlowType ?? Capture.Type);
internal sealed record BoundCaptureStorage(CaptureSymbol Capture, TypeSymbol StorageType) : BoundExpression(StorageType);
internal sealed record BoundLocalStorage(LocalSymbol Local, TypeSymbol StorageType) : BoundExpression(StorageType);
internal sealed record BoundClosureSelf(ClosureSymbol Closure) : BoundExpression(Closure.Type);
internal sealed record BoundEnvironment(ClosureSymbol Closure, IReadOnlyList<BoundExpression> Arguments) : BoundExpression(Closure.Type);
internal abstract record BoundDelegate(TypeSymbol DelegateType) : BoundExpression(DelegateType);
internal sealed record BoundLambda(ClosureSymbol Closure, TypeSymbol DelegateType) : BoundDelegate(DelegateType);
internal sealed record BoundMethodDelegate(ImportedMethod Method, BoundExpression? Receiver, TypeSymbol DelegateType) : BoundDelegate(DelegateType);
internal sealed record BoundMethodGroup(IReadOnlyList<ImportedMethod> Methods, BoundExpression? Receiver) : BoundDelegate(TypeSymbol.Unsupported);
