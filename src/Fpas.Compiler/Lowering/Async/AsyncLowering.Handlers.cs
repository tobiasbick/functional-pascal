using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Bound;

namespace Fpas.Compiler.Lowering.Async;

internal sealed partial class AsyncLowering
{
    private int Try(BoundTry guarded, int next, Context outer)
    {
        int Cleanup(int target) => guarded.Finally is null ? target : Statement(guarded.Finally, target, outer);

        // Preserve the exception while cleanup executes: cleanup can itself catch an
        // exception and overwrite the state machine's pending exception field.
        int Unwind(int target)
        {
            if (guarded.Finally is null) return target;
            var pending = Temporary(ExceptionType, "<pending-exception>");
            var restore = Block(outer, new AsyncPropagate(new BoundLocal(pending), target));
            var cleanup = Cleanup(restore);
            return Block(outer, new AsyncGoto(cleanup), new BoundBinding(pending, new BoundPendingException(ExceptionType)));
        }

        // A catch belongs to this try; its own finally runs after the handler.
        // A search which escapes this try carries its finally into the second pass.
        int EscapingSearch(Func<int, int> innerUnwind)
            => outer.Search!(target => innerUnwind(Unwind(target)));
        var failed = EscapingSearch(target => target);
        var after = Cleanup(next);
        var context = outer with { Return = Cleanup(outer.Return), Break = outer.Break < 0 ? -1 : Cleanup(outer.Break),
            Continue = outer.Continue < 0 ? -1 : Cleanup(outer.Continue), Exception = failed, Search = EscapingSearch };

        int Search(Func<int, int> innerUnwind)
        {
            var handler = EscapingSearch(innerUnwind);
            if (guarded.Catches.Count == 0) return handler;
            var original = Temporary(ExceptionType, "<caught-exception>");
            foreach (var caught in guarded.Catches.Reverse())
            {
                var body = Statement(caught.Body, after, context with { Caught = original });
                var selected = innerUnwind(body);
                var following = handler;
                var filterFailed = Block(context, new AsyncPropagate(new BoundLocal(original), following));
                var filterContext = context with { Exception = filterFailed };
                var matched = caught.Filter is null ? selected : Expression(caught.Filter,
                    value => Block(filterContext, new AsyncBranch(value, selected, following)), filterContext);
                if (caught.Local is not null)
                    matched = Block(context, new AsyncGoto(matched), new BoundBinding(caught.Local, new BoundConversion(new BoundLocal(original), caught.Local.Type, false)));
                handler = Block(context, new AsyncBranch(new BoundTypeTest(new BoundLocal(original), caught.ExceptionType), matched, following));
            }
            return Block(context, new AsyncGoto(handler), new BoundBinding(original, new BoundPendingException(ExceptionType)));
        }

        var bodyContext = context with { Exception = Search(target => target), Search = Search };
        return Statement(guarded.Body, after, bodyContext);
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
