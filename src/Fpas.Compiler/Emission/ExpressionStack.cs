using Fpas.Compiler.Binding;

namespace Fpas.Compiler.Emission;

internal static class ExpressionStack
{
    internal static int Depth(BoundExpression expression) => expression switch
    {
        BoundAddress address => Depth(address.Target) + 1,
        BoundDereference dereference => Depth(dereference.Address),
        BoundTypeTest test => Math.Max(2, Depth(test.Value)),
        BoundSequence sequence => sequence.Bindings.Select(binding => Depth(binding.Initializer)).Append(Depth(sequence.Value)).Max(),
        BoundRecordField field => Depth(field.Receiver),
        BoundSafeCast cast => Depth(cast.Value),
        BoundLambda lambda => lambda.Closure.Captures.Select((capture, index) => index + Depth(capture.Value)).DefaultIfEmpty(2).Max() + 2,
        BoundMethodDelegate method => method.Receiver is null ? 2 : Depth(method.Receiver) + 2,
        BoundCall call => Math.Max(call.Receiver is null ? 0 : Depth(call.Receiver), Arguments(call.Arguments) + (call.Receiver is null ? 0 : 1)),
        BoundNewArray array => Arguments(array.Lengths),
        BoundArrayLiteral array => array.Elements.Select(element => 3 + Depth(element)).DefaultIfEmpty(1).Max(),
        BoundArrayIndex index => Math.Max(Depth(index.Receiver), 1 + Arguments(index.Indices)),
        BoundArrayLength length => Depth(length.Receiver),
        BoundNew creation => Arguments(creation.Arguments),
        BoundField field => field.Receiver is null ? 1 : Depth(field.Receiver),
        BoundProperty property => Math.Max(property.Receiver is null ? 0 : Depth(property.Receiver), Arguments(property.Arguments ?? []) + (property.Receiver is null ? 0 : 1)),
        BoundNullTest test => Depth(test.Operand),
        BoundCoalesce coalesce => Math.Max(Depth(coalesce.Left), Depth(coalesce.Right)),
        BoundConversion conversion => Depth(conversion.Operand),
        BoundUnary unary => Math.Max(2, Depth(unary.Operand) +
            (unary.Operator == "-" && unary.Checked && unary.Type == TypeSymbol.Int32 ? 1 : 0)),
        BoundBinary binary => Math.Max(Depth(binary.Left),
            Depth(binary.Right) + (binary.Left.Type == TypeSymbol.Boolean && binary.Operator is "and" or "or" ? 0 : 1)),
        BoundConditional conditional => conditional.Arms.Select(arm => Math.Max(Depth(arm.Condition), Depth(arm.Value)))
            .Append(Depth(conditional.Else)).Max(),
        _ => 1
    };

    private static int Arguments(IReadOnlyList<BoundExpression> arguments)
        => arguments.Select((argument, index) => index + Depth(argument)).DefaultIfEmpty(1).Max();
}
