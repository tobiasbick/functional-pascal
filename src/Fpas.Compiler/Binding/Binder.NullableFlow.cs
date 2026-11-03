using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class Binder
{
    private void SetFacts(IEnumerable<object> facts)
    {
        var copy = facts.ToArray();
        nonNull.Clear(); nonNull.UnionWith(copy);
    }

    private void UpdateFact(LocalSymbol local, BoundExpression value)
    {
        nonNull.Remove(local);
        if (local.Type.CanHaveNull && local.Type.NeedsNullCheck && !value.Type.NeedsNullCheck && value.Type != TypeSymbol.Null) nonNull.Add(local);
    }

    private object? StableSymbol(ExpressionSyntax syntax) => syntax switch
    {
        ParenthesizedSyntax parenthesized => StableSymbol(parenthesized.Expression),
        NameExpressionSyntax name when HasValue(name.Name.Text) => Value(name.Name.Text, name.Position) switch
        {
            BoundLocal local => local.Local, BoundParameter parameter => parameter.Parameter, BoundCapture capture => capture.Capture, _ => null
        },
        _ => null
    };

    private HashSet<object> Refine(ExpressionSyntax syntax, bool truth, HashSet<object> input)
    {
        if (syntax is ParenthesizedSyntax wrapped) return Refine(wrapped.Expression, truth, input);
        if (syntax is UnarySyntax { Operator.Text: "not" } unary) return Refine(unary.Operand, !truth, input);
        if (truth && syntax is TypeTestSyntax test && StableSymbol(test.Value) is { } tested)
        {
            var type = tested switch { LocalSymbol local => local.Type, ParameterSymbol parameter => parameter.Type, CaptureSymbol capture => capture.Type, _ => TypeSymbol.Unsupported };
            if (type.CanHaveNull) { var refined = input.ToHashSet(); refined.Add(tested); return refined; }
        }
        if (syntax is BinarySyntax binary)
        {
            var op = binary.Operator.Text;
            if (op is "and" or "or")
            {
                var first = op == "and";
                if (truth == first) return Refine(binary.Right, truth, Refine(binary.Left, truth, input));
                var skipped = Refine(binary.Left, truth, input);
                skipped.IntersectWith(Refine(binary.Right, truth, Refine(binary.Left, !truth, input)));
                return skipped;
            }
            if (op is "=" or "<>")
            {
                var candidate = binary.Left is LiteralSyntax { Token.Text: "null", Token.Kind: TokenKind.Keyword } ? binary.Right :
                    binary.Right is LiteralSyntax { Token.Text: "null", Token.Kind: TokenKind.Keyword } ? binary.Left : null;
                if (candidate is not null && StableSymbol(candidate) is { } symbol)
                {
                    var type = symbol switch { LocalSymbol local => local.Type, ParameterSymbol parameter => parameter.Type, CaptureSymbol capture => capture.Type, _ => TypeSymbol.Unsupported };
                    var result = input.ToHashSet();
                    if (type.CanHaveNull && type.NeedsNullCheck)
                    {
                        if (truth == (op == "<>")) result.Add(symbol); else result.Remove(symbol);
                    }
                    return result;
                }
            }
        }
        return MayInvoke(syntax) ? input.Where(symbol => !IsMutableCapture(symbol)).ToHashSet() : input.ToHashSet();
    }

    private BoundIf BindIf(IfSyntax conditional)
    {
        var remainder = nonNull.ToHashSet();
        var arms = new List<BoundIfArm>();
        var exits = new List<HashSet<object>>();
        foreach (var arm in conditional.Arms)
        {
            SetFacts(remainder);
            var condition = Condition(arm.Condition);
            var afterCondition = nonNull.ToHashSet();
            SetFacts(Refine(arm.Condition, true, afterCondition));
            var body = BindBlock(arm.Statements);
            if (FlowAnalysis.Of(body).HasFlag(ExitFlow.Next)) exits.Add(nonNull.ToHashSet());
            arms.Add(new(condition, body));
            remainder = Refine(arm.Condition, false, afterCondition);
        }
        SetFacts(remainder);
        var otherwise = BindBlock(conditional.Else);
        if (FlowAnalysis.Of(otherwise).HasFlag(ExitFlow.Next)) exits.Add(nonNull.ToHashSet());
        SetFacts(Merge(exits));
        return new(arms, otherwise);
    }

    private BoundExpression BindConditional(ConditionalSyntax conditional, bool check, TypeSymbol? expected)
    {
        var remainder = nonNull.ToHashSet();
        var arms = new List<BoundConditionalArm>();
        var exits = new List<HashSet<object>>();
        foreach (var arm in conditional.Arms)
        {
            SetFacts(remainder);
            var condition = Condition(arm.Condition, check);
            var afterCondition = nonNull.ToHashSet();
            SetFacts(Refine(arm.Condition, true, afterCondition));
            arms.Add(new(condition, BindExpression(arm.Value, check, expected)));
            exits.Add(nonNull.ToHashSet());
            remainder = Refine(arm.Condition, false, afterCondition);
        }
        SetFacts(remainder);
        var otherwise = BindExpression(conditional.Else, check, expected);
        exits.Add(nonNull.ToHashSet());
        SetFacts(Merge(exits));
        var common = arms.Aggregate(otherwise.Type, (type, arm) => references.Common(type, arm.Value.Type));
        if (common.Kind is TypeKind.Void or TypeKind.Unsupported)
            throw source.Error("FP3002", "Conditional branches require a common value type.", conditional.Position);
        return new BoundConditional(arms.Select(arm => new BoundConditionalArm(arm.Condition, Convert(arm.Value, common, conditional.Position))).ToArray(),
            Convert(otherwise, common, conditional.Else.Position), common);
    }

    private static HashSet<object> Merge(IReadOnlyList<HashSet<object>> exits)
    {
        if (exits.Count == 0) return [];
        var result = exits[0].ToHashSet();
        foreach (var exit in exits.Skip(1)) result.IntersectWith(exit);
        return result;
    }

    private void InvalidateAssigned(IEnumerable<StatementSyntax> statements)
    {
        ForgetCapturedFacts();
        foreach (var statement in statements)
        {
            foreach (var symbol in Expressions(statement).SelectMany(ReferenceWrites)) nonNull.Remove(symbol);
            foreach (var assignedName in AssignedInLambdas(statement))
                if (FindLocal(assignedName) is { } captured) nonNull.Remove(captured);
            if (statement is AssignmentSyntax { Target: NameExpressionSyntax name } && FindLocal(name.Name.Text) is { } local) nonNull.Remove(local);
            var nested = statement switch
            {
                CaseStatementSyntax match => match.Arms.SelectMany(arm => arm.Statements ?? []),
                BlockSyntax block => block.Statements,
                IfSyntax branch => branch.Arms.SelectMany(arm => arm.Statements).Concat(branch.Else),
                WhileSyntax loop => loop.Body, RepeatSyntax loop => loop.Body, ForSyntax loop => loop.Body, ForEachSyntax loop => loop.Body,
                TrySyntax guarded => guarded.Body.Concat(guarded.Catches.SelectMany(handler => handler.Body)).Concat(guarded.Finally ?? []),
                _ => []
            };
            InvalidateAssigned(nested);
        }
    }

    private IEnumerable<object> ReferenceWrites(ExpressionSyntax expression)
    {
        if (expression is ArgumentModeSyntax { Mode: "ref" or "out" } argument && StableSymbol(argument.Value) is { } symbol)
            yield return symbol;
        foreach (var child in Children(expression))
            foreach (var written in ReferenceWrites(child)) yield return written;
    }
}
