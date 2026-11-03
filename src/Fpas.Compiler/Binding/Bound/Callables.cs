using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Types;

namespace Fpas.Compiler.Binding.Bound;

internal sealed record BoundCapture(CaptureSymbol Capture, TypeSymbol? FlowType = null) : BoundExpression(FlowType ?? Capture.Type);
internal sealed record BoundCaptureStorage(CaptureSymbol Capture, TypeSymbol StorageType) : BoundExpression(StorageType);
internal sealed record BoundLocalStorage(LocalSymbol Local, TypeSymbol StorageType) : BoundExpression(StorageType);
internal sealed record BoundClosureSelf(ClosureSymbol Closure) : BoundExpression(Closure.Type);
internal sealed record BoundEnvironment(ClosureSymbol Closure, IReadOnlyList<BoundExpression> Arguments) : BoundExpression(Closure.Type);
internal abstract record BoundDelegate(TypeSymbol DelegateType) : BoundExpression(DelegateType);
internal sealed record BoundLambda(ClosureSymbol Closure, TypeSymbol DelegateType) : BoundDelegate(DelegateType);
internal sealed record BoundMethodDelegate(ImportedMethod Method, BoundExpression? Receiver, TypeSymbol DelegateType) : BoundDelegate(DelegateType);
internal sealed record BoundMethodGroup(IReadOnlyList<ImportedMethod> Methods, BoundExpression? Receiver) : BoundDelegate(TypeSymbol.Unsupported);
