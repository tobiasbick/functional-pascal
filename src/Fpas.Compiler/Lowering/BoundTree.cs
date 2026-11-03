using Fpas.Compiler.Binding;

namespace Fpas.Compiler.Lowering;

internal static class BoundTree
{
    internal static bool HasAwait(BoundExpression expression) => expression is BoundAwait || Children(expression).Any(HasAwait);

    internal static IEnumerable<BoundExpression> Children(BoundExpression expression) => expression switch
    {
        BoundAwait awaited => [awaited.Operand],
        BoundCall call => (call.Receiver is null ? [] : new[] { call.Receiver }).Concat(call.Arguments),
        BoundNew creation => creation.Arguments,
        BoundSequence sequence => sequence.Bindings.Select(binding => binding.Initializer).Append(sequence.Value),
        BoundBinary binary => [binary.Left, binary.Right],
        BoundConditional conditional => conditional.Arms.SelectMany(arm => new[] { arm.Condition, arm.Value }).Append(conditional.Else),
        BoundCoalesce coalesce => [coalesce.Left, coalesce.Right],
        BoundUnary unary => [unary.Operand],
        BoundConversion conversion => [conversion.Operand],
        BoundNullTest test => [test.Operand],
        BoundTypeTest test => [test.Value],
        BoundSafeCast cast => [cast.Value],
        BoundArrayLength length => [length.Receiver],
        BoundArrayIndex index => new[] { index.Receiver }.Concat(index.Indices),
        BoundArrayLiteral array => array.Elements,
        BoundNewArray array => array.Lengths,
        BoundProperty property => (property.Receiver is null ? [] : new[] { property.Receiver }).Concat(property.Arguments ?? []),
        BoundField { Receiver: not null } field => [field.Receiver],
        BoundRecordField field => [field.Receiver],
        BoundAddress address => [address.Target],
        BoundDereference dereference => [dereference.Address],
        BoundMethodDelegate { Receiver: not null } method => [method.Receiver],
        BoundLambda lambda => lambda.Closure.Captures.Select(capture => capture.Value),
        BoundEnvironment environment => environment.Arguments,
        _ => []
    };
}
