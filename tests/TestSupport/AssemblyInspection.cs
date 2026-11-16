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
        var attribute = Attribute(reader, reader.GetAssemblyDefinition().GetCustomAttributes(), "DebuggableAttribute");
        var blob = reader.GetBlobReader(attribute.Value);
        if (blob.ReadUInt16() != 1) throw new BadImageFormatException("Invalid attribute prolog.");
        return (DebuggableAttribute.DebuggingModes)blob.ReadInt32();
    }

    /// <summary>Returns the single attribute whose referenced constructor type has the given name.</summary>
    /// <param name="reader">Metadata that owns the attributes.</param>
    /// <param name="attributes">Attributes of one metadata entity.</param>
    /// <param name="name">Simple attribute type name.</param>
    /// <returns>The matching attribute.</returns>
    internal static CustomAttribute Attribute(MetadataReader reader, IEnumerable<CustomAttributeHandle> attributes, string name)
        => attributes.Select(reader.GetCustomAttribute).Single(attribute =>
        {
            var constructor = reader.GetMemberReference((MemberReferenceHandle)attribute.Constructor);
            return reader.GetString(reader.GetTypeReference((TypeReferenceHandle)constructor.Parent).Name) == name;
        });
}
