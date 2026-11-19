// [Documentation](../../../docs/architecture/compiler.md#semantic-snapshots)
using System.Security.Cryptography;

namespace Fpas.Compiler.Analysis;

/// <summary>Immutable reference bytes associated with an explicit logical reference path.</summary>
public sealed class MetadataImage
{
    private readonly byte[] bytes;
    /// <summary>The full reference path these bytes stand for.</summary>
    public string Path { get; }
    /// <summary>The uppercase hexadecimal SHA-256 hash of the bytes.</summary>
    public string ContentHash { get; }
    /// <summary>The image size in bytes.</summary>
    public int Length => bytes.Length;

    /// <summary>Captures a copy of reference assembly bytes for a reference path.</summary>
    /// <param name="path">The reference path; it is normalized to a full path.</param>
    /// <param name="image">The assembly bytes to copy.</param>
    public MetadataImage(string path, ReadOnlySpan<byte> image)
    {
        Path = System.IO.Path.GetFullPath(path); bytes = image.ToArray();
        ContentHash = Convert.ToHexString(SHA256.HashData(bytes));
    }

    /// <summary>Opens a read-only stream over the captured bytes.</summary>
    /// <returns>A new stream positioned at the start.</returns>
    public Stream OpenRead() => new MemoryStream(bytes, writable: false);

    private MetadataImage(string path, MetadataImage image)
    { Path = System.IO.Path.GetFullPath(path); bytes = image.bytes; ContentHash = image.ContentHash; }

    /// <summary>Associates the same bytes with another reference path without copying them.</summary>
    /// <param name="path">The new reference path; it is normalized to a full path.</param>
    /// <returns>An image sharing these bytes.</returns>
    public MetadataImage WithPath(string path) => new(path, this);
}
