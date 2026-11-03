using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Security.Cryptography;

namespace Fpas.Compiler.Binding.References;

internal sealed class ReferenceAssembly : IDisposable
{
    private readonly PEReader image;
    internal MetadataReader Reader { get; }
    internal AssemblyIdentity Identity { get; }

    internal ReferenceAssembly(string path)
    {
        image = new PEReader(new MemoryStream(File.ReadAllBytes(path), writable: false));
        try
        {
            if (!image.HasMetadata)
                throw new BadImageFormatException("The reference has no managed metadata.");
            Reader = image.GetMetadataReader();
            if (!Reader.IsAssembly)
                throw new BadImageFormatException("The reference is not an assembly.");
            var definition = Reader.GetAssemblyDefinition();
            var key = Reader.GetBlobBytes(definition.PublicKey);
            var token = key.Length == 0 ? [] : SHA1.HashData(key)[^8..].Reverse().ToArray();
            Identity = new(Reader.GetString(definition.Name), definition.Version,
                Reader.GetString(definition.Culture), token, definition.Flags & ~AssemblyFlags.PublicKey);
        }
        catch
        {
            image.Dispose();
            throw;
        }
    }

    public void Dispose() => image.Dispose();
}
