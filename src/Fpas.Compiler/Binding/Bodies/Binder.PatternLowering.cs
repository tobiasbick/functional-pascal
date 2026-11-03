using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Bound;
namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private static BoundExpression Both(BoundExpression left, BoundExpression right) => left is BoundBoolean { Value: true } ? right :
        right is BoundBoolean { Value: true } ? left : new BoundBinary(left, "and", right, TypeSymbol.Boolean, false);

    private BoundExpression LowerPattern(MatchPattern pattern, BoundExpression value)
    {
        switch (pattern)
        {
            case AnyMatch any:
                return any.Binding is null ? new BoundBoolean(true) : new BoundSequence([new BoundBinding(any.Binding, new BoundConversion(value, any.Binding.Type, false))], new BoundBoolean(true));
            case TypeMatch tested:
                return Both(new BoundTypeTest(value, tested.Target), new BoundSequence([new BoundBinding(tested.Binding,
                    new BoundConversion(value, tested.Target, false))], new BoundBoolean(true)));
            case ConstantMatch { Value: null }: return new BoundNullTest(value, true);
            case ConstantMatch constant:
                BoundExpression condition = new BoundBoolean(true);
                if (value.Type.NullableValue)
                {
                    condition = new BoundNullTest(value, false);
                    value = new BoundConversion(value, value.Type.NonNullable, false);
                }
                if (references.ScalarInfo(value.Type) is { Category: Syntax.ScalarCategory.Range } domain)
                    value = new BoundCall(ReferenceCatalog.DomainGetter(domain), [], value);
                if (value.Type.NonNullable != constant.Expression.Type.NonNullable)
                {
                    condition = Both(condition, new BoundTypeTest(value, constant.Expression.Type.NonNullable));
                    value = new BoundConversion(value, constant.Expression.Type.NonNullable, false);
                }
                var equals = constant.Expression.Type == TypeSymbol.String
                    ? (BoundExpression)new BoundCall(references.FindMethod(TypeSymbol.String, "op_Equality", [value.Type, constant.Expression.Type], 0, source), [value, constant.Expression])
                    : TypeFacts.Numeric(value.Type) ? NumericBinary(value, "=", constant.Expression, 0, false)
                    : new BoundBinary(value, "=", constant.Expression, TypeSymbol.Boolean, false);
                return Both(condition, equals);
            case RangeMatch range:
                var present = value.Type.NullableValue ? (BoundExpression)new BoundNullTest(value, false) : new BoundBoolean(true);
                if (value.Type.NullableValue) value = new BoundConversion(value, value.Type.NonNullable, false);
                if (references.ScalarInfo(value.Type) is { Category: Syntax.ScalarCategory.Range } rangeDomain)
                    value = new BoundCall(ReferenceCatalog.DomainGetter(rangeDomain), [], value);
                return Both(present, Both(NumericBinary(value, ">=", BoundNumber.Create(value.Type, NumericValue.ConvertTo(range.Lower, value.Type, true)), 0, false),
                    NumericBinary(value, "<=", BoundNumber.Create(value.Type, NumericValue.ConvertTo(range.Upper, value.Type, true)), 0, false)));
            case CaseMatch item:
                var temporary = Allocate("<case-value>", item.Case.Type with { IsNullable = true });
                BoundExpression test = new BoundNullTest(new BoundLocal(temporary), false);
                for (var index = 0; index < item.Fields.Count; index++)
                {
                    if (item.Fields[index] is AnyMatch { Binding: null }) continue;
                    var field = item.Case.Fields[index];
                    var payload = Allocate("<case-payload>", field.Type);
                    var read = new BoundCall(field.Getter, [], new BoundLocal(temporary, item.Case.Type));
                    test = Both(test, new BoundSequence([new BoundBinding(payload, read)], LowerPattern(item.Fields[index], new BoundLocal(payload))));
                }
                return new BoundSequence([new BoundBinding(temporary, new BoundSafeCast(value, temporary.Type))], test);
            default: throw new InvalidOperationException("Unrecognized bound pattern.");
        }
    }
}
