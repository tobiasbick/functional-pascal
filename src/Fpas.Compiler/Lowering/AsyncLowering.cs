using Fpas.Compiler.Binding;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Lowering;

internal sealed partial class AsyncLowering(BoundRoutine routine, ReferenceCatalog references)
{
    private sealed record Context(int Exception, int Return, int Break = -1, int Continue = -1, LocalSymbol? Caught = null);
    private readonly AsyncPlan plan = new(routine);
    private TypeSymbol ExceptionType => references.Symbol(references.Required("System.Exception"));
    private readonly SourceText source = routine.Source!;

    internal static AsyncPlan Lower(BoundRoutine routine, ReferenceCatalog references)
    {
        var lowering = new AsyncLowering(routine, references);
        lowering.Build();
        AsyncLiveness.Validate(lowering.plan, routine.Source!);
        return lowering.plan;
    }

    private void Build()
    {
        var failure = Block(new(0, 0), new AsyncFail());
        var completed = Block(new(failure, 0), new AsyncComplete());
        var context = new Context(failure, completed);
        if (routine.Symbol.BodyResult != TypeSymbol.Void) plan.Result ??= Temporary(routine.Symbol.BodyResult, "<async-result>");
        var success = Contracts(routine.Ensures, completed, context);
        context = context with { Return = success };
        var body = Statement(routine.Body, success, context);
        plan.Entry = Contracts(routine.Requires, body, context);
    }

    private LocalSymbol Temporary(TypeSymbol type, string name = "<async-temp>")
    {
        var local = new LocalSymbol(name, type, plan.Locals.Count);
        plan.Locals.Add(local);
        return local;
    }

    private int Block(Context context, AsyncExit exit, params BoundStatement[] statements)
    {
        var id = plan.Blocks.Count;
        plan.Blocks.Add(new(id, context.Exception, exit, statements));
        return id;
    }

    private int Contracts(IReadOnlyList<BoundContract>? contracts, int next, Context context)
    {
        foreach (var contract in (contracts ?? []).Reverse())
        {
            var failed = Block(context, new AsyncGoto(next), new BoundExpressionStatement(new BoundFailure(TypeSymbol.Void, contract.Message), false));
            var check = Block(context, new AsyncBranch(contract.Condition, next, failed));
            plan.Blocks[check].Location = contract.Location;
            next = check;
        }
        return next;
    }

    private int Statement(BoundStatement statement, int next, Context context)
    {
        switch (statement)
        {
            case BoundSequencePoint point:
                var firstBlock = plan.Blocks.Count;
                var entry = Statement(point.Statement, next, context);
                foreach (var block in plan.Blocks.Skip(firstBlock)) block.Location ??= point.Location;
                var marked = Block(context, new AsyncGoto(entry));
                plan.Blocks[marked].Location = point.Location;
                return marked;
            case BoundBlock block:
                foreach (var child in block.Statements.Reverse()) next = Statement(child, next, context);
                return next;
            case BoundBinding binding:
                return Expression(binding.Initializer, value => Block(context, new AsyncGoto(next), new BoundBinding(binding.Local, value)), context);
            case BoundAssignment assignment:
                return LValue(assignment.Target, target => Expression(assignment.Value,
                    value => Block(context, new AsyncGoto(next), new BoundAssignment(target, value)), context), context);
            case BoundExpressionStatement expression:
                return Expression(expression.Expression, value => Block(context, new AsyncGoto(next), new BoundExpressionStatement(value, expression.Discard)), context);
            case BoundReturn returned:
                return returned.Expression is null ? context.Return : Expression(returned.Expression,
                    value => Block(context, new AsyncGoto(context.Return), new BoundAssignment(new BoundLocal(plan.Result!), value)), context);
            case BoundThrow raised:
                return raised.Exception is null ? Block(context, new AsyncPropagate(new BoundLocal(context.Caught!), context.Exception))
                    : Expression(raised.Exception, value => Block(context, new AsyncRaise(value)), context);
            case BoundLoopExit exit: return exit.IsBreak ? context.Break : context.Continue;
            case BoundIf conditional:
                var otherwise = Statement(conditional.Else, next, context);
                foreach (var arm in conditional.Arms.Reverse())
                {
                    var yes = Statement(arm.Body, next, context); var no = otherwise;
                    otherwise = Expression(arm.Condition, condition => Block(context, new AsyncBranch(condition, yes, no)), context);
                }
                return otherwise;
            case BoundWhile loop:
                var conditionEntry = Block(context, new AsyncGoto(-1));
                var body = Statement(loop.Body, conditionEntry, context with { Break = next, Continue = conditionEntry });
                var condition = Expression(loop.Condition, value => Block(context, new AsyncBranch(value, body, next)), context);
                plan.Blocks[conditionEntry].Exit = new AsyncGoto(condition);
                return conditionEntry;
            case BoundFor loop: return For(loop, next, context);
            case BoundRepeat loop:
                var repeatCondition = Block(context, new AsyncGoto(-1));
                var repeatBody = Statement(loop.Body, repeatCondition, context with { Break = next, Continue = repeatCondition });
                var until = Expression(loop.Condition, value => Block(context, new AsyncBranch(value, next, repeatBody)), context);
                plan.Blocks[repeatCondition].Exit = new AsyncGoto(until);
                plan.Blocks[repeatCondition].Location = loop.ConditionLocation;
                return repeatBody;
            case BoundTry guarded: return Try(guarded, next, context);
            case BoundUsing resource:
                var cleanup = resource.IsAsync ? AsyncDispose(resource, context) : new BoundBlock([new BoundDispose(resource)]);
                var protectedBody = Try(new(resource.Body, [], cleanup), next, context);
                return Expression(resource.Initializer, value => Block(context, new AsyncGoto(protectedBody), new BoundBinding(resource.Local, value)), context);
            case BoundDispose dispose: return Block(context, new AsyncGoto(next), dispose);
            default: throw new InvalidOperationException("Unknown statement in async lowering.");
        }
    }

    private int For(BoundFor loop, int next, Context context)
    {
        var counter = new BoundLocal(loop.Counter);
        var head = Block(context, new AsyncGoto(-1));
        var increment = Block(context, new AsyncGoto(head), new BoundAssignment(counter, loop.Advance));
        var step = Block(context, new AsyncBranch(loop.AtLimit, next, increment));
        var body = Statement(loop.Body, step, context with { Break = next, Continue = step });
        plan.Blocks[head].Exit = new AsyncBranch(loop.Test, body, next);
        return Expression(loop.Lower, lower => Block(context, new AsyncGoto(Expression(loop.Upper,
            upper => Block(context, new AsyncGoto(head), new BoundBinding(loop.Limit, upper)), context)), new BoundBinding(loop.Counter, lower)), context);
    }
}
