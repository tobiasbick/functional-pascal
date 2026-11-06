using System.Collections.Immutable;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using System.Security.Cryptography;

namespace Fpas.IntegrationTests.Debugging;

internal static class InvalidBreakpointSymbols
{
    // Build a recognizable PDB with an invalid IL location, so native binding really fails.
    internal static void Write(Debuggee program)
    {
        using var stream = File.OpenRead(program.Assembly);
        using var pe = new PEReader(stream);
        var metadata = pe.GetMetadataReader();
        var path = Path.ChangeExtension(program.Assembly, ".pdb");
        BlobContentId identity;
        using (var old = MetadataReaderProvider.FromPortablePdbStream(File.OpenRead(path)))
            identity = new(old.GetMetadataReader().DebugMetadataHeader!.Id);
        var builder = new MetadataBuilder();
        var document = builder.AddDocument(builder.GetOrAddDocumentName(program.Source),
            builder.GetOrAddGuid(new("8829d00f-11b8-4213-878b-770e8597ac16")),
            builder.GetOrAddBlob(SHA256.HashData(File.ReadAllBytes(program.Source))), default);
        var points = new BlobBuilder();
        points.WriteCompressedInteger(0); // No local signature is needed for binding.
        points.WriteCompressedInteger(0x1fffffff); // Deliberately outside every method body.
        points.WriteCompressedInteger(0); points.WriteCompressedInteger(1);
        points.WriteCompressedInteger(1); points.WriteCompressedInteger(1);
        var entryToken = pe.PEHeaders.CorHeader!.EntryPointTokenOrRelativeVirtualAddress;
        var entry = (MethodDefinitionHandle)MetadataTokens.Handle(entryToken);
        foreach (var method in metadata.MethodDefinitions)
            builder.AddMethodDebugInformation(method == entry ? document : default,
                method == entry ? builder.GetOrAddBlob(points) : default);
        var rows = new int[64]; rows[(int)TableIndex.MethodDef] = metadata.MethodDefinitions.Count;
        var pdb = new PortablePdbBuilder(builder, rows.ToImmutableArray(), entry, _ => identity);
        var image = new BlobBuilder(); pdb.Serialize(image);
        File.WriteAllBytes(path, image.ToArray());
    }
}
