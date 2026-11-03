using System.Globalization;
using System.Numerics;

namespace Fpas.Compiler.Binding;

internal static class NumericValue
{
    internal static BigInteger Integer(object value) => value switch
    {
        BigInteger number => number, char character => character, ulong number => number, long number => number,
        uint number => number, int number => number, ushort number => number, short number => number, byte number => number, sbyte number => number,
        nint number => (long)number, nuint number => (ulong)number, decimal number => new(number), double number => new(number), float number => new(number),
        _ => throw new InvalidOperationException("An integral constant is required.")
    };

    internal static object ConvertTo(object value, TypeSymbol target, bool check)
    {
        target = target.NonNullable;
        if (value is char character) value = (ushort)character;
        if (target == TypeSymbol.Double) return value is nint ni ? (double)ni : value is nuint nu ? (double)nu : System.Convert.ToDouble(value, CultureInfo.InvariantCulture);
        if (target == TypeSymbol.Single) return value is nint ni ? (float)ni : value is nuint nu ? (float)nu : System.Convert.ToSingle(value, CultureInfo.InvariantCulture);
        if (target == TypeSymbol.Decimal) return value is nint ni ? (decimal)(long)ni : value is nuint nu ? (decimal)(ulong)nu : System.Convert.ToDecimal(value, CultureInfo.InvariantCulture);
        if (!NumericFacts.Integral(target)) return value;
        if (value is double real) return FloatingIntegral(real, target, check);
        if (value is float single) return FloatingIntegral(single, target, check);
        if (value is decimal) check = true;
        var number = Integer(value);
        if (check && !NumericFacts.Fits(number, target)) throw new OverflowException();
        var modulus = BigInteger.One << NumericFacts.Bits(target);
        number = (number % modulus + modulus) % modulus;
        if (!NumericFacts.Unsigned(target) && number > NumericFacts.Max(target)) number -= modulus;
        return target.Kind switch
        {
            TypeKind.SByte => (sbyte)number, TypeKind.Byte => (byte)number, TypeKind.Int16 => (short)number, TypeKind.UInt16 => (ushort)number,
            TypeKind.Int32 => (int)number, TypeKind.UInt32 => (uint)number, TypeKind.Int64 => (long)number, TypeKind.UInt64 => (ulong)number,
            TypeKind.Char => (char)(ushort)number, TypeKind.NativeInt => (nint)(long)number, TypeKind.NativeUInt => (nuint)(ulong)number,
            _ => throw new InvalidOperationException()
        };
    }

    private static object FloatingIntegral(double value, TypeSymbol target, bool check) => (target.Kind, check) switch
    {
        (TypeKind.SByte, true) => checked((sbyte)value), (TypeKind.SByte, false) => unchecked((sbyte)value),
        (TypeKind.Byte, true) => checked((byte)value), (TypeKind.Byte, false) => unchecked((byte)value),
        (TypeKind.Int16, true) => checked((short)value), (TypeKind.Int16, false) => unchecked((short)value),
        (TypeKind.UInt16, true) => checked((ushort)value), (TypeKind.UInt16, false) => unchecked((ushort)value),
        (TypeKind.Int32, true) => checked((int)value), (TypeKind.Int32, false) => unchecked((int)value),
        (TypeKind.UInt32, true) => checked((uint)value), (TypeKind.UInt32, false) => unchecked((uint)value),
        (TypeKind.Int64, true) => checked((long)value), (TypeKind.Int64, false) => unchecked((long)value),
        (TypeKind.UInt64, true) => checked((ulong)value), (TypeKind.UInt64, false) => unchecked((ulong)value),
        (TypeKind.Char, true) => checked((char)value), (TypeKind.Char, false) => unchecked((char)value),
        (TypeKind.NativeInt, true) => checked((nint)value), (TypeKind.NativeInt, false) => unchecked((nint)value),
        (TypeKind.NativeUInt, true) => checked((nuint)value), (TypeKind.NativeUInt, false) => unchecked((nuint)value),
        _ => throw new InvalidOperationException()
    };

    internal static object Unary(string op, object value, TypeSymbol type, bool check)
    {
        if (op == "+") return value;
        if (op == "not" && value is bool boolean) return !boolean;
        if (type == TypeSymbol.Decimal) return -(decimal)value;
        if (type == TypeSymbol.Single) return -(float)value;
        if (type == TypeSymbol.Double) return -(double)value;
        return ConvertTo(op == "not" ? ~Integer(value) : -Integer(value), type, op != "not" && check);
    }

    internal static object Binary(string op, object left, object right, TypeSymbol type, bool check)
    {
        if (NumericFacts.Floating(type))
        {
            var a = System.Convert.ToDouble(left, CultureInfo.InvariantCulture); var b = System.Convert.ToDouble(right, CultureInfo.InvariantCulture);
            if (op is "=" or "<>" or "<" or ">" or "<=" or ">=") return Compare(op, a == b, a < b, a > b, a <= b, a >= b);
            var value = op switch { "+" => a + b, "-" => a - b, "*" => a * b, "/" => a / b, _ => throw new InvalidOperationException() };
            return type == TypeSymbol.Single ? (object)(float)value : value;
        }
        if (type == TypeSymbol.Decimal)
        {
            var a = (decimal)left; var b = (decimal)right;
            if (op is "=" or "<>" or "<" or ">" or "<=" or ">=") return Compare(op, a == b, a < b, a > b, a <= b, a >= b);
            return op switch { "+" => a + b, "-" => a - b, "*" => a * b, "/" => a / b, _ => throw new InvalidOperationException() };
        }
        var x = Integer(left); var y = Integer(right);
        if (op is "=" or "<>" or "<" or ">" or "<=" or ">=") return Compare(op, x == y, x < y, x > y, x <= y, x >= y);
        if (op is "div" or "mod" && !NumericFacts.Unsigned(type) && x == NumericFacts.Min(type) && y == -1)
        {
            if (op == "div") throw new OverflowException();
            return ConvertTo(BigInteger.Zero, type, false);
        }
        var count = (int)(y & (NumericFacts.Bits(type) == 64 ? 63 : 31));
        var result = op switch
        {
            "+" => x + y, "-" => x - y, "*" => x * y, "div" => x / y, "mod" => x % y,
            "and" => x & y, "or" => x | y, "xor" => x ^ y, "shl" => x << count, "shr" => x >> count,
            _ => throw new InvalidOperationException()
        };
        return ConvertTo(result, type, check && op is "+" or "-" or "*" or "div");
    }

    private static bool Compare(string op, bool eq, bool lt, bool gt, bool le, bool ge)
        => op switch { "=" => eq, "<>" => !eq, "<" => lt, ">" => gt, "<=" => le, ">=" => ge, _ => false };
}
