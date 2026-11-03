using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private BoundExpression BindLiftedUnary(BoundExpression operand, string op, int position, bool check)
    {
        if (operand.Type.NonNullable == TypeSymbol.Boolean)
            throw source.Error("FP3002", "Nullable Boolean logical operators are not supported; handle absence explicitly.", position);
        var bindings = new List<BoundBinding>();
        var saved = Save(operand, bindings);
        var result = BindUnaryOperation(new BoundConversion(saved, operand.Type.NonNullable, check), op, position, check, lifted: true);
        return LiftedResult(bindings, new BoundNullTest(saved, false), result, position);
    }

    private BoundExpression BindLiftedBinary(BoundExpression left, string op, BoundExpression right, BinarySyntax syntax, bool check)
    {
        if (op is "and" or "or" or "xor" && (left.Type.NonNullable == TypeSymbol.Boolean || right.Type.NonNullable == TypeSymbol.Boolean))
            throw source.Error("FP3002", "Nullable Boolean logical operators are not supported; handle absence explicitly.", syntax.Position);
        var bindings = new List<BoundBinding>();
        BoundExpression SaveOperand(BoundExpression value, TypeSymbol other)
            => Save(value.Type == TypeSymbol.Null ? Convert(value, other.NonNullable with { IsNullable = true }, syntax.Position) : value, bindings);
        var a = SaveOperand(left, right.Type);
        var b = SaveOperand(right, op is "shl" or "shr" ? TypeSymbol.Int32 : left.Type);
        BoundExpression Present(BoundExpression value) => value.Type.NullableValue ? new BoundNullTest(value, false) : new BoundBoolean(true);
        var presentA = Present(a); var presentB = Present(b);
        var present = new BoundBinary(presentA, "and", presentB, TypeSymbol.Boolean, false);
        var operation = BindBinaryOperation(LiftedOperand(left, a, check), op, LiftedOperand(right, b, check), syntax, check, lifted: true);
        if (op is "=" or "<>")
        {
            var absent = new BoundBinary(presentA, op, presentB, TypeSymbol.Boolean, false);
            return new BoundSequence(bindings, new BoundConditional([new(present, operation)], absent, TypeSymbol.Boolean));
        }
        if (op is "<" or ">" or "<=" or ">=")
            return new BoundSequence(bindings, new BoundConditional([new(present, operation)], new BoundBoolean(false), TypeSymbol.Boolean));
        return LiftedResult(bindings, present, operation, syntax.Position);
    }

    private static BoundExpression LiftedOperand(BoundExpression original, BoundExpression saved, bool check)
    {
        if (saved.Type.NullableValue) return new BoundConversion(saved, saved.Type.NonNullable, check);
        // Keep constant-expression promotions while evaluating the original operand before the null tests.
        if (NumericFacts.Integral(original.Type) && ConstantValue.IsConstant(original))
        {
            try { return BoundNumber.Create(original.Type, ConstantValue.Evaluate(original)!); }
            catch (Exception error) when (error is OverflowException or DivideByZeroException or InvalidOperationException) { }
        }
        return saved;
    }

    private BoundExpression LiftedResult(IReadOnlyList<BoundBinding> bindings, BoundExpression present, BoundExpression operation, int position)
    {
        if (!operation.Type.IsValue || operation.Type.IsNullable)
            throw source.Error("FP3002", "A lifted operator requires a non-nullable value result.", position);
        var result = operation.Type with { IsNullable = true };
        return new BoundSequence(bindings, new BoundConditional([new(present, new BoundConversion(operation, result, false))],
            new BoundConversion(new BoundNull(), result, false), result));
    }
}
