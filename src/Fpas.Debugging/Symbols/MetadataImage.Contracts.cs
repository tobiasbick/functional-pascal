using System.Reflection.Metadata;

namespace Fpas.Debugging.Symbols;

internal partial class MetadataImage
{
    private BlobReader? Contract(CustomAttributeHandleCollection attributes, string name, byte[] signature)
    {
        BlobReader? result = null;
        foreach (var handle in attributes)
        {
            var attribute = Metadata.GetCustomAttribute(handle);
            if (attribute.Constructor.Kind != HandleKind.MemberReference) continue;
            var constructor = Metadata.GetMemberReference((MemberReferenceHandle)attribute.Constructor);
            if (constructor.Parent.Kind != HandleKind.TypeReference) continue;
            var type = Metadata.GetTypeReference((TypeReferenceHandle)constructor.Parent);
            if (!Metadata.StringComparer.Equals(type.Namespace, "Fpas.Core") || !Metadata.StringComparer.Equals(type.Name, name)) continue;
            var encoded = Metadata.GetBlobReader(constructor.Signature);
            if (result is not null || !Metadata.StringComparer.Equals(constructor.Name, ".ctor") || encoded.Length != signature.Length
                || !encoded.ReadBytes(signature.Length).SequenceEqual(signature) || type.ResolutionScope.Kind != HandleKind.AssemblyReference
                || !Metadata.StringComparer.Equals(Metadata.GetAssemblyReference((AssemblyReferenceHandle)type.ResolutionScope).Name, "Fpas.Core")) throw new BadImageFormatException("Invalid FPAS metadata marker.");
            var blob = Metadata.GetBlobReader(attribute.Value);
            if (blob.Length > 65536 || blob.ReadUInt16() != 1) throw new BadImageFormatException("Invalid FPAS metadata payload.");
            result = blob;
        }
        return result;
    }

    private static string ContractName(ref BlobReader blob)
    {
        var name = blob.ReadSerializedString();
        if (string.IsNullOrEmpty(name) || name.Length > 8192) throw new BadImageFormatException("Invalid FPAS metadata name.");
        return name;
    }

    private static void ContractEnd(ref BlobReader blob)
    {
        if (blob.ReadUInt16() != 0 || blob.RemainingBytes != 0) throw new BadImageFormatException("Unexpected FPAS metadata payload.");
    }
}
