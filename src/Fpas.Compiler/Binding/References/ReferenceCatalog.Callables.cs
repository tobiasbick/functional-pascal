using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.References;

internal sealed partial class ReferenceCatalog
{
    internal TypeSymbol Callable(IReadOnlyList<TypeSymbol> parameters, TypeSymbol result, SourceText source, int position)
    {
        if (parameters.Any(type => type.Kind == TypeKind.ByRef))
            throw source.Error("FP3019", "A managed-reference callable requires an explicit imported delegate type.", position);
        if (parameters.Count > 16) throw source.Error("FP3019", "Callable shorthand supports at most 16 parameters; use an imported delegate for other signatures.", position);
        var arguments = result == TypeSymbol.Void ? parameters : parameters.Append(result).ToArray();
        var name = result == TypeSymbol.Void ? "System.Action" : "System.Func";
        if (arguments.Count == 0) return Symbol(Required(name));
        return Symbol(Required(name + "`" + arguments.Count)) with { Arguments = new(arguments) };
    }

    internal ImportedMethod? DelegateSignature(TypeSymbol type, bool allowOpen = false)
        => type.Kind == TypeKind.Named && !type.NullableValue && Derives(type, Symbol(Required("System.MulticastDelegate")))
            ? Methods(type.NonNullable, "Invoke", false, includeOpen: allowOpen).SingleOrDefault() : null;

    internal bool DelegateCompatible(TypeSymbol target, IReadOnlyList<TypeSymbol> parameters, TypeSymbol result, IReadOnlyList<ParameterSymbol>? info = null)
    {
        var invoke = DelegateSignature(target);
        bool Compatible(TypeSymbol from, TypeSymbol to) => from == to || from.IsReference && to.IsReference && CanAssign(from, to);
        return invoke is not null && invoke.Parameters.Count == parameters.Count &&
            (info is null || info.Select((parameter, index) => (parameter.ByRef ? parameter.Mode : "") == (invoke.Parameter(index).ByRef ? invoke.Parameter(index).Mode : "")).All(value => value)) &&
            invoke.DelegateParameters.Select((parameter, index) => Compatible(parameter, parameters[index])).All(value => value) && Compatible(result, invoke.Result);
    }

    internal ImportedMethod? DelegateMethod(TypeSymbol target, IReadOnlyList<ImportedMethod> methods)
    {
        var invoke = DelegateSignature(target);
        if (invoke is null) return null;
        var applicable = methods.Where(method => DelegateCompatible(target, method.DelegateParameters, method.Result, method.ParameterInfo)).ToArray();
        var best = applicable.Where(candidate => !applicable.Any(other => Better(other, candidate, invoke.Parameters))).ToArray();
        return best.Length == 1 ? best[0] : null;
    }

    internal TypeSymbol CellType(TypeSymbol element) => Symbol(Required("System.Runtime.CompilerServices.StrongBox`1")) with { Arguments = new([element]) };
    internal ImportedField CellValue(TypeSymbol element)
        => new(CellType(element), "Value", element, false, false, SignatureType: new(TypeKind.TypeParameter));
    internal ImportedMethod CellConstructor(TypeSymbol element)
        => new(CellType(element), ".ctor", TypeSymbol.Void, [element], false, SignatureParameters: [new(TypeKind.TypeParameter)]);
}
