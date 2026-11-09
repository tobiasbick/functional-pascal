using System.Diagnostics.CodeAnalysis;
using System.Reflection;
using System.Security.Cryptography;

namespace Fpas.Deployment;

/// <summary>Provides real files for metadata readers, MSBuild and native loaders that consume bundled assets.</summary>
public static class DistributionAssets
{
    private const string Prefix = "Fpas.Distribution.";

    /// <summary>Gets whether the entry assembly is loaded from a single-file executable.</summary>
    public static bool IsBundled => Location(Assembly.GetEntryAssembly()!).Length == 0;

    [UnconditionalSuppressMessage("SingleFile", "IL3000", Justification = "An empty location selects the embedded asset, never an adjacent assembly.")]
    private static string Location(Assembly assembly) => assembly.Location;

    /// <summary>Returns an assembly's file, extracting its embedded copy when bundled.</summary>
    /// <param name="assembly">The managed assembly whose file is needed.</param>
    /// <returns>The absolute path to the assembly file.</returns>
    public static string Managed(Assembly assembly)
        => Location(assembly) is { Length: > 0 } path ? path : Materialize(assembly.GetName().Name + ".dll");

    /// <summary>Returns an adjacent support file, extracting its embedded copy when bundled.</summary>
    /// <param name="owner">The assembly that owns the support file.</param>
    /// <param name="name">The support filename.</param>
    /// <returns>The absolute path to the support file.</returns>
    public static string Sidecar(Assembly owner, string name)
        => Location(owner) is { Length: > 0 } path ? Path.Combine(Path.GetDirectoryName(path)!, name) : Materialize(name);

    /// <summary>Extracts and verifies an embedded asset in the configured or executable-local cache.</summary>
    /// <param name="name">The embedded asset filename.</param>
    /// <returns>The absolute path to the verified asset in its SHA-256 directory.</returns>
    public static string Materialize(string name)
    {
        if (Path.GetFileName(name) != name) throw new ArgumentException("An asset name must be a filename.", nameof(name));
        using var resource = Assembly.GetEntryAssembly()!.GetManifestResourceStream(Prefix + name)
            ?? throw new IOException("This compiler does not contain the distribution asset " + name + ".");
        using var data = new MemoryStream(); resource.CopyTo(data);
        var bytes = data.ToArray(); var hash = Convert.ToHexString(SHA256.HashData(bytes));
        var root = Environment.GetEnvironmentVariable("FPAS_ASSET_CACHE");
        if (string.IsNullOrEmpty(root))
            root = Path.Combine(AppContext.BaseDirectory, "fpas-assets");
        if (!Path.IsPathFullyQualified(root)) throw new IOException("FPAS_ASSET_CACHE must be an absolute path.");
        var directory = Path.Combine(root, hash); var file = Path.Combine(directory, name);
        if (OperatingSystem.IsWindows()) Directory.CreateDirectory(directory);
        else Directory.CreateDirectory(directory, UnixFileMode.UserRead | UnixFileMode.UserWrite | UnixFileMode.UserExecute);
        if (Matches(file, hash)) return file;
        var temporary = Path.Combine(directory, "." + Guid.NewGuid().ToString("N") + ".tmp");
        try
        {
            File.WriteAllBytes(temporary, bytes);
            try { File.Move(temporary, file); }
            catch (IOException) when (File.Exists(file))
            {
                if (!Matches(file, hash))
                {
                    try { File.Move(temporary, file, overwrite: true); }
                    catch (Exception error) when ((error is IOException or UnauthorizedAccessException) && Matches(file, hash)) { }
                }
            }
            if (!Matches(file, hash)) throw new IOException("Distribution asset verification failed: " + name);
            return file;
        }
        finally { File.Delete(temporary); }
    }

    private static bool Matches(string file, string hash)
    {
        if (!File.Exists(file)) return false;
        using var input = new FileStream(file, FileMode.Open, FileAccess.Read, FileShare.Read | FileShare.Delete);
        return Convert.ToHexString(SHA256.HashData(input)) == hash;
    }
}
