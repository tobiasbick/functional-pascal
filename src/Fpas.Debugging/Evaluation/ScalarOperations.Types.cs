using System.Globalization;
using System.Numerics;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Evaluation;

internal static partial class ScalarOperations
{
    private static uint Kind(string type) => type switch
    {
        "char" => 3, "sbyte" => 4, "byte" => 5, "short" => 6, "ushort" => 7, "int" => 8, "uint" => 9,
        "long" => 10, "ulong" => 11, "float" => 12, "double" => 13, "decimal" => 80, "nint" => 24, "nuint" => 25, _ => 0
    };
    private static bool Unsigned(uint kind) => kind is 3 or 5 or 7 or 9 or 11 or 25;
    private static bool Integral(uint kind) => kind is >= 3 and <= 11 or 24 or 25;
    private static int Bits(uint kind) => kind switch { 4 or 5 => 8, 3 or 6 or 7 => 16, 10 or 11 or 24 or 25 => 64, _ => 32 };
    private static BigInteger Minimum(uint kind) => Unsigned(kind) ? BigInteger.Zero : -(BigInteger.One << (Bits(kind) - 1));
    private static BigInteger Maximum(uint kind) => (BigInteger.One << (Bits(kind) - (Unsigned(kind) ? 0 : 1))) - 1;
    private static bool Fits(BigInteger value, uint kind) => Integral(kind) && value >= Minimum(kind) && value <= Maximum(kind);
    private static BigInteger Number(DebugBinding value) => value.Type == "char" ? StoredString(value)[0] : BigInteger.Parse(value.Scalar!, CultureInfo.InvariantCulture);

    private static bool Widens(uint from, uint to) => from == to || from switch
    {
        4 => to is 6 or 8 or 10 or 24 or 12 or 13 or 80,
        5 => to is 6 or 7 or 8 or 9 or 10 or 11 or 24 or 25 or 12 or 13 or 80,
        6 => to is 8 or 10 or 24 or 12 or 13 or 80,
        3 or 7 => to is 8 or 9 or 10 or 11 or 24 or 25 or 12 or 13 or 80 || from == 3 && to == 7,
        8 => to is 10 or 24 or 12 or 13 or 80, 9 => to is 10 or 11 or 25 or 12 or 13 or 80,
        24 => to is 10 or 12 or 13 or 80, 25 => to is 11 or 12 or 13 or 80,
        10 or 11 => to is 12 or 13 or 80, 12 => to == 13, _ => false
    };

    private static uint BinaryKind(DebugBinding left, string op, DebugBinding right)
    {
        var kind = Kind(left.Type);
        if (op is "shl" or "shr") return UnaryKind(kind, "+");
        if (op == "/" && Integral(kind) && Integral(Kind(right.Type))) return 13;
        return Promote(left, right);
    }

    private static uint Promote(DebugBinding left, DebugBinding right)
    {
        var a = Kind(left.Type); var b = Kind(right.Type);
        if (a == 0 || b == 0) throw Error("Only primitive scalar operations are available; overloaded operators are not evaluated.");
        if (a is 9 or 11 or 25 && right.Constant && (b == 8 || b == 10 && a == 11) && Fits(Number(right), a)) b = a;
        else if (b is 9 or 11 or 25 && left.Constant && (a == 8 || a == 10 && b == 11) && Fits(Number(left), b)) a = b;
        if (a == 80 || b == 80)
        { if (a is 12 or 13 || b is 12 or 13) throw Error("Decimal and binary floating point do not mix implicitly."); return 80; }
        if (a == 13 || b == 13) return 13;
        if (a == 12 || b == 12) return 12;
        foreach (var kind in new uint[] { 8, 9, 24, 25, 10, 11 }) if (Widens(a, kind) && Widens(b, kind)) return kind;
        throw Error("The numeric operands have no implicit common type.");
    }
    private static uint UnaryKind(uint kind, string op) => kind is 3 or 4 or 5 or 6 or 7 ? 8u
        : op == "-" && kind == 9 ? 10u : op == "-" && kind is 11 or 25 ? 0 : kind;
    private static double Real(DebugBinding value) => value.Type == "char" ? (double)Number(value) : double.Parse(value.Scalar!, CultureInfo.InvariantCulture);
    private static decimal Money(DebugBinding value) => value.Type == "char" ? (decimal)Number(value) : decimal.Parse(value.Scalar!, NumberStyles.Float, CultureInfo.InvariantCulture);
}
