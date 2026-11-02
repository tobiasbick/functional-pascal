using System.Reflection.Metadata;

namespace Fpas.Compiler.Binding;

internal static class NullabilityMetadata
{
    internal static TypeSymbol Parameter(TypeSymbol type, ImportedType owner, MethodDefinition method, int sequence)
    {
        var reader = owner.Assembly.Reader;
        byte[]? annotations = null;
        byte? overrideRoot = null;
        foreach (var handle in method.GetParameters())
        {
            var parameter = reader.GetParameter(handle);
            if (parameter.SequenceNumber != sequence) continue;
            var attributes = parameter.GetCustomAttributes();
            annotations = Read(reader, attributes, "NullableAttribute");
            var prefix = "System.Diagnostics.CodeAnalysis.";
            if (MetadataAttributes.Has(reader, attributes, prefix + (sequence == 0 ? "MaybeNullAttribute" : "AllowNullAttribute"))) overrideRoot = 2;
            if (MetadataAttributes.Has(reader, attributes, prefix + (sequence == 0 ? "NotNullAttribute" : "DisallowNullAttribute"))) overrideRoot = 1;
        }
        var context = Read(reader, method.GetCustomAttributes(), "NullableContextAttribute")?.FirstOrDefault() ?? Context(owner);
        return Annotate(type, annotations, context, overrideRoot);
    }

    internal static TypeSymbol Member(TypeSymbol type, ImportedType owner, CustomAttributeHandleCollection attributes)
    {
        var reader = owner.Assembly.Reader;
        byte? overrideRoot = null;
        if (MetadataAttributes.Has(reader, attributes, "System.Diagnostics.CodeAnalysis.MaybeNullAttribute")) overrideRoot = 2;
        if (MetadataAttributes.Has(reader, attributes, "System.Diagnostics.CodeAnalysis.NotNullAttribute")) overrideRoot = 1;
        return Annotate(type, Read(reader, attributes, "NullableAttribute"), Context(owner), overrideRoot);
    }

    internal static bool HasNullable(MetadataReader reader, CustomAttributeHandleCollection attributes)
        => Read(reader, attributes, "NullableAttribute") is not null;

    internal static bool GenericNotNull(ImportedType owner, GenericParameter parameter, MethodDefinition? method = null)
        => (Read(owner.Assembly.Reader, parameter.GetCustomAttributes(), "NullableAttribute")?.FirstOrDefault() ?? (method is { } declaration ? Read(owner.Assembly.Reader, declaration.GetCustomAttributes(), "NullableContextAttribute")?.FirstOrDefault() : null) ?? Context(owner)) == 1;

    private static byte Context(ImportedType owner)
    {
        for (ImportedType? type = owner; type is not null; type = type.DeclaringType)
            if (Read(type.Assembly.Reader, type.Definition.GetCustomAttributes(), "NullableContextAttribute") is [var context]) return context;
        var reader = owner.Assembly.Reader;
        return Read(reader, reader.GetModuleDefinition().GetCustomAttributes(), "NullableContextAttribute")?.FirstOrDefault() ?? 0;
    }

    private static TypeSymbol Annotate(TypeSymbol type, byte[]? flags, byte context, byte? overrideRoot)
    {
        var index = 0;
        TypeSymbol Visit(TypeSymbol current)
        {
            // Nullable<T> contributes its underlying value type's slot, not an
            // extra wrapper slot, even before generic substitution can fold it.
            if (current.Definition?.FullName == "System.Nullable`1" && current.Arguments is { Count: 1 })
                return current with { Arguments = new(current.Arguments.Select(Visit)) };
            if (current.IsReference || current.Kind is TypeKind.TypeParameter or TypeKind.MethodParameter || current.Arguments is { Count: > 0 })
            {
                var flag = flags is not null && index < flags.Length ? flags[index] : context;
                index++;
                if (current.IsReference) current = current with { IsNullable = flag != 1 };
                else if (current.Kind is TypeKind.TypeParameter or TypeKind.MethodParameter) current = current with { IsNullable = flag != 1 };
            }
            return current with
            {
                Element = current.Element is null ? null : Visit(current.Element),
                Arguments = current.Arguments is null ? null : new(current.Arguments.Select(Visit))
            };
        }
        var result = Visit(type);
        return overrideRoot is null || !result.IsReference && result.Kind is not (TypeKind.TypeParameter or TypeKind.MethodParameter) ? result : result with { IsNullable = overrideRoot == 2 };
    }

    private static byte[]? Read(MetadataReader reader, CustomAttributeHandleCollection attributes, string expected)
    {
        foreach (var handle in attributes)
        {
            var attribute = reader.GetCustomAttribute(handle);
            if (MetadataAttributes.Name(reader, attribute) != "System.Runtime.CompilerServices." + expected) continue;
            var signature = attribute.Constructor.Kind switch
            {
                HandleKind.MemberReference => reader.GetMemberReference((MemberReferenceHandle)attribute.Constructor).Signature,
                HandleKind.MethodDefinition => reader.GetMethodDefinition((MethodDefinitionHandle)attribute.Constructor).Signature,
                _ => default
            };
            if (signature.IsNil) return null;
            var constructorSignature = reader.GetBlobReader(signature);
            constructorSignature.ReadSignatureHeader();
            if (constructorSignature.ReadCompressedInteger() != 1 || constructorSignature.ReadByte() != 0x01) return null;
            var parameterType = constructorSignature.ReadByte();
            var value = reader.GetBlobReader(attribute.Value);
            if (value.ReadUInt16() != 1) return null;
            if (parameterType == 0x05) return [value.ReadByte()];
            if (parameterType == 0x1d && constructorSignature.ReadByte() == 0x05)
            {
                var length = value.ReadInt32();
                if (length >= 0 && length <= value.RemainingBytes - 2) return value.ReadBytes(length);
            }
        }
        return null;
    }
}
