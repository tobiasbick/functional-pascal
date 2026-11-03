using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Flow;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private (BoundBinding Saved, BoundLocal Value, object? Stable) CaseSelector(ExpressionSyntax syntax, bool check = true)
    {
        var value = BindExpression(syntax, check);
        if (value.Type.Kind is TypeKind.Void or TypeKind.Unsupported) throw source.Error("FP3024", "A case selector needs a supported value type.", syntax.Position);
        var type = value switch
        {
            BoundLocal local => local.Local.Type, BoundParameter parameter => parameter.Parameter.Type,
            BoundCapture capture => capture.Capture.Type, BoundNull => TypeSymbol.Object with { IsNullable = true }, _ => value.Type
        };
        var saved = Allocate("<case-selector>", type);
        return (new(saved, new BoundConversion(value, type, false)), new(saved), StableSymbol(syntax));
    }

    private (BoundExpression Test, IReadOnlyList<MatchPattern> Patterns) CasePatterns(CaseArmSyntax arm, BoundLocal value, object? stable)
    {
        var bindings = new Dictionary<string, LocalSymbol>(StringComparer.Ordinal);
        HashSet<string>? expected = null;
        var patterns = new List<MatchPattern>();
        BoundExpression? test = null;
        foreach (var syntax in arm.Patterns)
        {
            var names = new HashSet<string>(StringComparer.Ordinal);
            var pattern = BindPattern(syntax, value.Type, bindings, names);
            if (expected is not null && !expected.SetEquals(names)) throw source.Error("FP3024", "Alternative patterns must bind identical names and types.", arm.Position);
            expected ??= names;
            patterns.Add(pattern);
            var condition = LowerPattern(pattern, value);
            test = test is null ? condition : new BoundBinary(test, "or", condition, TypeSymbol.Boolean, false);
        }
        if (stable is not null && value.Type.CanHaveNull && patterns.All(pattern => pattern is CaseMatch or TypeMatch || pattern is ConstantMatch { Value: not null })) nonNull.Add(stable);
        return (test ?? new BoundBoolean(false), patterns);
    }

    private void CheckCaseCoverage(CaseArmSyntax arm, IReadOnlyList<MatchPattern> patterns, List<MatchPattern> covered, TypeSymbol type, PatternCoverage coverage)
    {
        var alternatives = covered.ToList();
        foreach (var pattern in patterns)
        {
            if (!coverage.Useful(alternatives, pattern, type)) throw source.Error("FP3025", "This case pattern is unreachable or already covered.", arm.Position);
            alternatives.Add(pattern);
        }
        if (arm.Guard is null) covered.AddRange(patterns);
    }

    private void PopCaseScope()
    {
        foreach (var local in scopes.Pop().Values) nonNull.Remove(local);
    }

    private BoundStatement BindCaseStatement(CaseStatementSyntax syntax)
    {
        var selector = CaseSelector(syntax.Selector);
        var remainder = nonNull.ToHashSet();
        var covered = new List<MatchPattern>();
        var coverage = new PatternCoverage(references, source, syntax.Position);
        var arms = new List<BoundIfArm>(); var exits = new List<HashSet<object>>();
        foreach (var arm in syntax.Arms)
        {
            SetFacts(remainder); scopes.Push(new(StringComparer.Ordinal));
            try
            {
                var (test, patterns) = CasePatterns(arm, selector.Value, selector.Stable);
                CheckCaseCoverage(arm, patterns, covered, selector.Value.Type, coverage);
                if (arm.Guard is not null)
                {
                    test = Both(test, Condition(arm.Guard));
                    if (selector.Stable is { } stable && !AfterEffects(arm.Guard, [stable]).Contains(stable)) selector.Stable = null;
                    remainder = Merge([remainder, Refine(arm.Guard, false, nonNull.ToHashSet())]);
                    SetFacts(Refine(arm.Guard, true, nonNull.ToHashSet()));
                }
                var body = BindBlock(arm.Statements!, newScope: false);
                var next = FlowAnalysis.Of(body).HasFlag(ExitFlow.Next);
                PopCaseScope();
                if (next) exits.Add(nonNull.ToHashSet());
                arms.Add(new(test, body));
            }
            catch { PopCaseScope(); throw; }
        }
        if (coverage.Useful(covered, new AnyMatch(), selector.Value.Type)) throw source.Error("FP3025", "Case matching is not exhaustive; add the missing cases or a fallback.", syntax.Position);
        SetFacts(Merge(exits));
        var exception = references.Symbol(references.Required("System.InvalidOperationException"));
        return new BoundBlock([selector.Saved, new BoundIf(arms, new([new BoundThrow(new BoundNew(exception,
            references.FindMethod(exception, ".ctor", [TypeSymbol.String], 0, source, false), [new BoundString("No case pattern matched.")]))]))]);
    }

    private BoundExpression BindCaseExpression(CaseExpressionSyntax syntax, bool check, TypeSymbol? expected)
    {
        var selector = CaseSelector(syntax.Selector, check);
        var remainder = nonNull.ToHashSet();
        var covered = new List<MatchPattern>(); var coverage = new PatternCoverage(references, source, syntax.Position);
        var arms = new List<BoundConditionalArm>(); var exits = new List<HashSet<object>>();
        foreach (var arm in syntax.Arms)
        {
            SetFacts(remainder); scopes.Push(new(StringComparer.Ordinal));
            try
            {
                var (test, patterns) = CasePatterns(arm, selector.Value, selector.Stable);
                CheckCaseCoverage(arm, patterns, covered, selector.Value.Type, coverage);
                if (arm.Guard is not null)
                {
                    test = Both(test, Condition(arm.Guard, check));
                    if (selector.Stable is { } stable && !AfterEffects(arm.Guard, [stable]).Contains(stable)) selector.Stable = null;
                    remainder = Merge([remainder, Refine(arm.Guard, false, nonNull.ToHashSet())]);
                    SetFacts(Refine(arm.Guard, true, nonNull.ToHashSet()));
                }
                arms.Add(new(test, BindExpression(arm.Value!, check, expected)));
                PopCaseScope(); exits.Add(nonNull.ToHashSet());
            }
            catch { PopCaseScope(); throw; }
        }
        if (coverage.Useful(covered, new AnyMatch(), selector.Value.Type)) throw source.Error("FP3025", "Case matching is not exhaustive; add the missing cases or a fallback.", syntax.Position);
        SetFacts(Merge(exits));
        var result = arms.Select(arm => arm.Value.Type).Aggregate(references.Common);
        if (result.Kind is TypeKind.Void or TypeKind.Unsupported) throw source.Error("FP3002", "Case expression arms require a common value type.", syntax.Position);
        return new BoundSequence([selector.Saved], new BoundConditional(arms.Select(arm => new BoundConditionalArm(arm.Condition, Convert(arm.Value, result, syntax.Position))).ToArray(),
            new BoundFailure(result, "No case pattern matched."), result));
    }
}
