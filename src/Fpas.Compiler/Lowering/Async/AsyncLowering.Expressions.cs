using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Bound;

namespace Fpas.Compiler.Lowering.Async;

internal sealed partial class AsyncLowering
{
    private int Expression(BoundExpression expression, Func<BoundExpression, int> next, Context context)
    {
        if (!BoundTree.HasAwait(expression)) return next(expression);
        switch (expression)
        {
            case BoundAwait awaited:
                return Expression(awaited.Operand, operand =>
                {
                    var awaiter = Temporary(awaited.GetAwaiter.Result, "<awaiter>");
                    var result = awaited.Type == TypeSymbol.Void ? null : Temporary(awaited.Type);
                    var after = next(result is null ? new BoundNoValue() : new BoundLocal(result));
                    BoundStatement receive = result is null
                        ? new BoundExpressionStatement(new BoundCall(awaited.GetResult, [], new BoundLocal(awaiter)), false)
                        : new BoundBinding(result, new BoundCall(awaited.GetResult, [], new BoundLocal(awaiter)));
                    var resume = Block(context, new AsyncGoto(after), receive);
                    plan.Blocks[resume].Location = awaited.IsImplicit ? null : awaited.Location;
                    var suspend = Block(context, new AsyncSuspend(awaiter, awaited.IsCompleted, resume, awaited.Location));
                    plan.Blocks[suspend].Location = awaited.IsImplicit ? null : awaited.Location;
                    return Block(context, new AsyncGoto(suspend), new BoundBinding(awaiter, new BoundCall(awaited.GetAwaiter, [], operand)));
                }, context);
            case BoundSequence sequence:
                var value = Expression(sequence.Value, next, context);
                foreach (var binding in sequence.Bindings.Reverse()) value = Statement(binding, value, context);
                return value;
            case BoundBinary binary when binary.Left.Type == TypeSymbol.Boolean && binary.Operator is "and" or "or":
                var boolean = Temporary(TypeSymbol.Boolean);
                var done = next(new BoundLocal(boolean));
                var right = Expression(binary.Right, operand => Block(context, new AsyncGoto(done), new BoundBinding(boolean, operand)), context);
                var skipped = Block(context, new AsyncGoto(done), new BoundBinding(boolean, new BoundBoolean(binary.Operator == "or")));
                return Expression(binary.Left, operand => Block(context, binary.Operator == "and" ? new AsyncBranch(operand, right, skipped) : new AsyncBranch(operand, skipped, right)), context);
            case BoundBinary binary:
                return Values([binary.Left, binary.Right], values => next(binary with { Left = values[0], Right = values[1] }), context);
            case BoundConditional conditional:
                var conditionalResult = Temporary(conditional.Type);
                var following = next(new BoundLocal(conditionalResult));
                var fallback = Expression(conditional.Else, operand => Block(context, new AsyncGoto(following), new BoundBinding(conditionalResult, operand)), context);
                foreach (var arm in conditional.Arms.Reverse())
                {
                    var yes = Expression(arm.Value, operand => Block(context, new AsyncGoto(following), new BoundBinding(conditionalResult, operand)), context);
                    var no = fallback;
                    fallback = Expression(arm.Condition, condition => Block(context, new AsyncBranch(condition, yes, no)), context);
                }
                return fallback;
            case BoundCoalesce coalesce:
                var coalesced = Temporary(coalesce.Type);
                var continuation = next(new BoundLocal(coalesced));
                var empty = Expression(coalesce.Right, operand => Block(context, new AsyncGoto(continuation), new BoundBinding(coalesced, operand)), context);
                return Values([coalesce.Left], operands =>
                {
                    var present = Block(context, new AsyncGoto(continuation), new BoundBinding(coalesced, new BoundConversion(operands[0], coalesce.Type, true)));
                    return Block(context, new AsyncBranch(new BoundNullTest(operands[0], true), empty, present));
                }, context);
            case BoundCall call:
                return Values((call.Receiver is null ? [] : new[] { call.Receiver }).Concat(call.Arguments).ToArray(), values =>
                    next(call with { Receiver = call.Receiver is null ? null : values[0], Arguments = values.Skip(call.Receiver is null ? 0 : 1).ToArray() }), context);
            case BoundNew creation: return Values(creation.Arguments, values => next(creation with { Arguments = values }), context);
            case BoundArrayLiteral array: return Values(array.Elements, values => next(array with { Elements = values }), context);
            case BoundNewArray array: return Values(array.Lengths, values => next(array with { Lengths = values }), context);
            case BoundArrayIndex index: return Values(new[] { index.Receiver }.Concat(index.Indices).ToArray(), values => next(index with { Receiver = values[0], Indices = values.Skip(1).ToArray() }), context);
            case BoundArrayLength length: return Expression(length.Receiver, operand => next(length with { Receiver = operand }), context);
            case BoundProperty property:
                return Values((property.Receiver is null ? [] : new[] { property.Receiver }).Concat(property.Arguments ?? []).ToArray(), values =>
                    next(property with { Receiver = property.Receiver is null ? null : values[0], Arguments = values.Skip(property.Receiver is null ? 0 : 1).ToArray() }), context);
            case BoundField field: return Expression(field.Receiver!, operand => next(field with { Receiver = operand }), context);
            case BoundRecordField field: return Expression(field.Receiver, operand => next(field with { Receiver = operand }), context);
            case BoundUnary unary: return Expression(unary.Operand, operand => next(unary with { Operand = operand }), context);
            case BoundConversion conversion: return Expression(conversion.Operand, operand => next(conversion with { Operand = operand }), context);
            case BoundNullTest test: return Expression(test.Operand, operand => next(test with { Operand = operand }), context);
            case BoundTypeTest test: return Expression(test.Value, operand => next(test with { Value = operand }), context);
            case BoundSafeCast cast: return Expression(cast.Value, operand => next(cast with { Value = operand }), context);
            case BoundAddress address: return LValue(address.Target, target => next(address with { Target = target }), context);
            case BoundDereference dereference: return Expression(dereference.Address, operand => next(dereference with { Address = operand }), context);
            case BoundMethodDelegate method: return Expression(method.Receiver!, operand => next(method with { Receiver = operand }), context);
            default: throw new InvalidOperationException("Unknown suspending expression in async lowering.");
        }
    }

    private int Values(IReadOnlyList<BoundExpression> expressions, Func<IReadOnlyList<BoundExpression>, int> next, Context context)
    {
        var values = expressions.Select(expression => Temporary(expression.Type)).ToArray();
        var following = next(values.Select(local => (BoundExpression)new BoundLocal(local)).ToArray());
        for (var index = expressions.Count - 1; index >= 0; index--)
        {
            var local = values[index]; var target = following;
            following = Expression(expressions[index], value => Block(context, new AsyncGoto(target), new BoundBinding(local, value)), context);
        }
        return following;
    }

    private int LValue(BoundExpression target, Func<BoundExpression, int> next, Context context) => target switch
    {
        BoundArrayIndex index => Values(new[] { index.Receiver }.Concat(index.Indices).ToArray(), values => next(index with { Receiver = values[0], Indices = values.Skip(1).ToArray() }), context),
        BoundField { Receiver: not null } field => Values([field.Receiver], values => next(field with { Receiver = values[0] }), context),
        BoundProperty property => Values((property.Receiver is null ? [] : new[] { property.Receiver }).Concat(property.Arguments ?? []).ToArray(), values => next(property with
            { Receiver = property.Receiver is null ? null : values[0], Arguments = values.Skip(property.Receiver is null ? 0 : 1).ToArray() }), context),
        _ => next(target)
    };
}
