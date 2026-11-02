using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class Binder
{
    private static TypeSymbol? LiteralVector(TypeSymbol target) => target.Kind == TypeKind.Array && target.Rank == 1 ? target.NonNullable :
        ReferenceCatalog.ArrayInterface(target) ? TypeSymbol.Array(target.TypeArguments[0]) : null;

    private bool AcceptsArrayLiteral(BoundArrayLiteral array, TypeSymbol target)
        => LiteralVector(target) is { } vector && array.Elements.All(element => Accepts(element, vector.Element!));

    private BoundArrayLiteral RetargetArrayLiteral(BoundArrayLiteral array, TypeSymbol target, int position)
    {
        var vector = LiteralVector(target)!;
        return new(vector, array.Elements.Select(element => Convert(element, vector.Element!, position)).ToArray());
    }

    private BoundExpression BindNewArray(NewArraySyntax syntax, bool check)
    {
        var element = BindType(syntax.Element);
        if (syntax.Lengths.Count is < 1 or > 32) throw source.Error("FP3002", "An array allocation requires between 1 and 32 lengths.", syntax.Position);
        var lengths = syntax.Lengths.Select(length => Convert(BindExpression(length, check), TypeSymbol.Int32, length.Position)).ToArray();
        if (element.CanHaveNull && !element.IsNullable && !lengths.Any(length => length is BoundInteger { Value: 0 }))
            throw source.Error("FP3013", "Default-null array elements require a nullable element type or a fully initialized literal.", syntax.Position);
        return new BoundNewArray(TypeSymbol.Array(element, lengths.Length), lengths);
    }

    private BoundExpression BindArrayLiteral(ArrayLiteralSyntax syntax, bool check, TypeSymbol? expected)
    {
        var target = expected is null ? null : LiteralVector(expected);
        var bound = target is not null ? syntax.Elements.Select(element => (BoundExpression?)BindExpression(element, check, target.Element)).ToArray() : BindInferenceValues(syntax.Elements, check);
        var elementType = target?.Element ?? bound.Where(element => element is not null).Select(element => element!.Type).AggregateOrDefault(references.Common);
        if (elementType is null || elementType.Kind is TypeKind.Unsupported or TypeKind.Void or TypeKind.Null)
            throw source.Error("FP3002", "An array literal needs a common element type; an empty literal needs an expected vector type.", syntax.Position);
        var elements = BindDeferredArguments(syntax.Elements, bound, syntax.Elements.Select(_ => elementType).ToArray(), check);
        return new BoundArrayLiteral(TypeSymbol.Array(elementType), elements.Select(element => Convert(element, elementType, syntax.Position)).ToArray());
    }

    private BoundExpression BindIndex(IndexSyntax syntax, bool check, bool writing = false)
    {
        var receiver = Receiver(syntax.Receiver, check);
        var indices = syntax.Indices.Select(index => BindExpression(index, check)).ToArray();
        if (receiver.Type.Kind == TypeKind.Array)
        {
            if (indices.Length != receiver.Type.Rank) throw source.Error("FP3002", "Array index count must match its rank.", syntax.Position);
            return new BoundArrayIndex(receiver, indices.Select(index => Convert(index, TypeSymbol.Int32, syntax.Position)).ToArray());
        }
        var property = references.Indexer(receiver.Type, indices.Select(index => index.Type).ToArray(), writing, source, syntax.Position);
        if (writing && receiver.Type.IsValue && !Writable(receiver)) throw source.Error("FP3009", "A value-type indexer assignment requires writable storage.", syntax.Position);
        var accessor = writing ? property.Setter! : property.Getter!;
        ForgetCapturedFacts();
        return new BoundProperty(property, receiver, ConvertArguments(indices, accessor.Parameters.Take(indices.Length).ToArray(), syntax.Position));
    }
}

internal static class TypeSequence
{
    internal static TypeSymbol? AggregateOrDefault(this IEnumerable<TypeSymbol> types, Func<TypeSymbol, TypeSymbol, TypeSymbol> common)
    {
        using var iterator = types.GetEnumerator();
        if (!iterator.MoveNext()) return null;
        var result = iterator.Current;
        while (iterator.MoveNext()) result = common(result, iterator.Current);
        return result;
    }
}
