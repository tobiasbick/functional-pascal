using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Flow;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private BoundWhile BindWhile(WhileSyntax loop)
    {
        InvalidateAssigned(loop.Body);
        InvalidateEffects(loop.Condition);
        var incoming = nonNull.ToHashSet();
        var condition = Condition(loop.Condition);
        var afterCondition = nonNull.ToHashSet();
        SetFacts(Refine(loop.Condition, true, afterCondition));
        loopDepth++;
        BoundBlock body;
        try { body = BindBlock(loop.Body); }
        finally { loopDepth--; }
        SetFacts(FlowAnalysis.Of(body).HasFlag(ExitFlow.Break) ? Merge([incoming, afterCondition]) : Refine(loop.Condition, false, afterCondition));
        return new(condition, body);
    }

    private BoundRepeat BindRepeat(RepeatSyntax loop)
    {
        InvalidateAssigned(loop.Body);
        InvalidateEffects(loop.Condition);
        var incoming = nonNull.ToHashSet();
        loopDepth++;
        BoundBlock body;
        try { body = BindBlock(loop.Body); }
        finally { loopDepth--; }
        // Continue can skip assignments in the body; the condition must be safe
        // on every route that reaches it, after the iteration's cleanup.
        SetFacts(incoming);
        var condition = Condition(loop.Condition);
        var after = nonNull.ToHashSet();
        SetFacts(FlowAnalysis.Of(body).HasFlag(ExitFlow.Break) ? Merge([incoming, after]) : Refine(loop.Condition, true, after));
        return new(condition, body, source.StatementLocation(loop.Until.Position));
    }

    private BoundFor BindFor(ForSyntax loop)
    {
        var type = BindType(loop.Type);
        if (type.IsNullable || !NumericFacts.Integral(type) && references.ScalarInfo(type)?.Category != ScalarCategory.Range)
            throw source.Error("FP3002", "A counting loop requires a non-nullable primitive integral or subrange counter.", loop.Type.Position);
        var lower = Convert(BindExpression(loop.Lower, expected: type), type, loop.Lower.Position);
        var upper = Convert(BindExpression(loop.Upper, expected: type), type, loop.Upper.Position);
        InvalidateAssigned(loop.Body);
        var incoming = nonNull.ToHashSet();
        scopes.Push(new(StringComparer.Ordinal)); loopDepth++;
        try
        {
            var counter = Allocate(loop.Name.Text, type); Declare(counter, loop.Name.Position);
            var limit = Allocate("<ForLimit>", type);
            return CountingLoop(counter, limit, lower, upper, loop.Descending, BindBlock(loop.Body, newScope: false));
        }
        finally
        {
            loopDepth--;
            foreach (var local in scopes.Pop().Values) nonNull.Remove(local);
            SetFacts(incoming);
        }
    }

    private BoundFor CountingLoop(LocalSymbol counter, LocalSymbol limit, BoundExpression lower, BoundExpression upper, bool descending, BoundBlock body)
    {
        var range = references.ScalarInfo(counter.Type);
        BoundExpression Read(LocalSymbol local) => range is null ? new BoundLocal(local) : new BoundCall(ReferenceCatalog.DomainGetter(range), [], new BoundLocal(local));
        var value = Read(counter); var endpoint = Read(limit);
        var next = NumericBinary(value, descending ? "-" : "+", new BoundInteger(1), 0, true);
        next = Convert(next, range?.Base ?? counter.Type, 0, explicitCast: true);
        if (range is not null) next = new BoundNew(counter.Type, ReferenceCatalog.DomainConstructor(range), [next]);
        return new(counter, limit, lower, upper, NumericBinary(value, descending ? ">=" : "<=", endpoint, 0, false),
            NumericBinary(value, "=", endpoint, 0, false), next, body);
    }
}
