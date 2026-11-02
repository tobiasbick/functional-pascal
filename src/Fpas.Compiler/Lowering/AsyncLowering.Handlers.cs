using Fpas.Compiler.Binding;

namespace Fpas.Compiler.Lowering;

internal sealed partial class AsyncLowering
{
    private int Try(BoundTry guarded, int next, Context outer)
    {
        int Cleanup(int target) => guarded.Finally is null ? target : Statement(guarded.Finally, target, outer);
        var after = Cleanup(next);
        var failed = outer.Exception;
        if (guarded.Finally is not null)
        {
            var pending = Temporary(ExceptionType, "<pending-exception>");
            var propagate = Block(outer, new AsyncPropagate(new BoundLocal(pending), outer.Exception));
            var cleanup = Cleanup(propagate);
            failed = Block(outer, new AsyncGoto(cleanup), new BoundBinding(pending, new BoundPendingException(ExceptionType)));
        }
        var context = outer with { Return = Cleanup(outer.Return), Break = outer.Break < 0 ? -1 : Cleanup(outer.Break),
            Continue = outer.Continue < 0 ? -1 : Cleanup(outer.Continue), Exception = failed };
        var handler = failed;
        if (guarded.Catches.Count != 0)
        {
            var original = Temporary(ExceptionType, "<caught-exception>");
            handler = Block(context, new AsyncPropagate(new BoundLocal(original), failed));
            foreach (var caught in guarded.Catches.Reverse())
            {
                var body = Statement(caught.Body, after, context with { Caught = original });
                var following = handler;
                var filterFailed = Block(context, new AsyncPropagate(new BoundLocal(original), following));
                var filterContext = context with { Exception = filterFailed };
                var matched = caught.Filter is null ? body : Expression(caught.Filter,
                    value => Block(filterContext, new AsyncBranch(value, body, following)), filterContext);
                if (caught.Local is not null)
                    matched = Block(context, new AsyncGoto(matched), new BoundBinding(caught.Local, new BoundConversion(new BoundLocal(original), caught.Local.Type, false)));
                handler = Block(context, new AsyncBranch(new BoundTypeTest(new BoundLocal(original), caught.ExceptionType), matched, following));
            }
            handler = Block(context, new AsyncGoto(handler), new BoundBinding(original, new BoundPendingException(ExceptionType)));
        }
        return Statement(guarded.Body, after, context with { Exception = handler });
    }

    private BoundBlock AsyncDispose(BoundUsing resource, Context context)
    {
        BoundExpression receiver = new BoundLocal(resource.Local);
        var present = receiver.Type.CanHaveNull || receiver.Type.NullableValue;
        if (present) receiver = new BoundConversion(receiver, receiver.Type.NonNullable, true);
        var call = new BoundCall(resource.Dispose, [], receiver);
        var awaited = references.Await(call, source, 0) with { IsImplicit = true };
        BoundStatement cleanup = new BoundExpressionStatement(awaited, false);
        if (present) cleanup = new BoundIf([new(new BoundNullTest(new BoundLocal(resource.Local), false), new([cleanup]))], new([]));
        return new([cleanup]);
    }
}
