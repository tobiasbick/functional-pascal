using System.Numerics;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Evaluation;

internal static partial class ScalarOperations
{
    internal static DebugBinding Unary(string op, DebugBinding value)
    {
        Available(value);
        DebugBinding result;
        if (op == "not" && value.Type == "bool") result = Boolean(value.Scalar != "true");
        else
        {
            var kind = UnaryKind(Kind(value.Type), op);
            if (kind == 0 || op == "not" && !Integral(kind)) throw Error("The unary operator requires a supported primitive scalar.");
            result = op == "+" ? Convert(value, kind) : kind == 80 ? Decimal(-Money(value)) : kind is 12 or 13 ? Floating(kind, -Real(value))
                : Integer(kind, op == "not" ? ~Number(value) : -Number(value), check: op != "not");
        }
        return result with { Constant = value.Constant };
    }

    internal static DebugBinding Binary(DebugBinding left, string op, DebugBinding right)
    {
        Available(left); Available(right);
        return Compute(left, op, right) with { Constant = left.Constant && right.Constant };
    }

    private static DebugBinding Compute(DebugBinding left, string op, DebugBinding right)
    {
        if (op is "=" or "<>" && (left.Kind == "null" || right.Kind == "null"))
        {
            var operand = left.Kind == "null" ? right : left;
            if (operand.Kind != "null" && operand.Type != "string" && operand.RuntimeType?.ElementKind is not (14 or 18 or 20 or 28 or 29)) throw Error("Only reference values can be compared with null in passive expressions.");
            return Boolean((left.Kind == right.Kind) == (op == "="));
        }
        if (left.Type == "bool" && right.Type == "bool")
        {
            var a = left.Scalar == "true"; var b = right.Scalar == "true";
            return Boolean(op switch { "and" => a && b, "or" => a || b, "xor" => a ^ b, "=" => a == b, "<>" => a != b, _ => throw Error("Unsupported Boolean operator.") });
        }
        if (left.Type == "string" && right.Type == "string" && op is "+" or "=" or "<>"
            || op == "+" && (left.Type == "string" && right.Type == "char" || left.Type == "char" && right.Type == "string"))
        {
            Complete(left); Complete(right); var a = StoredString(left); var b = StoredString(right);
            return op == "+" ? Text(a + b) : Boolean((a == b) == (op == "="));
        }
        var shift = op is "shl" or "shr"; var kind = BinaryKind(left, op, right);
        if (kind == 0) throw Error("The operands have no supported primitive representation.");
        if (kind is 12 or 13)
        {
            var a = Real(left); var b = Real(right);
            if (kind == 12) { a = (float)a; b = (float)b; }
            if (Comparison(op)) return Compare(op, a == b, a < b, a > b, a <= b, a >= b);
            return Floating(kind, op switch { "+" => a + b, "-" => a - b, "*" => a * b, "/" => a / b, _ => throw Error("Unsupported floating operator.") });
        }
        if (kind == 80)
        {
            var a = Money(left); var b = Money(right);
            if (Comparison(op)) return Compare(op, a == b, a < b, a > b, a <= b, a >= b);
            return Decimal(op switch { "+" => a + b, "-" => a - b, "*" => a * b, "/" => a / b, _ => throw Error("Unsupported decimal operator.") });
        }
        if (!Integral(kind) || !Integral(Kind(right.Type))) throw Error("The operator requires integral operands.");
        var x = Number(left); var y = Number(right);
        if (Comparison(op)) return Compare(op, x == y, x < y, x > y, x <= y, x >= y);
        if (shift && !Widens(Kind(right.Type), 8) && !(right.Constant && Fits(y, 8))) throw Error("A shift count must convert implicitly to int.");
        if (op is "div" or "mod" && y.IsZero) throw new DebugException("division_by_zero", "Integral division by zero.");
        if (op is "div" or "mod" && !Unsigned(kind) && x == Minimum(kind) && y == -1)
        { if (op == "div") throw new DebugException("overflow", "Integral division overflows its type."); return Integer(kind, BigInteger.Zero); }
        var count = (int)(y & (Bits(kind) == 64 ? 63 : 31));
        var number = op switch
        {
            "+" => x + y, "-" => x - y, "*" => x * y, "div" => x / y, "mod" => x % y,
            "and" => x & y, "or" => x | y, "xor" => x ^ y, "shl" => x << count, "shr" => x >> count,
            _ => throw Error("The numeric operator does not accept these operands.")
        };
        return Integer(kind, number, check: op is "+" or "-" or "*" or "div");
    }

    private static DebugBinding Convert(DebugBinding value, uint kind) => kind == 80 ? Decimal(Money(value)) : kind is 12 or 13 ? Floating(kind, Real(value)) : Integer(kind, Number(value));
    private static bool Comparison(string op) => op is "=" or "<>" or "<" or ">" or "<=" or ">=";
    private static DebugBinding Compare(string op, bool equal, bool less, bool greater, bool lessEqual, bool greaterEqual)
        => Boolean(op switch { "=" => equal, "<>" => !equal, "<" => less, ">" => greater, "<=" => lessEqual, ">=" => greaterEqual, _ => false });
    internal static void Available(DebugBinding value)
    {
        if (value.Status != "available") throw new DebugException(value.Error?.Code ?? value.Status, value.Error?.Message ?? "The requested value is unavailable at this stop.");
    }
    private static void Complete(DebugBinding value)
    { if (value.Truncated) throw new DebugException("truncated_value", "The operation needs string content omitted by the inspection limit."); }
}
