using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class Binder
{
    private BoundExpression DomainNew(ScalarInfo info, NewSyntax syntax, bool check)
    {
        if (syntax.Type.Nullable) throw source.Error("FP3005", "Cannot construct a nullable domain directly.", syntax.Position);
        if (syntax.Arguments.Count == 0 && info.Type.IsValue) return new BoundNew(info.Type, null, []);
        if (syntax.Arguments.Count != 1) throw source.Error("FP3031", "A domain constructor requires one underlying value.", syntax.Position);
        var argument = syntax.Arguments[0];
        if (argument is NamedArgumentSyntax named)
        {
            if (named.Name.Text != "Value") throw source.Error("FP3031", "The domain constructor parameter is Value.", named.Name.Position);
            argument = named.Value;
        }
        var value = Convert(BindExpression(argument, check, info.Base), info.Base, argument.Position);
        ValidateDomainConstant(info, value, argument.Position);
        return new BoundNew(info.Type, ReferenceCatalog.DomainConstructor(info), [value]);
    }

    private void ValidateDomainConstant(ScalarInfo info, BoundExpression value, int position)
    {
        if (info.Category != ScalarCategory.Range || !ConstantValue.IsConstant(value)) return;
        try
        {
            var constant = NumericValue.Integer(ConstantValue.Evaluate(value)!);
            if (constant < info.Lower || constant > info.Upper) throw source.Error("FP3031", "The constant is outside the declared subrange.", position);
        }
        catch (Exception error) when (error is OverflowException or DivideByZeroException)
        { throw source.Error("FP3031", "The constant cannot be represented in the declared subrange.", position); }
    }

    private BoundExpression? ScalarConversion(BoundExpression value, TypeSymbol target, int position, bool explicitCast, bool check)
    {
        if (value.Type.NonNullable == target.NonNullable || value.Type == TypeSymbol.Null) return null;
        var from = references.ScalarInfo(value.Type); var to = references.ScalarInfo(target);
        if (explicitCast && from?.Category == ScalarCategory.Enum && TypeFacts.Numeric(target.NonNullable))
            return Convert(new BoundConversion(value, from.Base with { IsNullable = value.Type.IsNullable }, check), target, position, explicitCast: true, check: check);
        if (explicitCast && to?.Category == ScalarCategory.Enum && TypeFacts.Numeric(value.Type.NonNullable))
            return new BoundConversion(Convert(value, to.Base with { IsNullable = target.IsNullable }, position, explicitCast: true, check: check), target, check);
        if (from is { Category: not ScalarCategory.Enum } &&
            (from.Category == ScalarCategory.Range && TypeFacts.Numeric(target.NonNullable) || explicitCast && target.NonNullable == from.Base))
        {
            if (value.Type.NullableValue)
                return explicitCast || target.NullableValue && references.CanAssign(from.Base, target.NonNullable) ? new BoundConversion(value, target, check) : null;
            if (value.Type.IsReference && value.Type.NeedsNullCheck) throw source.Error("FP3013", "Validate a nullable domain before reading its payload.", position);
            return Convert(new BoundCall(ReferenceCatalog.DomainGetter(from), [], value), target, position, explicitCast: explicitCast, check: check);
        }
        if (explicitCast && to is { Category: not ScalarCategory.Enum } && TypeFacts.Numeric(value.Type.NonNullable) &&
            (references.CanAssign(value.Type.NonNullable, to.Base) || ConstantFits(value, to.Base)))
        {
            if (value.Type.NullableValue) return new BoundConversion(value, target, check);
            var underlying = Convert(value, to.Base, position);
            ValidateDomainConstant(to, underlying, position);
            var wrapped = new BoundNew(to.Type, ReferenceCatalog.DomainConstructor(to), [underlying]);
            return target.NullableValue ? new BoundConversion(wrapped, target, check) : wrapped;
        }
        if (explicitCast && to is { Category: ScalarCategory.Distinct } && value.Type.NonNullable == to.Base)
        {
            if (value.Type.NullableValue) return new BoundConversion(value, target, check);
            if (value.Type.IsReference && value.Type.NeedsNullCheck) throw source.Error("FP3013", "Validate a nullable string before constructing its domain.", position);
            var wrapped = new BoundNew(to.Type, ReferenceCatalog.DomainConstructor(to), [value]);
            return target.IsNullable ? new BoundConversion(wrapped, target, check) : wrapped;
        }
        return null;
    }

    private BoundExpression? ScalarBinary(BoundExpression left, string op, BoundExpression right, int position, bool check)
    {
        var info = references.ScalarInfo(left.Type);
        if (info is null || left.Type.NullableValue || right.Type.NullableValue) return null;
        if (left.Type.NonNullable == right.Type.NonNullable)
        {
            if (op is "=" or "<>")
            {
                if (info.Category == ScalarCategory.Enum)
                    return NumericBinary(new BoundConversion(left, info.Base, false), op, new BoundConversion(right, info.Base, false), position, check);
                var parameter = info.Type.IsReference ? info.Type with { IsNullable = true } : info.Type;
                return new BoundCall(new(info.Type, op == "=" ? "op_Equality" : "op_Inequality", TypeSymbol.Boolean, [parameter, parameter], true), [left, right]);
            }
            if (info.Category == ScalarCategory.Enum && op is "and" or "or" or "xor")
                return new BoundConversion(NumericBinary(new BoundConversion(left, info.Base, false), op, new BoundConversion(right, info.Base, false), position, false), info.Type, false);
            if (info.Category != ScalarCategory.Enum && op is "<" or ">" or "<=" or ">=")
            {
                if (info.Type.IsReference && (left.Type.NeedsNullCheck || right.Type.NeedsNullCheck))
                    throw source.Error("FP3013", "Validate nullable domains before comparing their payloads.", position);
                var a = new BoundCall(ReferenceCatalog.DomainGetter(info), [], left); var b = new BoundCall(ReferenceCatalog.DomainGetter(info), [], right);
                if (info.Base == TypeSymbol.String)
                    return NumericBinary(new BoundCall(references.FindMethod(TypeSymbol.String, "CompareOrdinal", [TypeSymbol.String, TypeSymbol.String], position, source), [a, b]), op, new BoundInteger(0), position, check);
                if (TypeFacts.Numeric(info.Base)) return NumericBinary(a, op, b, position, check);
            }
        }
        if (info.Category == ScalarCategory.Range)
        {
            var a = new BoundCall(ReferenceCatalog.DomainGetter(info), [], left);
            var other = references.ScalarInfo(right.Type);
            var b = other?.Category == ScalarCategory.Range ? new BoundCall(ReferenceCatalog.DomainGetter(other), [], right) : right;
            if (TypeFacts.Numeric(b.Type)) return NumericBinary(a, op, b, position, check);
        }
        return null;
    }
}
