using Fpas.Compiler.Analysis;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Types;

namespace Fpas.Compiler.Binding.Semantics;

internal sealed partial class SemanticRecorder
{
    private static string Key(MetadataIdentity value) => value.GetKey();

    internal MetadataIdentity EditorMethod(ImportedMethod method) => MethodMetadata(method);
    internal MetadataIdentity EditorField(ImportedField field) => Metadata(field.Owner, "field", field.Name);

    private MetadataIdentity Metadata(TypeSymbol owner, string kind, string name, int arity = 0, IEnumerable<string>? parameters = null, string? result = null)
    {
        var definition = owner.Definition;
        if (definition is null && owner.Container is null && owner.Record is null && owner.Union is null && owner.Scalar is null && owner.Interface is null)
            definition = references.Definition(owner.NonNullable);
        var assembly = definition?.Assembly.Identity;
        return new(assembly?.Name ?? assemblyName, assembly?.Version.ToString() ?? "1.0.0.0", assembly?.Culture ?? "",
            assembly is null ? "" : Convert.ToHexString(assembly.PublicKeyToken), TypeName(owner), kind, name, arity,
            Array.AsReadOnly((parameters ?? []).ToArray()), result, definition?.Assembly.ContentHash);
    }

    private MetadataIdentity TypeMetadata(TypeSymbol type) => Metadata(type, "type", TypeName(type),
        type.Definition?.Definition.GetGenericParameters().Count ?? type.Record?.GenericParameters.Count ?? type.Union?.GenericParameters.Count ?? type.Interface?.GenericParameters.Count ?? 0);

    private MetadataIdentity MethodMetadata(ImportedMethod method)
    {
        var source = method.SourceMethod;
        var parameters = source?.Parameters.Select(parameter => Clr(parameter.SignatureType, source)) ?? (method.SignatureParameters ?? method.Parameters).Select(type => Clr(type));
        return Metadata(method.Owner, "method", method.Name, source?.GenericParameters.Count ?? method.MethodArguments?.Count ?? 0,
            parameters, Clr(source?.Result ?? method.SignatureResult ?? method.Result, source));
    }

    private string TypeName(TypeSymbol type)
    {
        if (type.Definition is not null) return type.Definition.FullName;
        string Arity(int count) => count == 0 ? "" : "`" + count.ToString(System.Globalization.CultureInfo.InvariantCulture);
        if (type.Container is { } container) return container.Syntax.Name.Text;
        if (type.Record is { } record) return record.UnionCase is { } member
            ? TypeName(member.Owner.Type) + "+" + record.Name : record.Owner.Syntax.Name.Text + "+" + record.Name + Arity(record.GenericParameters.Count);
        if (type.Union is { } union) return union.Owner.Syntax.Name.Text + "+" + union.Name + Arity(union.GenericParameters.Count);
        if (type.Interface is { } contract) return contract.Owner.Syntax.Name.Text + "+" + contract.Name + Arity(contract.GenericParameters.Count);
        if (type.Scalar is { } scalar) return scalar.Owner.Syntax.Name.Text + "+" + scalar.Name;
        return references.Definition(type.NonNullable).FullName;
    }

    private string Clr(TypeSymbol type, RoutineSymbol? method = null)
    {
        if (type.NullableValue) return "System.Nullable`1<" + Clr(type.NonNullable, method) + ">";
        if (type.Kind == TypeKind.Array) return Clr(type.Element!, method) + "[" + new string(',', type.Rank - 1) + "]";
        if (type.Kind == TypeKind.ByRef) return Clr(type.Element!, method) + "&";
        if (type.Parameter is { } parameter)
        {
            var index = method?.GenericParameters.ToList().IndexOf(parameter) ?? -1;
            return index >= 0 ? "!!" + index : parameterKinds.GetValueOrDefault(parameter, "!") + parameter.Index;
        }
        if (type.Kind is TypeKind.TypeParameter or TypeKind.MethodParameter) return (type.Kind == TypeKind.MethodParameter ? "!!" : "!") + type.ParameterIndex;
        if (type == TypeSymbol.Void) return "System.Void";
        var name = TypeName(type);
        return name + (type.Arguments is { Count: > 0 } ? "<" + string.Join(',', type.Arguments.Select(argument => Clr(argument, method))) + ">" : "");
    }
}
