using System.Collections.Immutable;
using System.Reflection.Metadata;

namespace Fpas.Compiler.Binding;

internal sealed class SignatureTypes(ReferenceCatalog catalog, ReferenceAssembly assembly, bool includePrivate = false)
    : ISignatureTypeProvider<TypeSymbol, object?>
{
    public TypeSymbol GetPrimitiveType(PrimitiveTypeCode code) => code switch
    {
        PrimitiveTypeCode.Void => TypeSymbol.Void, PrimitiveTypeCode.String => TypeSymbol.String,
        PrimitiveTypeCode.Int32 => TypeSymbol.Int32, PrimitiveTypeCode.Boolean => TypeSymbol.Boolean,
        PrimitiveTypeCode.Double => TypeSymbol.Double, PrimitiveTypeCode.Char => TypeSymbol.Char, PrimitiveTypeCode.Object => TypeSymbol.Object,
        _ => TypeSymbol.Unsupported
    };
    public TypeSymbol GetGenericInstantiation(TypeSymbol genericType, ImmutableArray<TypeSymbol> arguments)
    {
        if (genericType == TypeSymbol.Unsupported || arguments.Any(argument => argument == TypeSymbol.Unsupported)) return TypeSymbol.Unsupported;
        if (genericType.Definition?.FullName == "System.Nullable`1" && arguments.Length == 1 && arguments[0].IsValue && !arguments[0].IsNullable)
            return arguments[0] with { IsNullable = true };
        return genericType.Definition is null ? TypeSymbol.Unsupported : genericType with { Arguments = new(arguments) };
    }
    public TypeSymbol GetGenericTypeParameter(object? context, int index) => new(TypeKind.TypeParameter, ParameterIndex: index);
    public TypeSymbol GetTypeFromDefinition(MetadataReader reader, TypeDefinitionHandle handle, byte kind)
        => catalog.FromDefinition(assembly, handle, includePrivate);
    public TypeSymbol GetTypeFromReference(MetadataReader reader, TypeReferenceHandle handle, byte kind)
        => catalog.FromReference(assembly, handle);
    public TypeSymbol GetTypeFromSpecification(MetadataReader reader, object? context, TypeSpecificationHandle handle, byte kind)
        => reader.GetTypeSpecification(handle).DecodeSignature(this, context);
    public TypeSymbol GetArrayType(TypeSymbol elementType, ArrayShape shape) => shape.Rank >= 2 && elementType != TypeSymbol.Unsupported ? TypeSymbol.Array(elementType, shape.Rank) : TypeSymbol.Unsupported;
    public TypeSymbol GetByReferenceType(TypeSymbol elementType) => elementType == TypeSymbol.Unsupported ? TypeSymbol.Unsupported : TypeSymbol.ByRef(elementType);
    public TypeSymbol GetFunctionPointerType(MethodSignature<TypeSymbol> signature) => TypeSymbol.Unsupported;
    public TypeSymbol GetGenericMethodParameter(object? context, int index) => new(TypeKind.MethodParameter, ParameterIndex: index);
    public TypeSymbol GetModifiedType(TypeSymbol modifier, TypeSymbol type, bool required)
        => modifier.Definition?.FullName is "System.Runtime.InteropServices.InAttribute" or "System.Runtime.CompilerServices.IsReadOnlyAttribute" && type.Kind == TypeKind.ByRef
            ? type with { Definition = modifier.Definition, ParameterIndex = required ? 1 : 0 } : !required ? type : TypeSymbol.Unsupported;
    public TypeSymbol GetPinnedType(TypeSymbol type) => TypeSymbol.Unsupported;
    public TypeSymbol GetPointerType(TypeSymbol elementType) => TypeSymbol.Unsupported;
    public TypeSymbol GetSZArrayType(TypeSymbol elementType) => elementType == TypeSymbol.Unsupported ? TypeSymbol.Unsupported : TypeSymbol.Array(elementType);
}
