// [Documentation](../../../../docs/pascal/language/basics/primitive-types.md)
using Fpas.Compiler.Binding.Types;
using System.Globalization;

namespace Fpas.Compiler.Binding.Bound;

internal static class ConstantValue
{
    internal static bool IsConstant(BoundExpression expression) => expression switch
    {
        BoundNull or BoundInteger or BoundDouble or BoundNumber or BoundBoolean or BoundCharacter or BoundString => true,
        BoundField { Field.IsLiteral: true } => true,
        BoundConversion conversion => ConstantConversion(conversion) && IsConstant(conversion.Operand),
        BoundUnary unary => IsConstant(unary.Operand),
        BoundBinary binary => !binary.Left.Type.IsReference && IsConstant(binary.Left) && IsConstant(binary.Right),
        BoundNullTest test => IsConstant(test.Operand),
        BoundConditional conditional => conditional.Arms.All(arm => IsConstant(arm.Condition) && IsConstant(arm.Value)) && IsConstant(conditional.Else),
        BoundCall { Method.Owner.Kind: TypeKind.String, Method.Name: "Concat" or "op_Equality" or "op_Inequality" } call => call.Arguments.All(IsConstant),
        BoundCall { Method.Owner.Kind: TypeKind.Char, Method.Name: "ToString", Arguments.Count: 0, Receiver: { } receiver } => IsConstant(receiver),
        _ => false
    };

    private static bool ConstantConversion(BoundConversion conversion)
    {
        var from = conversion.Operand.Type.NonNullable;
        var to = conversion.Target.NonNullable;
        static bool Numeric(TypeSymbol type) => TypeFacts.Numeric(type) || type.Definition?.IsEnum == true ||
            type.Scalar?.Syntax.Category == Syntax.ScalarCategory.Enum;
        if (conversion.Operand.Type.NullableValue && conversion.Target.IsValue && !conversion.Target.NullableValue) return false;
        // Boxing and reference upcasts to object preserve a constant's payload.
        // Unboxing and reference downcasts can throw and are runtime operations.
        return from == TypeSymbol.Null || from == to || to == TypeSymbol.Object || Numeric(from) && Numeric(to);
    }

    internal static object? Evaluate(BoundExpression expression)
    {
        var narrow = Evaluate(expression, 32);
        var wide = Evaluate(expression, 64);
        var same = (narrow, wide) switch
        {
            (float left, float right) => BitConverter.SingleToInt32Bits(left) == BitConverter.SingleToInt32Bits(right),
            (double left, double right) => BitConverter.DoubleToInt64Bits(left) == BitConverter.DoubleToInt64Bits(right),
            (decimal left, decimal right) => decimal.GetBits(left).SequenceEqual(decimal.GetBits(right)),
            _ => Equals(narrow, wide)
        };
        if (!same) throw new InvalidOperationException("A compile-time constant must have the same value on 32-bit and 64-bit runtimes.");
        return narrow is not null && expression.Type.NonNullable.Kind is TypeKind.NativeInt or TypeKind.NativeUInt
            ? NumericValue.ConvertTo(narrow, expression.Type, true) : narrow;
    }

    private static object? Evaluate(BoundExpression expression, int nativeBits) => expression switch
    {
        BoundNull => null,
        BoundInteger number => number.Value,
        BoundDouble number => number.Value,
        BoundNumber number => Literal(number.Type, number.Value, nativeBits),
        BoundBoolean value => value.Value,
        BoundCharacter value => value.Value,
        BoundString text => text.Value,
        BoundField { Field.IsLiteral: true } field => Literal(field.Type, field.Field.Constant, nativeBits),
        BoundConversion conversion when ConstantConversion(conversion) => Evaluate(conversion.Operand, nativeBits) is { } value && TypeFacts.Numeric(conversion.Target.NonNullable)
            ? NumericValue.ConvertTo(value, conversion.Target, conversion.Checked, nativeBits) : Evaluate(conversion.Operand, nativeBits),
        BoundUnary unary => Unary(unary, nativeBits),
        // Reference equality observes allocation/boxing identity, not the values
        // retained by constant evaluation. String value equality uses BoundCall.
        BoundBinary binary when !binary.Left.Type.IsReference => Binary(binary, nativeBits),
        BoundNullTest test => (Evaluate(test.Operand, nativeBits) is null) == test.EqualsNull,
        BoundConditional conditional => Evaluate(conditional.Arms.FirstOrDefault(arm => (bool)Evaluate(arm.Condition, nativeBits)!)?.Value ?? conditional.Else, nativeBits),
        BoundCall { Method.Owner.Kind: TypeKind.String, Method.Name: "Concat" } call => string.Concat(call.Arguments.Select(argument => Evaluate(argument, nativeBits))),
        BoundCall { Method.Owner.Kind: TypeKind.Char, Method.Name: "ToString", Arguments.Count: 0, Receiver: { } receiver } => ((char)Evaluate(receiver, nativeBits)!).ToString(),
        BoundCall { Method.Owner.Kind: TypeKind.String, Method.Name: "op_Equality" or "op_Inequality" } call
            => Equals(Evaluate(call.Arguments[0], nativeBits), Evaluate(call.Arguments[1], nativeBits)) == (call.Method.Name == "op_Equality"),
        _ => throw new InvalidOperationException("A compile-time constant is required.")
    };

    private static object? Literal(TypeSymbol type, object? value, int nativeBits)
        => value is not null && type.NonNullable.Kind is TypeKind.NativeInt or TypeKind.NativeUInt
            ? NumericValue.ConvertTo(value, type, true, nativeBits) : value;

    private static object Unary(BoundUnary unary, int nativeBits)
        => NumericValue.Unary(unary.Operator, Evaluate(unary.Operand, nativeBits)!, unary.Type, unary.Checked, nativeBits);

    private static object Binary(BoundBinary binary, int nativeBits)
    {
        var left = Evaluate(binary.Left, nativeBits);
        if (left is bool condition && binary.Operator == "and" && !condition) return false;
        if (left is bool alternative && binary.Operator == "or" && alternative) return true;
        var right = Evaluate(binary.Right, nativeBits);
        if (TypeFacts.Numeric(binary.Left.Type)) return NumericValue.Binary(binary.Operator, left!, right!, binary.Left.Type, binary.Checked, nativeBits);
        var equal = left is double first && right is double second ? first == second : Equals(left, right);
        if (binary.Operator == "=") return equal;
        if (binary.Operator == "<>") return !equal;
        if (left is bool l && right is bool r) return binary.Operator switch
        { "and" => l && r, "or" => l || r, "xor" => l ^ r, _ => throw new InvalidOperationException() };
        throw new InvalidOperationException("Unsupported constant operation.");
    }
}
