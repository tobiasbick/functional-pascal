using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class Binder
{
    private BoundStatement BindForEach(ForEachSyntax syntax)
    {
        var type = BindType(syntax.Type);
        var value = Receiver(syntax.Source);
        if (!value.Type.IsReference && !value.Type.IsValue && !value.Type.IsParameter) throw source.Error("FP3002", "The source is not a supported enumerable.", syntax.Source.Position);
        InvalidateAssigned(syntax.Body);
        var incoming = nonNull.ToHashSet();
        scopes.Push(new(StringComparer.Ordinal));
        loopDepth++;
        try
        {
            var item = Allocate(syntax.Name.Text, type);
            Declare(item, syntax.Name.Position);
            var sourceLocal = Allocate("<Enumerable>", value.Type);
            var cached = new BoundLocal(sourceLocal);
            BoundStatement loop;
            if (value.Type == TypeSymbol.String || value.Type.Kind == TypeKind.Array && value.Type.Rank == 1)
            {
                var index = Allocate("<Index>", TypeSymbol.Int32);
                var counter = new BoundLocal(index);
                BoundExpression length = value.Type.Kind == TypeKind.Array ? new BoundArrayLength(cached) :
                    new BoundProperty(references.Property(TypeSymbol.String, "Length", false)!, cached);
                BoundExpression current = value.Type.Kind == TypeKind.Array ? new BoundArrayIndex(cached, [counter]) :
                    new BoundProperty(references.Indexer(TypeSymbol.String, [TypeSymbol.Int32], false, source, syntax.Position), cached, [counter]);
                var body = BindIteration(item, Convert(current, type, syntax.Position), syntax.Body);
                loop = new BoundFor(index, Allocate("<Limit>", TypeSymbol.Int32), new BoundInteger(0),
                    new BoundBinary(length, "-", new BoundInteger(1), TypeSymbol.Int32, true), false, body);
            }
            else loop = BindEnumerator(syntax, item, cached);
            return new BoundBlock([new BoundBinding(sourceLocal, value), loop]);
        }
        finally
        {
            loopDepth--;
            foreach (var local in scopes.Pop().Values) nonNull.Remove(local);
            SetFacts(incoming);
        }
    }

    private BoundBlock BindIteration(LocalSymbol item, BoundExpression current, IReadOnlyList<StatementSyntax> body)
        => new(new BoundStatement[] { new BoundBinding(item, current) }.Concat(BindBlock(body, newScope: false).Statements).ToArray());

    private BoundStatement BindEnumerator(ForEachSyntax syntax, LocalSymbol item, BoundExpression collection)
    {
        var views = references.EnumerableViews(collection.Type);
        var compatible = views.Where(view => references.CanAssign(view.TypeArguments[0], item.Type)).ToArray();
        TypeSymbol view;
        if (compatible.Length == 1) view = compatible[0];
        else if (views.Count > 0)
            throw source.Error("FP3002", "Enumeration requires a unique generic element type implicitly convertible to the loop binding.", syntax.Position);
        else
        {
            view = references.Symbol(references.Required("System.Collections.IEnumerable"));
            if (!references.CanAssign(collection.Type, view)) throw source.Error("FP3002", "The source is not a supported enumerable.", syntax.Source.Position);
        }
        var acquire = references.FindMethod(view, "GetEnumerator", [], syntax.Position, source, isStatic: false);
        var enumerator = Allocate("<Enumerator>", acquire.Result);
        var receiver = new BoundLocal(enumerator);
        var initializer = new BoundCall(acquire, [], Convert(collection, view, syntax.Position));
        var enumeratorBase = references.Symbol(references.Required("System.Collections.IEnumerator"));
        var moveNext = references.FindMethod(enumeratorBase, "MoveNext", [], syntax.Position, source, isStatic: false);
        var current = (BoundExpression)new BoundProperty(references.Property(acquire.Result, "Current", false)!, receiver);
        if (collection.Type.Kind == TypeKind.Array)
            current = new BoundConversion(current, collection.Type.Element!, true);
        var iteration = BindIteration(item, Convert(current, item.Type, syntax.Position), syntax.Body);
        var loop = new BoundWhile(new BoundCall(moveNext, [], Convert(receiver, enumeratorBase, syntax.Position)), iteration);
        var disposable = references.Symbol(references.Required("System.IDisposable"));
        var dispose = references.FindMethod(disposable, "Dispose", [], syntax.Position, source, isStatic: false);
        return new BoundUsing(enumerator, initializer, dispose, new([loop]), ConditionalDispose: views.Count == 0);
    }
}
