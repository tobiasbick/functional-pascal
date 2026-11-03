using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Bound;
namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private AttributeValue AttributeConstant(BoundExpression expression, TypeSymbol target, int position)
    {
        var converted = Convert(expression, target, position);
        return AttributeValue(converted, target, position);
    }

    private AttributeValue AttributeValue(BoundExpression value, TypeSymbol target, int position)
    {
        if (!references.AttributeType(target)) throw source.Error("FP3035", "The value has no legal CLR custom-attribute representation.", position);
        if (value is BoundNull or BoundConversion { Operand: BoundNull })
            return new(target.NonNullable == TypeSymbol.Object ? TypeSymbol.String : target, null);
        if (value is BoundConversion conversion && (target.NonNullable == TypeSymbol.Object || conversion.Operand.Type.NonNullable == target.NonNullable))
            return AttributeValue(conversion.Operand, target.NonNullable == TypeSymbol.Object ? conversion.Operand.Type : target, position);
        if (value is BoundConversion { Operand: BoundArrayLiteral literal } && target.Kind == TypeKind.Array)
            return AttributeValue(literal, target, position);
        if (target.NonNullable == TypeSymbol.Object)
        {
            if (value.Type.NonNullable == TypeSymbol.Object) throw source.Error("FP3035", "A boxed attribute value must have a constant CLR payload type.", position);
            return AttributeValue(value, value.Type, position);
        }
        if (value is BoundArrayLiteral array)
            return new(target, array.Elements.Select(element => AttributeConstant(element, target.Element!, position)).ToArray());
        if (value is BoundTypeOf typeOf && target.Definition?.FullName == "System.Type")
        {
            if (ContainsAttributeParameter(typeOf.Operand)) throw source.Error("FP3035", "Attribute typeof arguments cannot contain generic parameters.", position);
            return new(target, typeOf.Operand);
        }
        if (target.Kind == TypeKind.Array || target.Definition?.FullName == "System.Type" || !ConstantValue.IsConstant(value))
            throw source.Error("FP3035", "Attribute values require CLR constants, typeof of a closed type, or array literals of those values.", position);
        try
        {
            var constant = ConstantValue.Evaluate(value);
            if (constant is not null && references.ScalarInfo(target) is { Category: Syntax.ScalarCategory.Enum } enumInfo)
                constant = NumericValue.ConvertTo(constant, enumInfo.Base, true);
            return new(target, constant);
        }
        catch (Exception failure) when (failure is InvalidOperationException or OverflowException or DivideByZeroException)
        { throw source.Error("FP3035", "The attribute constant cannot be evaluated or represented.", position); }
    }
}
