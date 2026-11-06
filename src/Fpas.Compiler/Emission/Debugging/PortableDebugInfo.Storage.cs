using System.Reflection.Metadata;

namespace Fpas.Compiler.Emission.Debugging;

internal sealed partial class PortableDebugInfo
{
    internal void AddStorage(EntityHandle parent, string name, bool cell)
    {
        var blob = new BlobBuilder(); blob.WriteUInt32(1); blob.WriteByte(cell ? (byte)1 : (byte)0); blob.WriteSerializedString(name);
        metadata.AddCustomDebugInformation(parent, metadata.GetOrAddGuid(new("4C590C68-DC5F-4A9D-89A5-B8CE5B937137")), metadata.GetOrAddBlob(blob));
    }
}
