using System.Collections;

namespace Fpas.Compiler.Binding.Types;

internal sealed class TypeList(IEnumerable<TypeSymbol> items) : IReadOnlyList<TypeSymbol>, IEquatable<TypeList>
{
    private readonly TypeSymbol[] values = items.ToArray();
    internal static readonly TypeList Empty = new([]);
    public int Count => values.Length;
    public TypeSymbol this[int index] => values[index];
    public IEnumerator<TypeSymbol> GetEnumerator() => ((IEnumerable<TypeSymbol>)values).GetEnumerator();
    IEnumerator IEnumerable.GetEnumerator() => GetEnumerator();
    public bool Equals(TypeList? other) => other is not null && values.SequenceEqual(other.values);
    public override bool Equals(object? other) => other is TypeList list && Equals(list);
    public override int GetHashCode()
    {
        var hash = new HashCode();
        foreach (var value in values) hash.Add(value);
        return hash.ToHashCode();
    }
}
