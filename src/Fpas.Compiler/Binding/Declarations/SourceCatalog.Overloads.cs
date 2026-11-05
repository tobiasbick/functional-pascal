using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Declarations;

internal sealed partial class SourceCatalog
{
    internal static void AddOverload(Dictionary<string, List<RoutineSymbol>> groups, RoutineSymbol method, SourceText source, int position)
    {
        if (!groups.TryGetValue(method.Name, out var overloads)) groups.Add(method.Name, overloads = []);
        var key = OverloadKey(method);
        if (overloads.FirstOrDefault(other => OverloadKey(other) == key) is { } previous)
            throw source.Error("FP3004", $"Duplicate CLR signature for '{method.Name}'. Result types, labels, nullability, static/instance, and ref/out/in alone cannot distinguish overloads.", position,
                related: previous.Declaration is { } declaration ? [new("The earlier routine with this CLR signature is here.", source.Location(declaration.Name.Position, declaration.Name.Length))] : []);
        overloads.Add(method);
    }

    private static string OverloadKey(RoutineSymbol method)
    {
        string TypeKey(TypeSymbol type)
        {
            if (type.NullableValue) return "Nullable<" + TypeKey(type.NonNullable) + ">";
            if (type.IsParameter) return (method.GenericParameters.Contains(type.Parameter!) ? "!!" : "!") + type.Parameter!.Index;
            if (type.Kind == TypeKind.Array) return TypeKey(type.Element!) + "[" + new string(',', type.Rank - 1) + "]";
            var name = type.Interface is { } contract ? contract.Owner.Syntax.Name.Text + "." + contract.Name : type.Record is { } record ? record.Owner.Syntax.Name.Text + "." + record.Name : type.Union is { } union ? union.Owner.Syntax.Name.Text + "." + union.Name : type.Definition?.FullName ?? type.Kind.ToString();
            return name + (type.Arguments is null ? "" : "<" + string.Join(',', type.Arguments.Select(TypeKey)) + ">");
        }
        return method.GenericParameters.Count + ":" + string.Join(';', method.Parameters.Select(parameter => (parameter.ByRef ? "&" : "") + TypeKey(parameter.Type)));
    }
}
