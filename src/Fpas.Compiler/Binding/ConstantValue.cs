using System.Globalization;

namespace Fpas.Compiler.Binding;

internal static class ConstantValue
{
    internal static bool IsConstant(BoundExpression expression) => expression switch
    {
        BoundNull or BoundInteger or BoundDouble or BoundBoolean or BoundCharacter or BoundString => true,
        BoundField { Field.IsLiteral: true } => true,
        BoundConversion conversion => IsConstant(conversion.Operand),
        BoundUnary unary => IsConstant(unary.Operand),
        BoundBinary binary => IsConstant(binary.Left) && IsConstant(binary.Right),
        BoundNullTest test => IsConstant(test.Operand),
        BoundConditional conditional => conditional.Arms.All(arm => IsConstant(arm.Condition) && IsConstant(arm.Value)) && IsConstant(conditional.Else),
        BoundCall { Method.Owner.Kind: TypeKind.String, Method.Name: "Concat" or "op_Equality" or "op_Inequality" } call => call.Arguments.All(IsConstant),
        _ => false
    };

    internal static object? Evaluate(BoundExpression expression) => expression switch
    {
        BoundNull => null,
        BoundInteger number => number.Value,
        BoundDouble number => number.Value,
        BoundBoolean value => value.Value,
        BoundCharacter value => value.Value,
        BoundString text => text.Value,
        BoundField { Field.IsLiteral: true } field => field.Field.Constant,
        BoundConversion conversion => ConvertValue(Evaluate(conversion.Operand), conversion.Target, conversion.Checked),
        BoundUnary unary => Unary(unary),
        BoundBinary binary => Binary(binary),
        BoundNullTest test => (Evaluate(test.Operand) is null) == test.EqualsNull,
        BoundConditional conditional => Evaluate(conditional.Arms.FirstOrDefault(arm => (bool)Evaluate(arm.Condition)!)?.Value ?? conditional.Else),
        BoundCall { Method.Owner.Kind: TypeKind.String, Method.Name: "Concat" } call => string.Concat(call.Arguments.Select(Evaluate)),
        BoundCall { Method.Owner.Kind: TypeKind.String, Method.Name: "op_Equality" or "op_Inequality" } call
            => Equals(Evaluate(call.Arguments[0]), Evaluate(call.Arguments[1])) == (call.Method.Name == "op_Equality"),
        _ => throw new InvalidOperationException("A compile-time constant is required.")
    };

    private static object? ConvertValue(object? value, TypeSymbol target, bool check)
    {
        if (value is null) return null;
        if (target.NullableValue) target = target.NonNullable;
        if (target == TypeSymbol.Double) return Convert.ToDouble(value, CultureInfo.InvariantCulture);
        if (target == TypeSymbol.Int32 && value is double real) return check ? checked((int)real) : unchecked((int)real);
        if (target == TypeSymbol.Int32 && value is char character) return (int)character;
        if (target == TypeSymbol.Char)
        {
            var number = Convert.ToDouble(value, CultureInfo.InvariantCulture);
            return check ? checked((char)number) : unchecked((char)number);
        }
        return value;
    }

    private static object Unary(BoundUnary unary)
    {
        var value = Evaluate(unary.Operand);
        return unary.Operator switch
        {
            "+" => value!, "not" when value is bool boolean => !boolean, "not" => ~(int)value!,
            "-" when value is double real => -real,
            "-" => unary.Checked ? checked(-(int)value!) : unchecked(-(int)value!),
            _ => throw new InvalidOperationException()
        };
    }

    private static object Binary(BoundBinary binary)
    {
        var left = Evaluate(binary.Left);
        if (left is bool condition && binary.Operator == "and" && !condition) return false;
        if (left is bool alternative && binary.Operator == "or" && alternative) return true;
        var right = Evaluate(binary.Right);
        var equal = left is double first && right is double second ? first == second : Equals(left, right);
        if (binary.Operator == "=") return equal;
        if (binary.Operator == "<>") return !equal;
        if (left is bool l && right is bool r) return binary.Operator switch
        { "and" => l && r, "or" => l || r, "xor" => l ^ r, _ => throw new InvalidOperationException() };
        if (left is double x && right is double y) return binary.Operator switch
        { "+" => x + y, "-" => x - y, "*" => x * y, "/" => x / y, "<" => x < y, ">" => x > y, "<=" => x <= y, ">=" => x >= y, _ => throw new InvalidOperationException() };
        var a = (int)left!; var b = (int)right!;
        return binary.Operator switch
        {
            "+" => binary.Checked ? checked(a + b) : unchecked(a + b),
            "-" => binary.Checked ? checked(a - b) : unchecked(a - b),
            "*" => binary.Checked ? checked(a * b) : unchecked(a * b),
            "div" => a / b, "mod" => a % b, "and" => a & b, "or" => a | b, "xor" => a ^ b,
            "shl" => a << b, "shr" => a >> b, "<" => a < b, ">" => a > b, "<=" => a <= b, ">=" => a >= b,
            _ => throw new InvalidOperationException()
        };
    }
}
