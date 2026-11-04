using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Bound;
namespace Fpas.Compiler.Binding.Symbols;

internal sealed class ClosureSymbol(int id, RecordSymbol? recordContext = null, IReadOnlyList<GenericParameterSymbol>? parameters = null)
{
    internal IReadOnlyList<GenericParameterSymbol> GenericParameters { get; } = parameters ?? [];
    internal int Id { get; } = id;
    internal RecordSymbol? RecordContext { get; } = recordContext;
    internal List<CaptureSymbol> Captures { get; } = [];
    internal BoundRoutine Body { get; set; } = null!;
    internal List<BoundRoutine> NestedBodies { get; } = [];
    internal IEnumerable<BoundRoutine> Bodies => NestedBodies.Count == 0 ? [Body] : NestedBodies;
    internal TypeSymbol Type => new(TypeKind.Generated, Closure: this);
}

internal sealed class CaptureSymbol(object original, TypeSymbol type, BoundExpression value, bool mutable, ClosureSymbol owner)
{
    internal ClosureSymbol Owner { get; } = owner;
    internal object Original { get; } = original;
    internal TypeSymbol Type { get; } = type;
    internal BoundExpression Value { get; } = value;
    internal bool Mutable { get; } = mutable;
    internal string FieldName { get; } = ChooseFieldName(original, owner);

    private static string ChooseFieldName(object original, ClosureSymbol owner)
    {
        var sourceName = original switch
        {
            LocalSymbol local => local.Name,
            ParameterSymbol parameter => parameter.Name,
            _ => throw new InvalidOperationException("A capture must refer to a source local or parameter.")
        };
        var used = owner.Captures.Select(capture => capture.FieldName).ToHashSet(StringComparer.Ordinal);
        var name = sourceName;
        for (var suffix = 1; used.Contains(name); suffix++)
            name = sourceName + "_" + suffix.ToString(System.Globalization.CultureInfo.InvariantCulture);
        return name;
    }
}
