using Fpas.Compiler.Binding.Bound;
namespace Fpas.Compiler.Binding.Flow;

[Flags]
internal enum ExitFlow { None = 0, Next = 1, Return = 2, Break = 4, Continue = 8, Throw = 16 }

internal static class FlowAnalysis
{
    internal static ExitFlow Of(BoundStatement statement) => statement switch
    {
        BoundSequencePoint point => Of(point.Statement),
        BoundReturn => ExitFlow.Return,
        BoundThrow => ExitFlow.Throw,
        BoundUsing resource => Of(resource.Body),
        BoundTry guarded => Try(guarded),
        BoundLoopExit exit => exit.IsBreak ? ExitFlow.Break : ExitFlow.Continue,
        BoundBlock block => Sequence(block.Statements),
        BoundIf conditional => Conditional(conditional),
        BoundWhile loop => loop.Condition is BoundBoolean { Value: true }
            ? Of(loop.Body) & (ExitFlow.Return | ExitFlow.Throw) | (Of(loop.Body).HasFlag(ExitFlow.Break) ? ExitFlow.Next : ExitFlow.None)
            : ExitFlow.Next | Of(loop.Body) & (ExitFlow.Return | ExitFlow.Throw),
        BoundFor loop => ExitFlow.Next | Of(loop.Body) & (ExitFlow.Return | ExitFlow.Throw),
        BoundRepeat loop => Repeat(loop),
        _ => ExitFlow.Next
    };

    private static ExitFlow Repeat(BoundRepeat loop)
    {
        var body = Of(loop.Body);
        var result = body & (ExitFlow.Return | ExitFlow.Throw);
        if (body.HasFlag(ExitFlow.Break) || (body & (ExitFlow.Next | ExitFlow.Continue)) != 0 && loop.Condition is not BoundBoolean { Value: false })
            result |= ExitFlow.Next;
        return result;
    }

    private static ExitFlow Try(BoundTry guarded)
    {
        var flow = guarded.Catches.Aggregate(Of(guarded.Body), (current, handler) => current | Of(handler.Body));
        if (guarded.Finally is null) return flow;
        var cleanup = Of(guarded.Finally);
        return cleanup.HasFlag(ExitFlow.Next) ? flow | cleanup & ~ExitFlow.Next : cleanup;
    }

    internal static ExitFlow Sequence(IEnumerable<BoundStatement> statements)
    {
        var flow = ExitFlow.Next;
        foreach (var statement in statements)
            if (flow.HasFlag(ExitFlow.Next))
                flow = flow & ~ExitFlow.Next | Of(statement);
        return flow;
    }

    private static ExitFlow Conditional(BoundIf conditional)
    {
        var flow = ExitFlow.None;
        foreach (var arm in conditional.Arms)
        {
            if (arm.Condition is BoundBoolean { Value: false })
                continue;
            flow |= Of(arm.Body);
            if (arm.Condition is BoundBoolean { Value: true })
                return flow;
        }
        return flow | Of(conditional.Else);
    }
}
