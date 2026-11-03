using System.Globalization;

namespace Fpas.Compiler.Binding;

internal static class ConstantValue
{
    internal static bool IsConstant(BoundExpression expression) => expression switch
    {
        BoundNull or BoundInteger or BoundDouble or BoundNumber or BoundBoolean or BoundCharacter or BoundString => true,
        BoundField { Field.IsLiteral: true } => true,
        BoundConversion conversion => IsConstant(conversion.Operand),
        BoundUnary unary => IsConstant(unary.Operand),
        BoundBinary binary => IsConstant(binary.Left) && IsConstant(binary.Right),
        BoundNullTest test => IsConstant(test.Operand),
        BoundConditional conditional => conditional.Arms.All(arm => IsConstant(arm.Condition) && IsConstant(arm.Value)) && IsConstant(conditional.Else),
        BoundCall { Method.Owner.Kind: TypeKind.String, Method.Name: "Concat" or "op_Equality" or "op_Inequality" } call => call.Arguments.All(IsConstant),
        BoundCall { Method.Owner.Kind: TypeKind.Char, Method.Name: "ToString", Arguments.Count: 0, Receiver: { } receiver } => IsConstant(receiver),
        _ => false
    };

    internal static object? Evaluate(BoundExpression expression) => expression switch
    {
        BoundNull => null,
        BoundInteger number => number.Value,
        BoundDouble number => number.Value,
        BoundNumber number => number.Value,
        BoundBoolean value => value.Value,
        BoundCharacter value => value.Value,
        BoundString text => text.Value,
        BoundField { Field.IsLiteral: true } field => field.Field.Constant,
        BoundConversion conversion => Evaluate(conversion.Operand) is { } value && TypeFacts.Numeric(conversion.Target.NonNullable)
            ? NumericValue.ConvertTo(value, conversion.Target, conversion.Checked) : Evaluate(conversion.Operand),
        BoundUnary unary => Unary(unary),
        BoundBinary binary => Binary(binary),
        BoundNullTest test => (Evaluate(test.Operand) is null) == test.EqualsNull,
        BoundConditional conditional => Evaluate(conditional.Arms.FirstOrDefault(arm => (bool)Evaluate(arm.Condition)!)?.Value ?? conditional.Else),
        BoundCall { Method.Owner.Kind: TypeKind.String, Method.Name: "Concat" } call => string.Concat(call.Arguments.Select(Evaluate)),
        BoundCall { Method.Owner.Kind: TypeKind.Char, Method.Name: "ToString", Arguments.Count: 0, Receiver: { } receiver } => ((char)Evaluate(receiver)!).ToString(),
        BoundCall { Method.Owner.Kind: TypeKind.String, Method.Name: "op_Equality" or "op_Inequality" } call
            => Equals(Evaluate(call.Arguments[0]), Evaluate(call.Arguments[1])) == (call.Method.Name == "op_Equality"),
        _ => throw new InvalidOperationException("A compile-time constant is required.")
    };

    private static object Unary(BoundUnary unary) => NumericValue.Unary(unary.Operator, Evaluate(unary.Operand)!, unary.Type, unary.Checked);

    private static object Binary(BoundBinary binary)
    {
        var left = Evaluate(binary.Left);
        if (left is bool condition && binary.Operator == "and" && !condition) return false;
        if (left is bool alternative && binary.Operator == "or" && alternative) return true;
        var right = Evaluate(binary.Right);
        if (TypeFacts.Numeric(binary.Left.Type)) return NumericValue.Binary(binary.Operator, left!, right!, binary.Left.Type, binary.Checked);
        var equal = left is double first && right is double second ? first == second : Equals(left, right);
        if (binary.Operator == "=") return equal;
        if (binary.Operator == "<>") return !equal;
        if (left is bool l && right is bool r) return binary.Operator switch
        { "and" => l && r, "or" => l || r, "xor" => l ^ r, _ => throw new InvalidOperationException() };
        throw new InvalidOperationException("Unsupported constant operation.");
    }
}
