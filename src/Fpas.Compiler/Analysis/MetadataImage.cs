using System.Security.Cryptography;

namespace Fpas.Compiler.Analysis;

/// <summary>Immutable reference bytes associated with an explicit logical reference path.</summary>
public sealed class MetadataImage
{
    private readonly byte[] bytes;
    public string Path { get; }
    public string ContentHash { get; }
    public int Length => bytes.Length;

    public MetadataImage(string path, ReadOnlySpan<byte> image)
    {
        Path = System.IO.Path.GetFullPath(path); bytes = image.ToArray();
        ContentHash = Convert.ToHexString(SHA256.HashData(bytes));
    }

    internal Stream OpenRead() => new MemoryStream(bytes, writable: false);

    private MetadataImage(string path, MetadataImage image)
    { Path = System.IO.Path.GetFullPath(path); bytes = image.bytes; ContentHash = image.ContentHash; }

    public MetadataImage WithPath(string path) => new(path, this);
}
