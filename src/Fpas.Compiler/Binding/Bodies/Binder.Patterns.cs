using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

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
                var rangeType = references.ScalarInfo(type) is { Category: ScalarCategory.Range } domain ? domain.Base : type.NonNullable;
                if (!NumericFacts.Integral(rangeType)) throw source.Error("FP3024", "Range patterns require an integral or subrange selector.", range.Position);
                var lower = BindConstantPattern(range.Lower, type); var upper = BindConstantPattern(range.Upper, type);
                if (lower.Value is null || upper.Value is null) throw source.Error("FP3024", "Range bounds must be non-null integral constants.", range.Position);
                var low = NumericValue.Integer(lower.Value); var high = NumericValue.Integer(upper.Value);
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
                catalog.Analysis?.ResolvedMethod(source, named.Name.Parts[^1], item.Factory);
                var fields = new MatchPattern?[item.Fields.Count];
                var isNamed = named.Fields?.FirstOrDefault()?.Name is not null;
                for (var index = 0; index < (named.Fields?.Count ?? 0); index++)
                {
                    var field = named.Fields![index];
                    if ((field.Name is not null) != isNamed) throw source.Error("FP3024", "Do not mix named and positional payload patterns.", field.Pattern.Position);
                    var slot = field.Name is null ? index : item.Fields.ToList().FindIndex(candidate => candidate.Name == field.Name.Text);
                    if (slot < 0 || fields[slot] is not null) throw source.Error("FP3024", "Unknown or duplicate payload name.", field.Pattern.Position);
                    if (field.Name is { } label) catalog.Analysis?.NamedProperty(source, label, item.Fields[slot].Getter, "argument");
                    fields[slot] = BindPattern(field.Pattern, item.Fields[slot].Type, bindings, names);
                }
                return new CaseMatch(item, fields.Select(field => field!).ToArray());
            default: throw new InvalidOperationException("Unrecognized pattern syntax.");
        }
    }

    private ConstantMatch BindConstantPattern(ExpressionSyntax syntax, TypeSymbol selector)
    {
        var expression = BindExpression(syntax);
        var domain = references.ScalarInfo(selector);
        var target = domain?.Category == ScalarCategory.Range ? domain.Base : selector.NonNullable;
        if (TypeFacts.Numeric(target) && (references.CanAssign(expression.Type, target) || ConstantFits(expression, target)))
            expression = Convert(expression, target, syntax.Position);
        object? value;
        try { value = ConstantValue.Evaluate(expression); }
        catch (OverflowException) { throw source.Error("FP3024", "The pattern constant is outside its type's range.", syntax.Position); }
        catch (InvalidOperationException) { throw source.Error("FP3024", "A literal or CLR metadata constant is required in a constant pattern.", syntax.Position); }
        if (value is double number && double.IsNaN(number) || value is float single && float.IsNaN(single))
            throw source.Error("FP3024", "NaN never matches a numeric constant pattern; use a guard or fallback.", syntax.Position);
        if (value is null) return new(new BoundNull(), null);
        if (!references.CanAssign(expression.Type, target)) throw source.Error("FP3024", "The constant cannot match this selector type.", syntax.Position);
        if (domain?.Category == ScalarCategory.Range && (NumericValue.Integer(value) < domain.Lower || NumericValue.Integer(value) > domain.Upper))
            throw source.Error("FP3024", "The pattern is outside the declared subrange.", syntax.Position);
        if (TypeFacts.Numeric(expression.Type)) expression = BoundNumber.Create(expression.Type, value);
        return new(expression, value);
    }

    private TypeSymbol PatternType(TypeSyntax syntax)
    {
        var type = BindType(syntax);
        if (type.IsNullable) throw source.Error("FP3024", "A CLR type test requires a non-nullable target type.", syntax.Position);
        return type;
    }

    private BoundExpression BindTypeTest(TypeTestSyntax syntax) => new BoundTypeTest(BindExpression(syntax.Value), PatternType(syntax.Target));
}
