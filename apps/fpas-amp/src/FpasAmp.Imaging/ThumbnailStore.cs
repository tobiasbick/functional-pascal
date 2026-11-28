// [Documentation](../../doc/imaging.md)
using FpasAmp.Media.Images;
using System.Security.Cryptography;
using System.Text;

namespace FpasAmp.Imaging;

/// <summary>Bounded, content-addressed RGBA thumbnails; call from background work.</summary>
public sealed class ThumbnailStore
{
    /// <summary>Thumbnail width/height, providing room for high-DPI album tiles.</summary>
    public const int Edge = 384;
    private const long DiskLimit = 256L * 1024 * 1024;
    private const long MemoryLimit = 32L * 1024 * 1024;
    private readonly string directory;
    private readonly object gate = new();
    private readonly Dictionary<string, LinkedListNode<(string Key, Bitmap Value)>> memory = new(StringComparer.Ordinal);
    private readonly LinkedList<(string Key, Bitmap Value)> order = new();
    private long memoryBytes;

    /// <summary>Creates a cache under an absolute, application-owned directory.</summary>
    /// <param name="directory">Thumbnail directory, outside the reconstructible catalogue.</param>
    public ThumbnailStore(string directory)
    {
        if (!Path.IsPathFullyQualified(directory)) throw new ArgumentException("Expected an absolute cache directory.", nameof(directory));
        this.directory = directory;
    }

    private static string Hash(byte[] value) => Convert.ToHexString(SHA256.HashData(value));
    private static string Hash(string value) => Hash(Encoding.UTF8.GetBytes(value));
    private static bool Digest(string value) => value.Length == 64 && value.All(Uri.IsHexDigit);
    private string Reference(string album) => Path.Combine(directory, Hash(album) + ".ref");

    /// <summary>Returns a valid thumbnail for the same album revision, including offline.</summary>
    /// <param name="album">Stable album identity.</param>
    /// <param name="revision">Observed audio/artwork freshness.</param>
    /// <returns>Cached pixels, or null when missing/stale/corrupt.</returns>
    public Bitmap? TryGet(string album, string revision)
    {
        lock (gate)
        {
            try
            {
                var path = Reference(album);
                if (!File.Exists(path) || new FileInfo(path).Length > 256) return null;
                var values = File.ReadAllText(path).Split('\n');
                if (values.Length < 2 || values[0] != Hash(revision) || !Digest(values[1])) return null;
                return Read(values[1]);
            }
            catch (IOException) { return null; }
            catch (UnauthorizedAccessException) { return null; }
        }
    }

    /// <summary>Decodes, or reuses, a shared content thumbnail and binds its album revision.</summary>
    /// <param name="album">Stable album identity.</param>
    /// <param name="revision">Observed source freshness.</param>
    /// <param name="encoded">Original encoded JPEG/PNG image, at most sixteen MiB.</param>
    /// <param name="cancellation">Checked before and after synchronous decoding.</param>
    /// <returns>Immutable-by-contract square RGBA pixels.</returns>
    public Bitmap Store(string album, string revision, byte[] encoded, CancellationToken cancellation)
    {
        cancellation.ThrowIfCancellationRequested();
        if (encoded.Length > 16 * 1024 * 1024) throw new InvalidDataException("Artwork exceeds sixteen MiB.");
        lock (gate)
        {
            cancellation.ThrowIfCancellationRequested();
            Directory.CreateDirectory(directory);
            string hash = Hash(encoded);
            string name = hash + ".rgba";
            string path = Path.Combine(directory, name);
            Bitmap? bitmap = Read(hash);
            if (bitmap is null || !ValidBlob(path))
            {
                bitmap ??= CoverDecoder.Decode(encoded, Edge, cancellation);
                var pixels = bitmap.Pixels;
                var bytes = new byte[pixels.Length + 8];
                BitConverter.TryWriteBytes(bytes.AsSpan(0, 4), Edge);
                BitConverter.TryWriteBytes(bytes.AsSpan(4, 4), Edge);
                pixels.CopyTo(bytes, 8);
                Atomic(path, bytes);
                Remember(hash, bitmap);
                Prune(name);
            }
            cancellation.ThrowIfCancellationRequested();
            Atomic(Reference(album), Encoding.UTF8.GetBytes(Hash(revision) + "\n" + hash));
            return bitmap;
        }
    }

    private Bitmap? Read(string hash)
    {
        if (memory.TryGetValue(hash, out var node))
        {
            order.Remove(node); order.AddFirst(node);
            return node.Value.Value;
        }
        var path = Path.Combine(directory, hash + ".rgba");
        try
        {
            byte[] pixels;
            using (var file = File.OpenRead(path))
            {
                if (!HasBlobHeader(file)) return null;
                pixels = new byte[Edge * Edge * 4];
                file.ReadExactly(pixels);
            }
            var bitmap = new Bitmap(Edge, Edge, pixels);
            File.SetLastWriteTimeUtc(path, DateTime.UtcNow);
            Remember(hash, bitmap);
            return bitmap;
        }
        catch (IOException) { return null; }
        catch (UnauthorizedAccessException) { return null; }
    }

    private static bool ValidBlob(string path)
    {
        try
        {
            using var file = File.OpenRead(path);
            return HasBlobHeader(file);
        }
        catch (IOException) { return false; }
        catch (UnauthorizedAccessException) { return false; }
    }

    private static bool HasBlobHeader(FileStream file)
    {
        if (file.Length != Edge * Edge * 4 + 8) return false;
        Span<byte> header = stackalloc byte[8];
        file.ReadExactly(header);
        return BitConverter.ToInt32(header[..4]) == Edge && BitConverter.ToInt32(header[4..]) == Edge;
    }

    private void Remember(string key, Bitmap bitmap)
    {
        if (memory.Remove(key, out var old)) { order.Remove(old); memoryBytes -= old.Value.Value.Pixels.Length; }
        memory[key] = order.AddFirst((key, bitmap));
        memoryBytes += bitmap.Pixels.Length;
        while (memoryBytes > MemoryLimit && order.Last is { } last)
        {
            order.RemoveLast(); memory.Remove(last.Value.Key);
            memoryBytes -= last.Value.Value.Pixels.Length;
        }
    }

    private void Prune(string retainedBlob)
    {
        var files = new DirectoryInfo(directory).GetFiles("*.rgba").OrderBy(file => file.LastWriteTimeUtc).ToArray();
        long bytes = files.Sum(file => file.Length);
        if (bytes <= DiskLimit) return;
        foreach (var file in files)
        {
            if (bytes <= DiskLimit) break;
            if (file.Name == retainedBlob) continue;
            bytes -= file.Length;
            file.Delete();
        }
        // References are tiny and reconstructible; discard those whose pixels were evicted.
        foreach (var file in new DirectoryInfo(directory).EnumerateFiles("*.ref"))
        {
            if (file.Length > 256) { file.Delete(); continue; }
            var value = File.ReadAllText(file.FullName).Split('\n');
            if (value.Length < 2 || !Digest(value[1]) || !File.Exists(Path.Combine(directory, value[1] + ".rgba"))) file.Delete();
        }
    }

    private static void Atomic(string path, byte[] bytes)
    {
        var temporary = path + "." + Guid.NewGuid().ToString("N") + ".tmp";
        try { File.WriteAllBytes(temporary, bytes); File.Move(temporary, path, true); }
        finally { File.Delete(temporary); }
    }
}
