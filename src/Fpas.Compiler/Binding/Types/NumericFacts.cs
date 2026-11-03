using System.Numerics;

namespace Fpas.Compiler.Binding.Types;

internal static class NumericFacts
{
    internal static bool NumericKind(TypeKind kind) => kind is TypeKind.SByte or TypeKind.Byte or TypeKind.Int16 or TypeKind.UInt16 or TypeKind.Int32 or TypeKind.UInt32 or
        TypeKind.Int64 or TypeKind.UInt64 or TypeKind.NativeInt or TypeKind.NativeUInt or TypeKind.Single or TypeKind.Double or TypeKind.Decimal or TypeKind.Char;
    internal static bool Integral(TypeSymbol type) => NumericKind(type.Kind) && type.Kind is not (TypeKind.Single or TypeKind.Double or TypeKind.Decimal) && !type.IsNullable;
    internal static bool Unsigned(TypeSymbol type) => type.Kind is TypeKind.Byte or TypeKind.UInt16 or TypeKind.UInt32 or TypeKind.UInt64 or TypeKind.NativeUInt or TypeKind.Char;
    internal static bool Floating(TypeSymbol type) => type.Kind is TypeKind.Single or TypeKind.Double;
    internal static int Bits(TypeSymbol type) => type.Kind switch
    {
        TypeKind.SByte or TypeKind.Byte => 8, TypeKind.Int16 or TypeKind.UInt16 or TypeKind.Char => 16,
        TypeKind.Int64 or TypeKind.UInt64 => 64, TypeKind.NativeInt or TypeKind.NativeUInt => IntPtr.Size * 8, _ => 32
    };
    internal static BigInteger Min(TypeSymbol type) => Unsigned(type) ? BigInteger.Zero : -(BigInteger.One << (Bits(type) - 1));
    internal static BigInteger Max(TypeSymbol type) => (BigInteger.One << (Bits(type) - (Unsigned(type) ? 0 : 1))) - 1;
    internal static bool Fits(BigInteger value, TypeSymbol type) => Integral(type) && value >= Min(type) && value <= Max(type);

    internal static bool Widen(TypeSymbol from, TypeSymbol to)
    {
        if (from == to) return true;
        if (!TypeFacts.Numeric(from) || !TypeFacts.Numeric(to)) return false;
        return from.Kind switch
        {
            TypeKind.SByte => to.Kind is TypeKind.Int16 or TypeKind.Int32 or TypeKind.Int64 or TypeKind.NativeInt or TypeKind.Single or TypeKind.Double or TypeKind.Decimal,
            TypeKind.Byte => to.Kind is TypeKind.Int16 or TypeKind.UInt16 or TypeKind.Int32 or TypeKind.UInt32 or TypeKind.Int64 or TypeKind.UInt64 or TypeKind.NativeInt or TypeKind.NativeUInt or TypeKind.Single or TypeKind.Double or TypeKind.Decimal,
            TypeKind.Int16 => to.Kind is TypeKind.Int32 or TypeKind.Int64 or TypeKind.NativeInt or TypeKind.Single or TypeKind.Double or TypeKind.Decimal,
            TypeKind.UInt16 or TypeKind.Char => to.Kind is TypeKind.Int32 or TypeKind.UInt32 or TypeKind.Int64 or TypeKind.UInt64 or TypeKind.NativeInt or TypeKind.NativeUInt or TypeKind.Single or TypeKind.Double or TypeKind.Decimal || from.Kind == TypeKind.Char && to.Kind == TypeKind.UInt16,
            TypeKind.Int32 => to.Kind is TypeKind.Int64 or TypeKind.NativeInt or TypeKind.Single or TypeKind.Double or TypeKind.Decimal,
            TypeKind.UInt32 => to.Kind is TypeKind.Int64 or TypeKind.UInt64 or TypeKind.NativeUInt or TypeKind.Single or TypeKind.Double or TypeKind.Decimal,
            TypeKind.NativeInt => to.Kind is TypeKind.Int64 or TypeKind.Single or TypeKind.Double or TypeKind.Decimal,
            TypeKind.NativeUInt => to.Kind is TypeKind.UInt64 or TypeKind.Single or TypeKind.Double or TypeKind.Decimal,
            TypeKind.Int64 or TypeKind.UInt64 => to.Kind is TypeKind.Single or TypeKind.Double or TypeKind.Decimal,
            TypeKind.Single => to.Kind == TypeKind.Double, _ => false
        };
    }

    internal static TypeSymbol Promote(TypeSymbol left, TypeSymbol right)
    {
        if (!TypeFacts.Numeric(left) || !TypeFacts.Numeric(right)) return TypeSymbol.Unsupported;
        if (left == TypeSymbol.Decimal || right == TypeSymbol.Decimal)
            return Floating(left) || Floating(right) ? TypeSymbol.Unsupported : TypeSymbol.Decimal;
        if (left == TypeSymbol.Double || right == TypeSymbol.Double) return TypeSymbol.Double;
        if (left == TypeSymbol.Single || right == TypeSymbol.Single) return TypeSymbol.Single;
        return new[] { TypeSymbol.Int32, TypeSymbol.UInt32, TypeSymbol.NativeInt, TypeSymbol.NativeUInt, TypeSymbol.Int64, TypeSymbol.UInt64 }
            .FirstOrDefault(type => Widen(left, type) && Widen(right, type)) ?? TypeSymbol.Unsupported;
    }

    internal static TypeSymbol Unary(TypeSymbol type, string op)
        => type.Kind is TypeKind.SByte or TypeKind.Byte or TypeKind.Int16 or TypeKind.UInt16 or TypeKind.Char ? TypeSymbol.Int32 :
            op == "-" && type == TypeSymbol.UInt32 ? TypeSymbol.Int64 : op == "-" && type.Kind is TypeKind.UInt64 or TypeKind.NativeUInt ? TypeSymbol.Unsupported : type;
}
