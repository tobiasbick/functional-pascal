using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Bound;

namespace Fpas.Compiler.Binding.Symbols;

internal sealed class LocalSymbol(string name, TypeSymbol type, int slot, bool mutable = false)
{
    internal string Name { get; } = name;
    internal TypeSymbol Type { get; } = type;
    internal int Slot { get; } = slot;
    internal bool Mutable { get; } = mutable;
    internal bool Captured { get; set; }
    internal Syntax.DocumentationComment? Documentation { get; set; }
}
internal sealed record ParameterSymbol(string Name, TypeSymbol Type, int Index, string Mode = "")
{
    internal bool ByRef => Mode is "ref" or "out" or "in";
    internal TypeSymbol SignatureType => ByRef ? TypeSymbol.ByRef(Type) : Type;
    internal BoundExpression? Default { get; set; }
}
internal sealed record RoutineSymbol(string Name, TypeSymbol Result, IReadOnlyList<ParameterSymbol> Parameters,
    bool IsPublic, bool IsEntryPoint, int Index, bool IsInstance = false, RecordSymbol? Owner = null, GeneratedMethodKind Generated = GeneratedMethodKind.None, VariantSymbol? VariantOwner = null, IReadOnlyList<GenericParameterSymbol>? TypeParameters = null, Syntax.RoutineSyntax? Declaration = null, bool IsAsync = false, InterfaceSymbol? InterfaceOwner = null)
{
    internal IReadOnlyList<GenericParameterSymbol> GenericParameters => TypeParameters ?? [];
    internal TypeSymbol BodyResult => !IsAsync ? Result : Result.TypeArguments.Count == 1 ? Result.TypeArguments[0] : TypeSymbol.Void;
}
