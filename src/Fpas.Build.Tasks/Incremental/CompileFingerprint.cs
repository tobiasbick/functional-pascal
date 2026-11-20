// [Documentation](../../../docs/build/msbuild.md#sdk-task-and-incremental-state)
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using Microsoft.Build.Framework;

namespace Fpas.Build.Tasks.Incremental;

internal sealed record ResourceInput(string Name, string Path, string SourcePath)
{
    internal static ResourceInput[] Read(ITaskItem[] items)
        => items.Select(item => new ResourceInput(item.GetMetadata("LogicalName"), item.GetMetadata("FullPath"),
            System.IO.Path.GetFullPath(item.GetMetadata("SourcePath"))))
            .OrderBy(item => item.Name, StringComparer.Ordinal).ThenBy(item => item.Path, StringComparer.Ordinal)
            .ThenBy(item => item.SourcePath, StringComparer.Ordinal).ToArray();
}

internal sealed record CompileFingerprint(string Inputs, string AssemblyHash, string? PdbHash = null, string? XmlHash = null,
    string[]? DocumentationFiles = null, JsonElement[]? Diagnostics = null)
{
    internal static string Options(string target, string configuration, string framework, string output, string debug, bool optimize,
        IReadOnlyList<string> documents, bool warningsAsErrors, string promoted, string excluded, string suppressed)
        => $"{target}\0{configuration}\0{framework}\0{output}\0{debug}\0{optimize}\0{string.Join('\0', documents)}\0{warningsAsErrors}\0{promoted}\0{excluded}\0{suppressed}";

    internal static IEnumerable<string> Tools(string compiler)
    {
        var path = Path.GetFullPath(compiler);
        var siblings = path.EndsWith(".dll", StringComparison.OrdinalIgnoreCase)
            ? Directory.EnumerateFiles(Path.GetDirectoryName(path)!).Where(asset => Path.GetExtension(asset) is ".dll" or ".json")
            : [];
        return siblings.Append(path).Append(typeof(CompileFingerprint).Assembly.Location).Distinct(StringComparer.Ordinal);
    }

    internal static string Compute(IEnumerable<string> sources, IEnumerable<string> references, string compiler, string options, string compilerAssembly = "",
        IReadOnlyList<ResourceInput>? resources = null, WindowsResourceInput? windowsResources = null)
    {
        using var hash = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
        hash.AppendData(Encoding.UTF8.GetBytes(options + "\0" + Environment.Version + "\0" + Path.GetFullPath(compiler) + "\0"));
        AddPaths("sources", sources);
        AddPaths("references", references);
        AddPaths("tools", Tools(compiler));
        AddPaths("compiler assembly", compilerAssembly.Length == 0 ? [] : [compilerAssembly]);
        foreach (var resource in resources ?? [])
        {
            hash.AppendData(Encoding.UTF8.GetBytes("resource\0" + resource.Name + "\0"));
            AddPaths("source", [resource.SourcePath]);
            if (resource.Path == resource.SourcePath) continue;
            if (File.Exists(resource.Path)) AddPaths("compiled", [resource.Path]);
            else hash.AppendData(Encoding.UTF8.GetBytes("missing compiled\0" + resource.Path + "\0"));
        }
        if (windowsResources is not null)
        {
            hash.AppendData(Encoding.UTF8.GetBytes("Windows resources\0" + windowsResources.Descriptor + "\0"));
            if (windowsResources.Icon.Length != 0) AddPaths("Windows application icon", [windowsResources.Icon]);
        }
        return Convert.ToHexString(hash.GetHashAndReset());

        void AddPaths(string kind, IEnumerable<string> paths)
        {
            hash.AppendData(Encoding.UTF8.GetBytes(kind + "\0"));
            foreach (var path in paths.Select(Path.GetFullPath).Order(StringComparer.Ordinal))
            {
                hash.AppendData(Encoding.UTF8.GetBytes(path + "\0"));
                using var stream = File.OpenRead(path);
                hash.AppendData(SHA256.HashData(stream));
            }
        }
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
