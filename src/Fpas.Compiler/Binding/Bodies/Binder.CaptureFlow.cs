using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private bool MayInvoke(ExpressionSyntax expression)
        => expression is CallSyntax or NewSyntax or MemberSyntax or IndexSyntax or WithSyntax or CaseExpressionSyntax ||
            operatorCalls.ContainsKey(expression) || expression.Children().Any(MayInvoke);

    private static IEnumerable<string> AssignedInLambdas(ExpressionSyntax expression)
    {
        if (expression is LambdaSyntax lambda)
        {
            foreach (var name in LambdaWrites(lambda, [])) yield return name;
            yield break;
        }
        foreach (var child in expression.Children())
            foreach (var name in AssignedInLambdas(child)) yield return name;
    }

    private static IEnumerable<string> LambdaWrites(LambdaSyntax lambda, HashSet<string> declared)
    {
        var locals = declared.Concat(lambda.Parameters.Select(parameter => parameter.Name.Text)).ToHashSet(StringComparer.Ordinal);
        if (lambda.Body is { } body)
            foreach (var name in StatementWrites(body, locals)) yield return name;
        if (lambda.Expression is { } expression)
            foreach (var name in ExpressionWrites(expression, locals)) yield return name;
    }

    private static IEnumerable<string> ExpressionWrites(ExpressionSyntax expression, HashSet<string> declared)
    {
        if (expression is LambdaSyntax lambda)
        {
            foreach (var name in LambdaWrites(lambda, declared)) yield return name;
            yield break;
        }
        if (expression is ArgumentModeSyntax { Mode: "ref" or "out", Value: NameExpressionSyntax target } && !declared.Contains(target.Name.Text))
            yield return target.Name.Text;
        foreach (var child in expression.Children())
            foreach (var name in ExpressionWrites(child, declared)) yield return name;
    }

    private static IEnumerable<string> StatementWrites(IReadOnlyList<StatementSyntax> statements, HashSet<string> declared)
    {
        var locals = declared.ToHashSet(StringComparer.Ordinal);
        IEnumerable<string> Block(IReadOnlyList<StatementSyntax> body, params string[] names)
            => StatementWrites(body, locals.Concat(names).ToHashSet(StringComparer.Ordinal));
        foreach (var statement in statements)
        {
            if (statement is AssignmentSyntax { Target: NameExpressionSyntax target } && !locals.Contains(target.Name.Text)) yield return target.Name.Text;
            IEnumerable<ExpressionSyntax> expressions = statement switch
            {
                TrySyntax => [], CaseStatementSyntax match => new[] { match.Selector }, _ => Expressions(statement)
            };
            foreach (var expression in expressions)
                foreach (var name in ExpressionWrites(expression, locals)) yield return name;
            switch (statement)
            {
                case BindingSyntax binding: locals.Add(binding.Name.Text); break;
                case UsingSyntax resource: locals.Add(resource.Binding.Name.Text); break;
                case BlockSyntax block:
                    foreach (var name in Block(block.Statements)) yield return name;
                    break;
                case IfSyntax branch:
                    foreach (var arm in branch.Arms)
                        foreach (var name in Block(arm.Statements)) yield return name;
                    foreach (var name in Block(branch.Else)) yield return name;
                    break;
                case WhileSyntax loop:
                    foreach (var name in Block(loop.Body)) yield return name;
                    break;
                case RepeatSyntax loop:
                    foreach (var name in Block(loop.Body)) yield return name;
                    break;
                case ForSyntax loop:
                    foreach (var name in Block(loop.Body, loop.Name.Text)) yield return name;
                    break;
                case ForEachSyntax loop:
                    foreach (var name in Block(loop.Body, loop.Name.Text)) yield return name;
                    break;
                case TrySyntax guarded:
                    foreach (var name in Block(guarded.Body)) yield return name;
                    foreach (var handler in guarded.Catches)
                    {
                        var handlerLocals = locals.ToHashSet(StringComparer.Ordinal);
                        if (handler.Name is not null) handlerLocals.Add(handler.Name.Text);
                        if (handler.Filter is not null)
                            foreach (var name in ExpressionWrites(handler.Filter, handlerLocals)) yield return name;
                        foreach (var name in StatementWrites(handler.Body, handlerLocals)) yield return name;
                    }
                    if (guarded.Finally is not null)
                        foreach (var name in Block(guarded.Finally)) yield return name;
                    break;
                case CaseStatementSyntax match:
                    foreach (var arm in match.Arms)
                    {
                        var armLocals = locals.Concat(arm.Patterns.SelectMany(PatternBindings)).ToHashSet(StringComparer.Ordinal);
                        if (arm.Guard is not null)
                            foreach (var name in ExpressionWrites(arm.Guard, armLocals)) yield return name;
                        foreach (var name in StatementWrites(arm.Statements!, armLocals)) yield return name;
                    }
                    break;
                case NestedRoutineSyntax nested:
                    var nestedLocals = locals.Concat(statements.OfType<BindingSyntax>().Select(binding => binding.Name.Text))
                        .Concat(statements.OfType<UsingSyntax>().Select(resource => resource.Binding.Name.Text))
                        .Concat(nested.Routine.Parameters.Select(parameter => parameter.Name.Text)).ToHashSet(StringComparer.Ordinal);
                    foreach (var name in StatementWrites(nested.Routine.Statements, nestedLocals)) yield return name;
                    break;
            }
        }
    }

    private static IEnumerable<string> PatternBindings(PatternSyntax pattern) => pattern switch
    {
        BindingPatternSyntax binding => [binding.Name.Text], TypePatternSyntax tested => [tested.Name.Text],
        NamedPatternSyntax named => (named.Fields ?? []).SelectMany(field => PatternBindings(field.Pattern)), _ => []
    };

    private static IEnumerable<StatementSyntax> Children(StatementSyntax statement) => statement switch
    {
        BlockSyntax block => block.Statements,
        CaseStatementSyntax match => match.Arms.SelectMany(arm => arm.Statements ?? []),
        IfSyntax branch => branch.Arms.SelectMany(arm => arm.Statements).Concat(branch.Else),
        WhileSyntax loop => loop.Body, RepeatSyntax loop => loop.Body, ForSyntax loop => loop.Body, ForEachSyntax loop => loop.Body,
        TrySyntax guarded => guarded.Body.Concat(guarded.Catches.SelectMany(handler => handler.Body)).Concat(guarded.Finally ?? []),
        _ => []
    };

    private static IEnumerable<ExpressionSyntax> Expressions(StatementSyntax statement) => statement switch
    {
        CaseStatementSyntax match => new[] { match.Selector }.Concat(match.Arms.Where(arm => arm.Guard is not null).Select(arm => arm.Guard!)),
        BindingSyntax binding => [binding.Initializer],
        UsingSyntax resource => [resource.Binding.Initializer],
        AssignmentSyntax assignment => [assignment.Target, assignment.Value],
        ExpressionStatementSyntax expression => [expression.Expression],
        SubscriptionSyntax subscription => [subscription.Event, subscription.Handler],
        ReturnSyntax { Expression: { } expression } => [expression],
        RaiseSyntax { Exception: { } expression } => [expression],
        IfSyntax branch => branch.Arms.Select(arm => arm.Condition),
        WhileSyntax loop => [loop.Condition], RepeatSyntax loop => [loop.Condition], ForSyntax loop => [loop.Lower, loop.Upper], ForEachSyntax loop => [loop.Source],
        TrySyntax guarded => guarded.Catches.Where(handler => handler.Filter is not null).Select(handler => handler.Filter!),
        _ => []
    };
}
