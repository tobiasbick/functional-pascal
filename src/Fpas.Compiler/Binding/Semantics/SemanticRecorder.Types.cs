using Fpas.Compiler.Analysis;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Types;

namespace Fpas.Compiler.Binding.Semantics;

internal sealed partial class SemanticRecorder
{
    private string Type(TypeSymbol type)
    {
        token.ThrowIfCancellationRequested();
        if (typeIds.TryGetValue(type, out var existing)) return existing;
        var id = snapshotId + ":t" + typeIds.Count.ToString(System.Globalization.CultureInfo.InvariantCulture);
        typeIds.Add(type, id);
        var kind = type.Kind switch
        {
            TypeKind.Parameter or TypeKind.TypeParameter or TypeKind.MethodParameter => "parameter",
            TypeKind.Array => "array", TypeKind.ByRef => "byref", TypeKind.Null => "null", TypeKind.Void => "void",
            TypeKind.Record => "record", TypeKind.Variant => "variant", TypeKind.Scalar => "domain", TypeKind.Interface => "interface",
            TypeKind.Unsupported => "unresolved", _ => "named"
        };
        if (type.Kind == TypeKind.Named)
        {
            if (references.ScalarInfo(type) is { } scalar) kind = scalar.Category == Syntax.ScalarCategory.Enum ? "enum" : "domain";
            else if (references.RecordInfo(type) is not null) kind = "record";
            else if (references.VariantInfo(type) is not null) kind = "variant";
            else if (type.IsInterface) kind = "interface";
            else if (type.Definition?.IsEnum == true) kind = "enum";
        }
        if (type.Scalar?.Syntax.Category == Syntax.ScalarCategory.Enum) kind = "enum";
        var parameter = type.Parameter is { } generic ? Entry(generic, "type_parameter", generic.Name).Id : null;
        var arguments = Array.AsReadOnly((type.Arguments ?? TypeList.Empty).Select(Type).ToArray());
        var element = type.Element is null ? null : Type(type.Element);
        var metadata = type.Kind is TypeKind.Null or TypeKind.Void or TypeKind.Unsupported or TypeKind.Generated or TypeKind.Async or TypeKind.ByRef or TypeKind.Array ||
            type.IsParameter || type.Kind is TypeKind.TypeParameter or TypeKind.MethodParameter ? null : TypeMetadata(type);
        // Recursive delegate signatures may refer back to the same type ID.
        var invoke = type.Kind is TypeKind.Named or TypeKind.Container ? references.DelegateSignature(type) : null;
        var callable = invoke is null ? null : Signature(invoke);
        var genericIndex = type.IsParameter ? (int?)type.Parameter!.Index : type.Kind is TypeKind.TypeParameter or TypeKind.MethodParameter ? type.ParameterIndex : null;
        var genericOwner = type.IsParameter ? parameterKinds.GetValueOrDefault(type.Parameter!, "!") == "!!" ? "method" : "type" :
            type.Kind == TypeKind.MethodParameter ? "method" : type.Kind == TypeKind.TypeParameter ? "type" : null;
        types.Add(id, new(id, kind, TypeFacts.Display(type), type.IsNullable, type.Rank, element, arguments, parameter, metadata, callable, genericIndex, genericOwner,
            type.IsValue, type.IsReference, type.NeedsNullCheck || type.Kind == TypeKind.Null));
        return id;
    }

    private CallableSignature Signature(ImportedMethod method)
        => new(Array.AsReadOnly(method.Parameters.Select((type, index) => new SemanticParameter(method.Parameter(index).Name, Type(type), method.Parameter(index).Mode)).ToArray()),
            Type(method.Result), Array.AsReadOnly((method.MethodArguments ?? TypeList.Empty).Select(Type).ToArray()));

    private string Callable(ImportedMethod method)
    {
        var signature = Signature(method);
        var identity = method.SourceMethod is { } source ? Entry(source, "routine", source.Name).Id : Key(MethodMetadata(method));
        var key = "callable:" + identity + ":" + string.Join(',', signature.Parameters.Select(parameter => parameter.Type)) + ":" + signature.Result;
        if (types.ContainsKey(key)) return key;
        var display = "(" + string.Join("; ", method.Parameters.Select((type, index) => method.Parameter(index).Mode + " " + TypeFacts.Display(type))) + "): " + TypeFacts.Display(method.Result);
        types.Add(key, new(key, "callable", display, false, 0, null, Array.Empty<string>(), null, null, signature));
        return key;
    }
}
