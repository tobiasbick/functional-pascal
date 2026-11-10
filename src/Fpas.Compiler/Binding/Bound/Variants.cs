using Fpas.Compiler.Binding.Types;

namespace Fpas.Compiler.Binding.Bound;

internal sealed record BoundTypeTest(BoundExpression Value, TypeSymbol Target) : BoundExpression(TypeSymbol.Boolean);
internal sealed record BoundFailure(TypeSymbol Result, string Message) : BoundExpression(Result);
