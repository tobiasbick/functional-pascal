using System.Diagnostics;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;

namespace Fpas.Tests.Support;

internal static class AssemblyInspection
{
    internal static DebuggableAttribute.DebuggingModes DebuggingModes(string path)
    {
        using var stream = File.OpenRead(path);
        using var pe = new PEReader(stream);
        var reader = pe.GetMetadataReader();
        var attribute = reader.GetAssemblyDefinition().GetCustomAttributes().Select(reader.GetCustomAttribute).Single(candidate =>
        {
            var constructor = reader.GetMemberReference((MemberReferenceHandle)candidate.Constructor);
            return reader.GetString(reader.GetTypeReference((TypeReferenceHandle)constructor.Parent).Name) == "DebuggableAttribute";
        });
        var blob = reader.GetBlobReader(attribute.Value);
        if (blob.ReadUInt16() != 1) throw new BadImageFormatException("Invalid attribute prolog.");
        return (DebuggableAttribute.DebuggingModes)blob.ReadInt32();
    }
}
