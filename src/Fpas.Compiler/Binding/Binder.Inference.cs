using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class Binder
{
    private OpenUnionInfo? OpenUnionTarget(ImportTarget? target) => target is UnionImport or ClrImport
        ? references.OpenUnion(ImportedValueType(target, unit.Syntax.Name.Position, allowOpen: true)) : null;

    private bool NeedsInferenceContext(ExpressionSyntax expression)
    {
        if (expression is ParenthesizedSyntax wrapped) return NeedsInferenceContext(wrapped.Expression);
        if (expression is ArrayLiteralSyntax array) return array.Elements.Count == 0 || array.Elements.All(NeedsInferenceContext);
        if (expression is MemberSyntax member && OpenUnionTarget(StaticTarget(member.Receiver, allowOpenGeneric: true)) is { } open)
            return open.Info.Cases.Any(item => item.Name == member.Name.Text && item.Fields.Count == 0);
        if (expression is CallSyntax { Target: MemberSyntax factory } call && OpenUnionTarget(StaticTarget(factory.Receiver, allowOpenGeneric: true)) is { } union)
        {
            var item = union.Info.Cases.FirstOrDefault(item => item.Name == factory.Name.Text);
            return item is not null && (call.Arguments.All(argument => argument is LiteralSyntax { Token.Text: "null" } || NeedsInferenceContext(argument)) ||
                union.Variables.Any(variable => !item.Fields.Any(field => new TypeInference(references, [variable]).ContainsVariable(field.Type))));
        }
        if (expression is ConditionalSyntax conditional) return conditional.Arms.All(arm => NeedsInferenceContext(arm.Value)) && NeedsInferenceContext(conditional.Else);
        return false;
    }

    private BoundExpression?[] BindInferenceValues(IReadOnlyList<ExpressionSyntax> syntax, bool check, IReadOnlyList<TypeSymbol>? parameters = null, TypeInference? inference = null)
    {
        var deferred = syntax.Select(NeedsInferenceContext).ToArray();
        if (syntax.Where((_, index) => deferred[index]).Any(MayInvoke))
        {
            ForgetCapturedFacts();
            var assigned = syntax.SelectMany(AssignedInLambdas).ToHashSet(StringComparer.Ordinal);
            nonNull.RemoveWhere(symbol => symbol is LocalSymbol local && assigned.Contains(local.Name) || symbol is CaptureSymbol { Original: LocalSymbol original } && assigned.Contains(original.Name));
        }
        var values = new BoundExpression?[syntax.Count];
        for (var index = 0; index < values.Length; index++)
        {
            if (deferred[index]) continue;
            var expected = parameters is not null && (inference is null || !inference.ContainsVariable(parameters[index])) ? parameters[index] : null;
            values[index] = BindExpression(syntax[index], check, expected);
        }
        return values;
    }

    private BoundExpression[] BindDeferredArguments(IReadOnlyList<ExpressionSyntax> syntax, IReadOnlyList<BoundExpression?> values, IReadOnlyList<TypeSymbol> parameters, bool check)
        => syntax.Select((argument, index) => values[index] ?? BindExpression(argument, check, parameters[index])).ToArray();

    private bool AcceptsDeferred(ExpressionSyntax syntax, TypeSymbol target)
    {
        if (syntax is ParenthesizedSyntax wrapped) return AcceptsDeferred(wrapped.Expression, target);
        if (syntax is ArrayLiteralSyntax) return target.Kind == TypeKind.Array && target.Rank == 1 || ReferenceCatalog.ArrayInterface(target);
        if (syntax is ConditionalSyntax conditional) return conditional.Arms.All(arm => AcceptsDeferred(arm.Value, target)) && AcceptsDeferred(conditional.Else, target);
        var member = syntax is CallSyntax { Target: MemberSyntax factory } ? factory : syntax as MemberSyntax;
        if (member is null) return false;
        var open = OpenUnionTarget(StaticTarget(member.Receiver, allowOpenGeneric: true));
        return open is not null && SameUnionOwner(open.Info.Type, target);
    }

    private static bool SameUnionOwner(TypeSymbol owner, TypeSymbol expected)
        => owner.Union is not null ? owner.Union == expected.Union : owner.Definition is not null && owner.Definition == expected.Definition;
}
