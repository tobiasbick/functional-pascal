namespace Fpas.Compiler.Binding.Types;

internal static class TypeFacts
{
    internal static bool PredefinedName(string name) => name is "bool" or "byte" or "sbyte" or "short" or "ushort" or
        "int" or "uint" or "long" or "ulong" or "nint" or "nuint" or "float" or "double" or "decimal" or "char" or "string" or "object";

    internal static TypeSymbol Primitive(string name) => name switch
    {
        "string" or "System.String" => TypeSymbol.String,
        "int" or "System.Int32" => TypeSymbol.Int32,
        "bool" or "System.Boolean" => TypeSymbol.Boolean,
        "double" or "System.Double" => TypeSymbol.Double,
        "char" or "System.Char" => TypeSymbol.Char,
        "object" or "System.Object" => TypeSymbol.Object,
        "sbyte" or "System.SByte" => TypeSymbol.SByte, "byte" or "System.Byte" => TypeSymbol.Byte,
        "short" or "System.Int16" => TypeSymbol.Int16, "ushort" or "System.UInt16" => TypeSymbol.UInt16,
        "uint" or "System.UInt32" => TypeSymbol.UInt32, "long" or "System.Int64" => TypeSymbol.Int64,
        "ulong" or "System.UInt64" => TypeSymbol.UInt64, "nint" or "System.IntPtr" => TypeSymbol.NativeInt,
        "nuint" or "System.UIntPtr" => TypeSymbol.NativeUInt, "float" or "System.Single" => TypeSymbol.Single,
        "decimal" or "System.Decimal" => TypeSymbol.Decimal,
        _ => TypeSymbol.Unsupported
    };
    internal static bool Numeric(TypeSymbol type) => !type.IsNullable && NumericFacts.NumericKind(type.Kind);
    internal static string? Alias(TypeSymbol type) => type.Kind switch
    {
        TypeKind.SByte => "sbyte", TypeKind.Byte => "byte", TypeKind.Int16 => "short", TypeKind.UInt16 => "ushort", TypeKind.UInt32 => "uint",
        TypeKind.Int64 => "long", TypeKind.UInt64 => "ulong", TypeKind.NativeInt => "nint", TypeKind.NativeUInt => "nuint", TypeKind.Single => "float", TypeKind.Decimal => "decimal", _ => null
    };
    internal static string? ClrName(TypeSymbol type) => type.Kind switch
    {
        TypeKind.SByte => "System.SByte", TypeKind.Byte => "System.Byte", TypeKind.Int16 => "System.Int16", TypeKind.UInt16 => "System.UInt16",
        TypeKind.UInt32 => "System.UInt32", TypeKind.Int64 => "System.Int64", TypeKind.UInt64 => "System.UInt64", TypeKind.NativeInt => "System.IntPtr",
        TypeKind.NativeUInt => "System.UIntPtr", TypeKind.Single => "System.Single", TypeKind.Decimal => "System.Decimal", _ => null
    };
    internal static string Display(TypeSymbol type) => type.ToString();
}
