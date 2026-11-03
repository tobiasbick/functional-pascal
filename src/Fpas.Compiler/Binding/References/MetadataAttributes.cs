using System.Reflection.Metadata;

namespace Fpas.Compiler.Binding.References;

internal static class MetadataAttributes
{
    internal static BlobReader? Contract(MetadataReader reader, CustomAttributeHandleCollection attributes, string name, byte[] signature)
    {
        BlobReader? result = null;
        foreach (var handle in attributes)
        {
            var attribute = reader.GetCustomAttribute(handle);
            if (Name(reader, attribute) != "Fpas.Core." + name) continue;
            static BadImageFormatException Invalid() => new("Invalid FPAS metadata attribute contract.");
            if (result is not null || attribute.Constructor.Kind != HandleKind.MemberReference) throw Invalid();
            var constructor = reader.GetMemberReference((MemberReferenceHandle)attribute.Constructor);
            if (constructor.Parent.Kind != HandleKind.TypeReference || reader.GetString(constructor.Name) != ".ctor" ||
                !reader.GetBlobBytes(constructor.Signature).SequenceEqual(signature)) throw Invalid();
            var type = reader.GetTypeReference((TypeReferenceHandle)constructor.Parent);
            if (type.ResolutionScope.Kind != HandleKind.AssemblyReference ||
                reader.GetString(reader.GetAssemblyReference((AssemblyReferenceHandle)type.ResolutionScope).Name) != "Fpas.Core") throw Invalid();
            var blob = reader.GetBlobReader(attribute.Value);
            if (blob.ReadUInt16() != 1) throw Invalid();
            result = blob;
        }
        return result;
    }

    internal static bool Has(MetadataReader reader, CustomAttributeHandleCollection attributes, string fullName)
        => attributes.Any(handle => Name(reader, reader.GetCustomAttribute(handle)) == fullName);

    internal static string Name(MetadataReader reader, CustomAttribute attribute)
    {
        var owner = attribute.Constructor.Kind switch
        {
            HandleKind.MemberReference => reader.GetMemberReference((MemberReferenceHandle)attribute.Constructor).Parent,
            HandleKind.MethodDefinition => reader.GetMethodDefinition((MethodDefinitionHandle)attribute.Constructor).GetDeclaringType(),
            _ => default(EntityHandle)
        };
        if (owner.Kind == HandleKind.TypeReference)
        {
            var type = reader.GetTypeReference((TypeReferenceHandle)owner);
            return reader.GetString(type.Namespace) + "." + reader.GetString(type.Name);
        }
        if (owner.Kind == HandleKind.TypeDefinition && !owner.IsNil)
        {
            var type = reader.GetTypeDefinition((TypeDefinitionHandle)owner);
            return reader.GetString(type.Namespace) + "." + reader.GetString(type.Name);
        }
        return "";
    }
}
