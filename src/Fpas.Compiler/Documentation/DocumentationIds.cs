using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;

namespace Fpas.Compiler.Documentation;

internal static class DocumentationIds
{
    internal static string Definition(TypeSymbol type)
    {
        if (type.Scalar is { } scalar) return scalar.Owner.Syntax.Name.Text + "." + scalar.Name;
        if (type.Interface is { } contract) return contract.Owner.Syntax.Name.Text + "." + contract.Name + Arity(contract.GenericParameters.Count);
        if (type.Record is { } record)
            return record.UnionCase is { } member ? Definition(member.Owner.Type) + "." + record.Name :
                record.Owner.Syntax.Name.Text + "." + record.Name + Arity(record.GenericParameters.Count);
        if (type.Union is { } union) return union.Owner.Syntax.Name.Text + "." + union.Name + Arity(union.GenericParameters.Count);
        return type.Definition!.FullName.Replace('+', '.');
    }

    internal static string Method(string unit, RoutineSymbol method)
    {
        var owner = method.InterfaceOwner is { } contract ? Definition(contract.Type) : method.Owner is { } record ? Definition(record.Type) : method.UnionOwner is { } union ? Definition(union.Type) : unit;
        var name = owner + "." + method.Name.Replace('.', '#') + (method.GenericParameters.Count == 0 ? "" : "``" + method.GenericParameters.Count);
        return "M:" + name + (method.Parameters.Count == 0 ? "" : "(" + string.Join(',', method.Parameters.Select(parameter => Type(parameter.SignatureType, method))) + ")");
    }

    private static string Arity(int count) => count == 0 ? "" : "`" + count;

    private static string Type(TypeSymbol type, RoutineSymbol method)
    {
        if (type.NullableValue) return "System.Nullable{" + Type(type.NonNullable, method) + "}";
        if (type.Kind == TypeKind.ByRef) return Type(type.Element!, method) + "@";
        if (type.Kind == TypeKind.Array) return Type(type.Element!, method) + (type.Rank == 1 ? "[]" : "[" + string.Join(',', Enumerable.Repeat("0:", type.Rank)) + "]");
        if (type.IsParameter)
        {
            var index = method.GenericParameters.ToList().IndexOf(type.Parameter!);
            return index >= 0 ? "``" + index : "`" + type.Parameter!.Index;
        }
        if (type.Kind is TypeKind.TypeParameter or TypeKind.MethodParameter)
            return (type.Kind == TypeKind.TypeParameter ? "`" : "``") + type.ParameterIndex;
        var primitive = TypeFacts.ClrName(type) ?? type.Kind switch
        {
            TypeKind.String => "System.String", TypeKind.Int32 => "System.Int32", TypeKind.Boolean => "System.Boolean",
            TypeKind.Double => "System.Double", TypeKind.Char => "System.Char", TypeKind.Object => "System.Object",
            TypeKind.NativeInt => "System.IntPtr", TypeKind.Void => "System.Void", _ => null
        };
        if (primitive is not null) return primitive;
        var name = Definition(type);
        if (type.TypeArguments.Count == 0) return name;
        var argument = 0;
        var segments = name.Split('.');
        for (var index = 0; index < segments.Length; index++)
        {
            var tick = segments[index].IndexOf('`');
            if (tick < 0) continue;
            var count = int.Parse(segments[index].AsSpan(tick + 1), System.Globalization.CultureInfo.InvariantCulture);
            segments[index] = segments[index][..tick] + "{" + string.Join(',', type.TypeArguments.Skip(argument).Take(count).Select(value => Type(value, method))) + "}";
            argument += count;
        }
        return string.Join('.', segments);
    }
}
