using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private OpenVariantInfo? OpenVariantTarget(ImportTarget? target) => target is VariantImport or ClrImport
        ? references.OpenVariant(ImportedValueType(target, unit.Syntax.Name.Position, allowOpen: true)) : null;

    private bool NeedsInferenceContext(ExpressionSyntax expression)
    {
        if (expression is ParenthesizedSyntax wrapped) return NeedsInferenceContext(wrapped.Expression);
        if (expression is ArrayLiteralSyntax array) return array.Elements.Count == 0 || array.Elements.All(NeedsInferenceContext);
        if (expression is MemberSyntax member && OpenVariantTarget(StaticTarget(member.Receiver, allowOpenGeneric: true)) is { } open)
            return open.Info.Cases.Any(item => item.Name == member.Name.Text && item.Fields.Count == 0);
        if (expression is CallSyntax { Target: MemberSyntax factory } call && OpenVariantTarget(StaticTarget(factory.Receiver, allowOpenGeneric: true)) is { } variant)
        {
            var item = variant.Info.Cases.FirstOrDefault(item => item.Name == factory.Name.Text);
            return item is not null && (call.Arguments.All(argument => argument is LiteralSyntax { Token.Text: "null" } || NeedsInferenceContext(argument)) ||
                variant.Variables.Any(variable => !item.Fields.Any(field => new TypeInference(references, [variable]).ContainsVariable(field.Type))));
        }
        if (expression is ConditionalSyntax conditional) return conditional.Arms.All(arm => NeedsInferenceContext(arm.Value)) && NeedsInferenceContext(conditional.Else);
        return false;
    }

    private BoundExpression?[] BindInferenceValues(IReadOnlyList<ExpressionSyntax> syntax, bool check, out HashSet<object>[] facts,
        IReadOnlyList<TypeSymbol>? parameters = null, TypeInference? inference = null)
    {
        var deferred = syntax.Select(NeedsInferenceContext).ToArray();
        var values = new BoundExpression?[syntax.Count];
        facts = new HashSet<object>[syntax.Count];
        for (var index = 0; index < values.Length; index++)
        {
            // Contextual binding must use the facts at this argument's runtime position.
            facts[index] = nonNull.ToHashSet();
            if (deferred[index]) { InvalidateEffects(syntax[index]); continue; }
            var expected = parameters is not null && (inference is null || !inference.ContainsVariable(parameters[index])) ? parameters[index] : null;
            values[index] = BindExpression(syntax[index], check, expected);
        }
        return values;
    }

    private BoundExpression[] BindDeferredArguments(IReadOnlyList<ExpressionSyntax> syntax, IReadOnlyList<BoundExpression?> values,
        IReadOnlyList<TypeSymbol> parameters, bool check, IReadOnlyList<HashSet<object>>? facts = null)
    {
        var after = nonNull.ToHashSet();
        var invalidated = new HashSet<object>();
        var result = new BoundExpression[syntax.Count];
        for (var index = 0; index < result.Length; index++)
        {
            if (values[index] is { } value) { result[index] = value; continue; }
            var before = (facts?[index] ?? nonNull).Except(invalidated).ToHashSet();
            SetFacts(before);
            result[index] = BindExpression(syntax[index], check, parameters[index]);
            invalidated.UnionWith(before.Except(nonNull));
        }
        // Delayed binding can discover captures that invalidate later flow facts.
        after.ExceptWith(invalidated);
        SetFacts(after);
        return result;
    }

    private bool AcceptsDeferred(ExpressionSyntax syntax, TypeSymbol target)
    {
        if (syntax is ParenthesizedSyntax wrapped) return AcceptsDeferred(wrapped.Expression, target);
        if (syntax is ArrayLiteralSyntax) return target.Kind == TypeKind.Array && target.Rank == 1 || ReferenceCatalog.ArrayInterface(target);
        if (syntax is ConditionalSyntax conditional) return conditional.Arms.All(arm => AcceptsDeferred(arm.Value, target)) && AcceptsDeferred(conditional.Else, target);
        var member = syntax is CallSyntax { Target: MemberSyntax factory } ? factory : syntax as MemberSyntax;
        if (member is null) return false;
        var open = OpenVariantTarget(StaticTarget(member.Receiver, allowOpenGeneric: true));
        return open is not null && SameVariantOwner(open.Info.Type, target);
    }

    private static bool SameVariantOwner(TypeSymbol owner, TypeSymbol expected)
        => owner.Variant is not null ? owner.Variant == expected.Variant : owner.Definition is not null && owner.Definition == expected.Definition;
}
