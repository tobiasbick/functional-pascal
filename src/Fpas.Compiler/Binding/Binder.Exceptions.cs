using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class Binder
{
    private BoundUsing BindUsing(UsingSyntax syntax, IReadOnlyList<StatementSyntax> remainder)
    {
        var type = BindType(syntax.Binding.Type);
        var value = BindExpression(syntax.Binding.Initializer);
        var initializer = Convert(value, type, syntax.Position);
        if (syntax.IsAsync && !currentRoutine.IsAsync) throw source.Error("FP3030", "await using requires an async body.", syntax.Position);
        var disposable = references.Symbol(references.Required(syntax.IsAsync ? "System.IAsyncDisposable" : "System.IDisposable"));
        if (!references.Derives(type, disposable))
            throw source.Error("FP3016", $"A using resource must implement {(syntax.IsAsync ? "System.IAsyncDisposable" : "System.IDisposable")}.", syntax.Position);
        var local = Allocate(syntax.Binding.Name.Text, type);
        local.Documentation = syntax.Binding.Documentation;
        Declare(local, syntax.Position);
        UpdateFact(local, value);
        var dispose = references.FindMethod(disposable, syntax.IsAsync ? "DisposeAsync" : "Dispose", [], syntax.Position, source, isStatic: false);
        var body = BindBlock(remainder, newScope: false);
        ForgetCapturedFacts();
        return new(local, initializer, dispose, body, IsAsync: syntax.IsAsync);
    }

    private BoundTry BindTry(TrySyntax syntax)
    {
        var entry = nonNull.ToHashSet();
        InvalidateAssigned(syntax.Body.Concat(syntax.Catches.SelectMany(handler => handler.Body)).Concat(syntax.Finally ?? []));
        var incoming = nonNull.ToHashSet();
        SetFacts(entry);
        var body = BindBlock(syntax.Body);
        var exits = new List<HashSet<object>>();
        if (FlowAnalysis.Of(body).HasFlag(ExitFlow.Next)) exits.Add(nonNull.ToHashSet());
        var catches = new List<BoundCatch>();
        var exception = references.Symbol(references.Required("System.Exception"));
        foreach (var handler in syntax.Catches)
        {
            var type = handler.Type is null ? exception : BindType(handler.Type);
            if (type.IsNullable || !type.IsReference || !references.Derives(type, exception))
                throw source.Error("FP3017", "A catch type must be a non-nullable CLR exception type.", handler.Position);
            if (catches.Any(previous => previous.Filter is null && references.Derives(type, previous.ExceptionType)))
                throw source.Error("FP3017", "This catch is hidden by an earlier unfiltered catch.", handler.Position);
            SetFacts(incoming);
            scopes.Push(new(StringComparer.Ordinal));
            catchDepth++;
            try
            {
                LocalSymbol? local = null;
                if (handler.Name is not null)
                {
                    local = Allocate(handler.Name.Text, type);
                    Declare(local, handler.Name.Position);
                }
                var previousFilter = bindingFilter; bindingFilter = true;
                BoundExpression? filter;
                try { filter = handler.Filter is null ? null : Condition(handler.Filter); }
                finally { bindingFilter = previousFilter; }
                var handlerBody = BindBlock(handler.Body, newScope: false);
                if (FlowAnalysis.Of(handlerBody).HasFlag(ExitFlow.Next)) exits.Add(nonNull.ToHashSet());
                catches.Add(new(type, local, filter, handlerBody));
            }
            finally
            {
                catchDepth--;
                foreach (var local in scopes.Pop().Values) nonNull.Remove(local);
            }
        }
        var merged = Merge(exits);
        BoundBlock? cleanup = null;
        if (syntax.Finally is not null)
        {
            SetFacts(incoming);
            finallyBoundaries.Push((loopDepth, catchDepth));
            try { cleanup = BindBlock(syntax.Finally); }
            finally { finallyBoundaries.Pop(); }
            var final = nonNull.ToHashSet();
            SetFacts(merged);
            InvalidateAssigned(syntax.Finally);
            merged = nonNull.ToHashSet();
            merged.UnionWith(final.Except(incoming));
        }
        SetFacts(merged);
        return new(body, catches, cleanup);
    }
}
