namespace Fpas.Compiler.Binding;

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
        _ => TypeSymbol.Unsupported
    };
    internal static bool Numeric(TypeSymbol type) => !type.IsNullable && type.Kind is TypeKind.Int32 or TypeKind.Double or TypeKind.Char;
    internal static string Display(TypeSymbol type) => type.ToString();
}
