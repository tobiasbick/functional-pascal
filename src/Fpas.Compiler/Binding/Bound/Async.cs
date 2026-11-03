using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.References;
namespace Fpas.Compiler.Binding.Bound;

internal sealed record BoundAwait(BoundExpression Operand, ImportedMethod GetAwaiter, ImportedMethod IsCompleted, ImportedMethod GetResult, SourceLocation Location, bool IsImplicit = false) : BoundExpression(GetResult.Result);
internal sealed record BoundNoValue() : BoundExpression(TypeSymbol.Void);
internal sealed record BoundPendingException(TypeSymbol ExceptionType) : BoundExpression(ExceptionType);
