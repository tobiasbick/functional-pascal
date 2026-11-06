using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using System.Security.Cryptography;

namespace Fpas.Compiler.Emission.Metadata;

internal sealed partial class AssemblyEmitter
{
    private static byte[] ContentIdentifiedImage(BlobBuilder image, GuidHandle identity)
    {
        var bytes = image.ToArray();
        using var pe = new PEReader(new MemoryStream(bytes, writable: false)); var reader = pe.GetMetadataReader();
        var offset = pe.PEHeaders.MetadataStartOffset + reader.GetHeapMetadataOffset(HeapIndex.Guid) + (MetadataTokens.GetHeapOffset(identity) - 1) * 16;
        var slot = bytes.AsSpan(offset, 16);
        // The identity covers emitted code, metadata and symbol linkage, with its own slot zeroed.
        slot.Clear(); var content = BlobContentId.FromHash(SHA256.HashData(bytes));
        content.Guid.TryWriteBytes(slot);
        return bytes;
    }
}
