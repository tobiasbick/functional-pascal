namespace Fpas.Compiler.Binding;

internal sealed class GenericParameterSymbol(string name, int index)
{
    internal string Name { get; } = name;
    internal int Index { get; } = index;
    internal string? Primary { get; set; }
    internal bool HasConstructor { get; set; }
    internal bool IsReference { get; set; }
    internal bool IsNotNull { get; set; }
    internal List<TypeSymbol> Constraints { get; } = [];
    internal TypeSymbol Type => new(TypeKind.Parameter, Parameter: this);
}
