using System.Diagnostics.CodeAnalysis;
using System.Reflection;
using System.Security.Cryptography;

namespace Fpas.Deployment;

// File consumers (metadata readers, MSBuild and native loaders) need real files even
// when the executable's managed assemblies are loaded directly from its bundle.
public static class DistributionAssets
{
    private const string Prefix = "Fpas.Distribution.";

    public static bool IsBundled => Location(Assembly.GetEntryAssembly()!).Length == 0;

    [UnconditionalSuppressMessage("SingleFile", "IL3000", Justification = "An empty location selects the embedded asset, never an adjacent assembly.")]
    private static string Location(Assembly assembly) => assembly.Location;

    public static string Managed(Assembly assembly)
        => Location(assembly) is { Length: > 0 } path ? path : Materialize(assembly.GetName().Name + ".dll");

    public static string Sidecar(Assembly owner, string name)
        => Location(owner) is { Length: > 0 } path ? Path.Combine(Path.GetDirectoryName(path)!, name) : Materialize(name);

    public static string Materialize(string name)
    {
        if (Path.GetFileName(name) != name) throw new ArgumentException("An asset name must be a filename.", nameof(name));
        using var resource = Assembly.GetEntryAssembly()!.GetManifestResourceStream(Prefix + name)
            ?? throw new IOException("This compiler does not contain the distribution asset " + name + ".");
        using var data = new MemoryStream(); resource.CopyTo(data);
        var bytes = data.ToArray(); var hash = Convert.ToHexString(SHA256.HashData(bytes));
        var root = Environment.GetEnvironmentVariable("FPAS_ASSET_CACHE");
        if (string.IsNullOrEmpty(root))
        {
            var home = OperatingSystem.IsWindows() ? Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData)
                : Environment.GetEnvironmentVariable("XDG_CACHE_HOME") is { Length: > 0 } xdg ? xdg
                : Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache");
            root = Path.Combine(home, "fpas", "assets");
        }
        if (!Path.IsPathFullyQualified(root)) throw new IOException("FPAS_ASSET_CACHE must be an absolute path.");
        var directory = Path.Combine(root, hash); var file = Path.Combine(directory, name);
        if (OperatingSystem.IsWindows()) Directory.CreateDirectory(directory);
        else Directory.CreateDirectory(directory, UnixFileMode.UserRead | UnixFileMode.UserWrite | UnixFileMode.UserExecute);
        if (Matches(file, hash)) return file;
        var temporary = Path.Combine(directory, "." + Guid.NewGuid().ToString("N") + ".tmp");
        try
        {
            File.WriteAllBytes(temporary, bytes);
            try { File.Move(temporary, file, overwrite: true); }
            catch (Exception error) when ((error is IOException or UnauthorizedAccessException) && Matches(file, hash)) { }
            if (!Matches(file, hash)) throw new IOException("Distribution asset verification failed: " + name);
            return file;
        }
        finally { File.Delete(temporary); }
    }

    private static bool Matches(string file, string hash)
    {
        if (!File.Exists(file)) return false;
        using var input = File.OpenRead(file);
        return Convert.ToHexString(SHA256.HashData(input)) == hash;
    }
}
