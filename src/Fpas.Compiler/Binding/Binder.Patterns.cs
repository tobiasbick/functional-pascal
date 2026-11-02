using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class Binder
{
    private MatchPattern BindPattern(PatternSyntax syntax, TypeSymbol type, Dictionary<string, LocalSymbol> bindings, HashSet<string> names)
    {
        LocalSymbol Binding(Token name, TypeSymbol annotation)
        {
            if (!names.Add(name.Text)) throw source.Error("FP3024", "A pattern cannot bind the same name twice.", name.Position);
            if (bindings.TryGetValue(name.Text, out var existing))
            {
                if (existing.Type != annotation) throw source.Error("FP3024", "Alternative patterns must bind identical names and types.", name.Position);
                return existing;
            }
            var local = Allocate(name.Text, annotation); Declare(local, name.Position); bindings.Add(name.Text, local); return local;
        }
        switch (syntax)
        {
            case AnyPatternSyntax: return new AnyMatch();
            case BindingPatternSyntax bound:
                var annotation = BindType(bound.Type);
                if (annotation != type) throw source.Error("FP3024", "A pattern annotation must match the selected static type exactly.", bound.Type.Position);
                return new AnyMatch(Binding(bound.Name, annotation));
            case TypePatternSyntax tested:
                var target = PatternType(tested.Type);
                var narrowed = Binding(tested.Name, target);
                if (target.CanHaveNull) nonNull.Add(narrowed);
                return new TypeMatch(target, narrowed);
            case ConstantPatternSyntax constant: return BindConstantPattern(constant.Value, type);
            case RangePatternSyntax range:
                if (type.NonNullable.Kind is not (TypeKind.Int32 or TypeKind.Char)) throw source.Error("FP3024", "Range patterns currently require int or char selectors.", range.Position);
                var lower = BindConstantPattern(range.Lower, type); var upper = BindConstantPattern(range.Upper, type);
                var low = System.Convert.ToInt64(lower.Value, System.Globalization.CultureInfo.InvariantCulture);
                var high = System.Convert.ToInt64(upper.Value, System.Globalization.CultureInfo.InvariantCulture);
                if (lower.Value is null || upper.Value is null) throw source.Error("FP3024", "Range bounds must be non-null integral constants.", range.Position);
                if (low > high) throw source.Error("FP3024", "A pattern range cannot be reversed.", range.Position);
                return new RangeMatch(low, high);
            case NamedPatternSyntax named:
                ExpressionSyntax path = new NameExpressionSyntax(named.Name.Parts[0]);
                foreach (var part in named.Name.Parts.Skip(1)) path = new MemberSyntax(path, part);
                var selected = PatternOwner(path, type);
                var info = references.UnionInfo(selected);
                if (info is null)
                {
                    if (named.Fields is not null) throw source.Error("FP3024", "Payload patterns require a declared union case.", named.Position);
                    return BindConstantPattern(path, type);
                }
                var item = info.Cases.SingleOrDefault(item => item.Name == named.Name.Parts[^1].Text)
                    ?? throw source.Error("FP3024", "Unknown union case in pattern.", named.Position);
                if (!references.CanAssign(item.Type, type.NonNullable)) throw source.Error("FP3024", "The union case cannot match this selector type.", named.Position);
                if (item.Fields.Count == 0 ? named.Fields is not null : named.Fields is null || named.Fields.Count != item.Fields.Count)
                    throw source.Error("FP3024", "A pattern must name every payload, using _ for ignored values; fieldless cases have no parentheses.", named.Position);
                var fields = new MatchPattern?[item.Fields.Count];
                var isNamed = named.Fields?.FirstOrDefault()?.Name is not null;
                for (var index = 0; index < (named.Fields?.Count ?? 0); index++)
                {
                    var field = named.Fields![index];
                    if ((field.Name is not null) != isNamed) throw source.Error("FP3024", "Do not mix named and positional payload patterns.", field.Pattern.Position);
                    var slot = field.Name is null ? index : item.Fields.ToList().FindIndex(candidate => candidate.Name == field.Name.Text);
                    if (slot < 0 || fields[slot] is not null) throw source.Error("FP3024", "Unknown or duplicate payload name.", field.Pattern.Position);
                    fields[slot] = BindPattern(field.Pattern, item.Fields[slot].Type, bindings, names);
                }
                return new CaseMatch(item, fields.Select(field => field!).ToArray());
            default: throw new InvalidOperationException("Unrecognized pattern syntax.");
        }
    }

    private ConstantMatch BindConstantPattern(ExpressionSyntax syntax, TypeSymbol selector)
    {
        var expression = BindExpression(syntax);
        object? value;
        try { value = PatternConstant(expression); }
        catch (OverflowException) { throw source.Error("FP3024", "The pattern constant is outside its type's range.", syntax.Position); }
        catch (InvalidOperationException) { throw source.Error("FP3024", "A literal or CLR metadata constant is required in a constant pattern.", syntax.Position); }
        if (value is double number && double.IsNaN(number)) throw source.Error("FP3024", "NaN never matches a numeric constant pattern; use a guard or fallback.", syntax.Position);
        if (value is null) return new(new BoundNull(), null);
        expression = expression.Type.Kind switch
        {
            TypeKind.Int32 => new BoundInteger((int)value), TypeKind.Double => new BoundDouble((double)value),
            TypeKind.Char => new BoundCharacter((char)value), TypeKind.String => new BoundString((string)value),
            TypeKind.Boolean => new BoundBoolean((bool)value), _ => expression
        };
        if (!references.CanAssign(expression.Type, selector.NonNullable)) throw source.Error("FP3024", "The constant cannot match this selector type.", syntax.Position);
        if (selector.NonNullable == TypeSymbol.Double && expression.Type != TypeSymbol.Double)
        {
            value = System.Convert.ToDouble(value, System.Globalization.CultureInfo.InvariantCulture);
            expression = new BoundDouble((double)value);
        }
        if (selector.NonNullable == TypeSymbol.Int32 && expression.Type == TypeSymbol.Char) { value = (int)(char)value!; expression = new BoundInteger((int)value); }
        return new(expression, value);
    }

    private static object? PatternConstant(BoundExpression expression) => expression switch
    {
        BoundNull => null, BoundBoolean boolean => boolean.Value, BoundInteger integer => integer.Value,
        BoundDouble real => real.Value, BoundCharacter character => character.Value, BoundString text => text.Value,
        BoundField { Field.IsLiteral: true } field => field.Field.Constant,
        BoundUnary { Operator: "+" } unary => PatternConstant(unary.Operand),
        BoundUnary { Operator: "-", Type.Kind: TypeKind.Int32 } unary => checked(-System.Convert.ToInt32(PatternConstant(unary.Operand), System.Globalization.CultureInfo.InvariantCulture)),
        BoundUnary { Operator: "-", Type.Kind: TypeKind.Double } unary => -System.Convert.ToDouble(PatternConstant(unary.Operand), System.Globalization.CultureInfo.InvariantCulture),
        BoundConversion { Target.Kind: TypeKind.Int32, Operand.Type.Kind: TypeKind.Char } conversion => (int)(char)PatternConstant(conversion.Operand)!,
        _ => throw new InvalidOperationException("Not a pattern constant.")
    };

    private TypeSymbol PatternType(TypeSyntax syntax)
    {
        var type = BindType(syntax);
        if (type.IsNullable) throw source.Error("FP3024", "A CLR type test requires a non-nullable target type.", syntax.Position);
        return type;
    }

    private BoundExpression BindTypeTest(TypeTestSyntax syntax) => new BoundTypeTest(BindExpression(syntax.Value), PatternType(syntax.Target));
}
