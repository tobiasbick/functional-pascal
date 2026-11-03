namespace Fpas.Compiler.Binding;

internal sealed partial class Binder
{
    private sealed record EnvironmentSite(Binder Binder, ClosureSymbol Target, List<BoundExpression> Arguments,
        HashSet<LocalSymbol> Available, int Position);
    private readonly List<EnvironmentSite> environmentSites = [];
    private Binder Root => parent?.Root ?? this;

    private BoundExpression NestedReceiver(ClosureSymbol target, int position)
    {
        if (closure == target) return new BoundClosureSelf(target);
        var arguments = new List<BoundExpression>();
        Root.environmentSites.Add(new(this, target, arguments, scopes.SelectMany(scope => scope.Values).ToHashSet(), position));
        return new BoundEnvironment(target, arguments);
    }

    private BoundExpression ReadOriginal(object symbol, int position)
    {
        if (symbol is LocalSymbol local && locals.Contains(local)) return new BoundLocal(local);
        if (symbol is ParameterSymbol parameter && parameters.Values.Any(item => ReferenceEquals(item, parameter))) return new BoundParameter(parameter);
        if (parent is null) throw new InvalidOperationException("A nested environment contains an unrelated binding.");
        return CaptureValue(symbol, parent.ReadOriginal(symbol, position), position);
    }

    private BoundExpression CaptureValue(object symbol, BoundExpression original, int position)
    {
        if (symbol is ParameterSymbol { ByRef: true }) throw source.Error("FP3028", "Managed reference parameters cannot be captured.", position);
        if (!captures.TryGetValue(symbol, out var capture))
        {
            capture = closure!.Captures.FirstOrDefault(item => ReferenceEquals(item.Original, symbol));
            if (capture is null)
            {
                var mutable = symbol is LocalSymbol { Mutable: true };
                var type = mutable ? ((LocalSymbol)symbol).Type : original.Type;
                if (mutable)
                {
                    ((LocalSymbol)symbol).Captured = true; parent!.ForgetCapturedFacts();
                    original = Storage(original, references.CellType(type));
                }
                capture = new(symbol, type, original, mutable, closure); closure.Captures.Add(capture);
            }
            captures.Add(symbol, capture);
        }
        return new BoundCapture(capture, nonNull.Contains(capture) ? capture.Type.NonNullable : null);
    }

    private static BoundExpression Storage(BoundExpression original, TypeSymbol type) => original switch
    {
        BoundLocal local => new BoundLocalStorage(local.Local, type),
        BoundCapture capture => new BoundCaptureStorage(capture.Capture, type),
        _ => throw new InvalidOperationException("A mutable capture requires local storage.")
    };

    private void FinishNestedBindings()
    {
        bool changed;
        do
        {
            changed = false;
            foreach (var site in environmentSites)
                while (site.Arguments.Count < site.Target.Captures.Count)
                {
                    var capture = site.Target.Captures[site.Arguments.Count];
                    if (capture.Original is LocalSymbol local && site.Binder.locals.Contains(local) && !site.Available.Contains(local))
                        throw source.Error("FP3033", $"Nested routines require '{local.Name}' to be initialized before a call or delegate creation.", site.Position);
                    var value = site.Binder.ReadOriginal(capture.Original, site.Position);
                    if (capture.Mutable) value = Storage(value, references.CellType(capture.Type));
                    site.Arguments.Add(value); changed = true;
                }
        } while (changed);
        environmentSites.Clear();
    }
}
