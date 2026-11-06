using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;

namespace Fpas.Debugging.Symbols;

internal partial class MetadataImage : IDisposable
{
    private readonly Stream stream;
    protected PEReader Image { get; }
    internal MetadataReader Metadata { get; }
    internal string Path { get; }
    internal Guid Identity => Metadata.GetGuid(Metadata.GetModuleDefinition().Mvid);
    internal uint EntryToken => checked((uint)Image.PEHeaders.CorHeader!.EntryPointTokenOrRelativeVirtualAddress);

    internal MetadataImage(string path)
    {
        Path = System.IO.Path.GetFullPath(path);
        stream = new FileStream(Path, FileMode.Open, FileAccess.Read, FileShare.Read | FileShare.Delete);
        try { Image = new(stream); Metadata = Image.GetMetadataReader(); }
        catch { Image?.Dispose(); stream.Dispose(); throw; }
    }

    internal string TypeName(uint token) => TypeName(token, 0);

    private string TypeName(uint token, int depth)
    {
        if (depth >= 64) throw new Core.DebugException("limit_exceeded", "Metadata type nesting exceeds 64 levels.");
        var definition = Metadata.GetTypeDefinition((TypeDefinitionHandle)MetadataTokens.Handle((int)token));
        var owner = definition.GetDeclaringType(); var name = Metadata.GetString(definition.Name);
        var ns = Metadata.GetString(definition.Namespace);
        var result = !owner.IsNil ? TypeName((uint)MetadataTokens.GetToken(owner), depth + 1) + "+" + name : ns.Length == 0 ? name : ns + "." + name;
        if (result.Length > 8192) throw new Core.DebugException("limit_exceeded", "A metadata type name exceeds 8192 UTF-16 code units.");
        return result;
    }

    internal IEnumerable<(string Name, uint Token)> Fields(uint type)
    {
        foreach (var handle in Metadata.GetTypeDefinition((TypeDefinitionHandle)MetadataTokens.Handle((int)type)).GetFields())
            yield return (Metadata.GetString(Metadata.GetFieldDefinition(handle).Name), (uint)MetadataTokens.GetToken(handle));
    }

    public virtual void Dispose() { Image.Dispose(); stream.Dispose(); }
}
