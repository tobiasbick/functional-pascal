using System.Security.Cryptography;
using System.Text;
using System.Text.Json;

namespace Fpas.Build.Tasks.Incremental;

internal sealed record CompileFingerprint(string Inputs, string AssemblyHash, string? PdbHash = null, string? XmlHash = null,
    string[]? DocumentationFiles = null, JsonElement[]? Diagnostics = null)
{
    internal static string Compute(IEnumerable<string> sources, IEnumerable<string> references, string compiler, string options)
    {
        using var hash = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
        hash.AppendData(Encoding.UTF8.GetBytes(options + "\0" + Environment.Version + "\0"));
        var tools = Directory.GetFiles(Path.GetDirectoryName(compiler)!, "*")
            .Where(path => Path.GetExtension(path) is ".dll" or ".json")
            .Append(typeof(CompileFingerprint).Assembly.Location);
        foreach (var path in sources.Concat(references).Concat(tools).Select(Path.GetFullPath).Distinct(StringComparer.Ordinal).Order(StringComparer.Ordinal))
        {
            hash.AppendData(Encoding.UTF8.GetBytes(path + "\0"));
            using var stream = File.OpenRead(path);
            hash.AppendData(SHA256.HashData(stream));
        }
        return Convert.ToHexString(hash.GetHashAndReset());
    }

    internal static CompileFingerprint? Read(string cache)
    {
        if (!File.Exists(cache)) return null;
        try { return JsonSerializer.Deserialize<CompileFingerprint>(File.ReadAllText(cache)); }
        catch (JsonException) { return null; }
    }

    internal bool IsCurrent(string inputs, string assembly, string? symbols, IReadOnlyList<string> documents)
        => Inputs == inputs && File.Exists(assembly) && AssemblyHash == HashAssembly(assembly) &&
            (symbols is null ? !File.Exists(Path.ChangeExtension(assembly, ".pdb")) : File.Exists(symbols) && PdbHash == HashAssembly(symbols)) &&
            (documents.Count == 0 ? DocumentationFiles is not { Length: > 0 } : documents.All(path => File.Exists(path) && XmlHash == HashAssembly(path)));

    internal static void Save(string cache, string inputs, string assembly, string? symbols, string[] documents, JsonElement[] diagnostics)
    {
        var temporary = cache + "." + Guid.NewGuid().ToString("N") + ".tmp";
        try
        {
            File.WriteAllText(temporary, JsonSerializer.Serialize(new CompileFingerprint(inputs, HashAssembly(assembly), symbols is null ? null : HashAssembly(symbols),
                documents.Length == 0 ? null : HashAssembly(documents[0]), documents, diagnostics)));
            File.Move(temporary, cache, overwrite: true);
        }
        finally { File.Delete(temporary); }
    }

    private static string HashAssembly(string path)
    {
        using var stream = File.OpenRead(path);
        return Convert.ToHexString(SHA256.HashData(stream));
    }
}
