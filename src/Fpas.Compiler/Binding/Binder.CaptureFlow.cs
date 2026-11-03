using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class Binder
{
    private bool MayInvoke(ExpressionSyntax expression)
        => expression is CallSyntax or NewSyntax or MemberSyntax or IndexSyntax or WithSyntax or CaseExpressionSyntax ||
            comparisonCalls.ContainsKey(expression) || Children(expression).Any(MayInvoke);

    private static IEnumerable<string> AssignedInLambdas(StatementSyntax statement)
        => Expressions(statement).SelectMany(AssignedInLambdas);

    private static IEnumerable<string> AssignedInLambdas(ExpressionSyntax expression)
    {
        if (expression is LambdaSyntax { Body: { } body })
            foreach (var name in AssignedNames(body)) yield return name;
        foreach (var child in Children(expression))
            foreach (var name in AssignedInLambdas(child)) yield return name;
    }

    private static IEnumerable<string> AssignedNames(IEnumerable<StatementSyntax> statements)
    {
        foreach (var statement in statements)
        {
            if (statement is AssignmentSyntax { Target: NameExpressionSyntax name }) yield return name.Name.Text;
            foreach (var nested in AssignedNames(Children(statement))) yield return nested;
            foreach (var nested in AssignedInLambdas(statement)) yield return nested;
        }
    }

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

    private static IEnumerable<ExpressionSyntax> Children(ExpressionSyntax expression) => expression switch
    {
        TypeTestSyntax test => [test.Value],
        CaseExpressionSyntax match => new[] { match.Selector }.Concat(match.Arms.SelectMany(arm => arm.Guard is null ? new[] { arm.Value! } : new[] { arm.Guard, arm.Value! })),
        ParenthesizedSyntax wrapped => [wrapped.Expression], UnarySyntax unary => [unary.Operand],
        BinarySyntax binary => [binary.Left, binary.Right], CastSyntax cast => [cast.Operand], UncheckedSyntax uncheckedExpression => [uncheckedExpression.Operand],
        CallSyntax call => new[] { call.Target }.Concat(call.Arguments), MemberSyntax member => [member.Receiver],
        GenericNameSyntax generic => [generic.Name], IndexSyntax index => new[] { index.Receiver }.Concat(index.Indices),
        NamedArgumentSyntax named => [named.Value],
        ArgumentModeSyntax argument => [argument.Value], AwaitSyntax awaited => [awaited.Operand],
        WithSyntax updated => new[] { updated.Receiver }.Concat(updated.Updates.Select(update => update.Value)),
        NewSyntax created => created.Arguments, NewArraySyntax array => array.Lengths, ArrayLiteralSyntax array => array.Elements,
        ConditionalSyntax branch => branch.Arms.SelectMany(arm => new[] { arm.Condition, arm.Value }).Append(branch.Else),
        LambdaSyntax { Expression: { } body } => [body], _ => []
    };
}
