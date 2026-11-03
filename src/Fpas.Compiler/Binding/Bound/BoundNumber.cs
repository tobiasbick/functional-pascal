using Fpas.Compiler.Binding.Types;
namespace Fpas.Compiler.Binding.Bound;

internal sealed record BoundNumber(TypeSymbol NumberType, object Value) : BoundExpression(NumberType)
{
    internal static BoundExpression Create(TypeSymbol type, object value) => type.Kind switch
    {
        TypeKind.Int32 => new BoundInteger((int)value), TypeKind.Double => new BoundDouble((double)value), TypeKind.Char => new BoundCharacter((char)value),
        TypeKind.NativeInt or TypeKind.NativeUInt => new BoundNumber(type, NumericValue.ConvertTo(value, type, true)),
        _ => new BoundNumber(type, value)
    };
}
