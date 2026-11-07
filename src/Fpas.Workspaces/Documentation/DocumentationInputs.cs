using Fpas.Workspaces.Loading;
using System.Security.Cryptography;
using System.Text;

namespace Fpas.Workspaces.Documentation;

internal sealed class DocumentationInputs
{
    private readonly Dictionary<string, string> hashes = new(OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal);
    internal string SnapshotId { get; }
    internal DocumentationInputs(string contextId, IEnumerable<string> assemblies, CancellationToken token)
    {
        foreach (var assembly in assemblies)
        {
            token.ThrowIfCancellationRequested(); var path = Path.ChangeExtension(assembly, ".xml");
            if (!hashes.ContainsKey(path)) hashes.Add(path, Hash(path, token));
        }
        SnapshotId = Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(contextId + "\n" +
            string.Join('\n', hashes.OrderBy(pair => pair.Key, StringComparer.Ordinal).Select(pair => pair.Key + "=" + pair.Value)))));
    }
    internal bool Exists(string path) => hashes[path] != "missing";
    internal byte[] Read(string path, CancellationToken token)
    {
        token.ThrowIfCancellationRequested(); using var stream = File.OpenRead(path);
        if (stream.Length > 67_108_864) throw new ContextFailure("documentation_limit", "XML documentation exceeds its 64 MiB input limit: " + path);
        var bytes = new byte[(int)stream.Length]; stream.ReadExactly(bytes); token.ThrowIfCancellationRequested();
        if (stream.ReadByte() != -1 || Convert.ToHexString(SHA256.HashData(bytes)) != hashes[path])
            throw new ContextFailure("inputs_changed", "Documentation changed while reading: " + path);
        return bytes;
    }
    internal void Verify(CancellationToken token)
    {
        foreach (var (path, hash) in hashes)
            if (Hash(path, token) != hash) throw new ContextFailure("inputs_changed", "Documentation input changed: " + path);
    }
    private static string Hash(string path, CancellationToken token)
    {
        token.ThrowIfCancellationRequested();
        if (!File.Exists(path)) return "missing";
        using var stream = File.OpenRead(path); using var hash = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
        var buffer = new byte[65_536]; int count; long bytes = 0;
        while ((count = stream.Read(buffer)) != 0)
        {
            token.ThrowIfCancellationRequested();
            if ((bytes += count) > 67_108_864) throw new ContextFailure("documentation_limit", "XML documentation exceeds its 64 MiB input limit: " + path);
            hash.AppendData(buffer.AsSpan(0, count));
        }
        return Convert.ToHexString(hash.GetHashAndReset());
    }
}
