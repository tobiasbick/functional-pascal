// [Documentation](../../docs/build/distribution.md#windows-portable-launcher)
using System.IO.Compression;
using System.Reflection;
using System.Security.Cryptography;
using System.Text;

namespace Fpas.Deployment.WindowsPortable;

internal static class Payload
{
    internal static string Extract(Assembly bundle, string cache)
    {
        using var manifest = Resource(bundle, "Manifest");
        var folder = Path.Combine(cache, Convert.ToHexString(SHA256.HashData(manifest)));
        manifest.Position = 0;
        using var reader = new StreamReader(manifest);
        var executable = reader.ReadLine() ?? throw new InvalidDataException("The bundle has no launch entry.");
        var gateName = Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(folder.ToUpperInvariant())));
        using var gate = new Mutex(false, "Fpas.Portable." + gateName);
        try { gate.WaitOne(); } catch (AbandonedMutexException) { }
        try
        {
            CreateDirectory(cache);
            CreateDirectory(folder);
            RestoreFiles(bundle, folder, reader);
            return Destination(folder, executable);
        }
        finally { gate.ReleaseMutex(); }
    }

    private static void RestoreFiles(Assembly bundle, string folder, StreamReader reader)
    {
        // Open and decompress the archive only if some file needs to be restored.
        ZipArchive? archive = null;
        try
        {
            while (reader.ReadLine() is { } line)
            {
                var hash = line[..64];
                var name = line[65..];
                var destination = Destination(folder, name);
                if (Matches(destination, hash)) continue;
                archive ??= new ZipArchive(Resource(bundle, "Archive"));
                var entry = archive.GetEntry(name) ?? throw new InvalidDataException("Missing bundled file: " + name);
                RestoreFile(entry, destination, hash);
            }
        }
        finally { archive?.Dispose(); }
    }

    private static void RestoreFile(ZipArchiveEntry entry, string destination, string hash)
    {
        var temporary = destination + "." + Guid.NewGuid().ToString("N") + ".tmp";
        try
        {
            entry.ExtractToFile(temporary);
            if (!Matches(temporary, hash))
                throw new InvalidDataException("Corrupt bundled file: " + entry.FullName);
            File.Move(temporary, destination, overwrite: true);
        }
        finally { File.Delete(temporary); }
    }

    private static string Destination(string folder, string name)
    {
        var path = Path.GetFullPath(Path.Combine(folder, name));
        if (!path.StartsWith(folder + Path.DirectorySeparatorChar, StringComparison.OrdinalIgnoreCase))
            throw new InvalidDataException("The bundle contains an invalid path.");

        var directory = folder;
        foreach (var component in Path.GetRelativePath(folder, Path.GetDirectoryName(path)!).Split(Path.DirectorySeparatorChar))
        {
            if (component == ".") continue;
            directory = Path.Combine(directory, component);
            CreateDirectory(directory);
        }
        return path;
    }

    private static void CreateDirectory(string directory)
    {
        if (Directory.CreateDirectory(directory).Attributes.HasFlag(FileAttributes.ReparsePoint))
            throw new IOException("Portable program directories cannot be redirected: " + directory);
    }

    private static bool Matches(string path, string hash)
    {
        try
        {
            if (File.GetAttributes(path).HasFlag(FileAttributes.ReparsePoint))
                throw new IOException("Portable program files cannot be redirected: " + path);
            using var file = File.OpenRead(path);
            return Convert.ToHexString(SHA256.HashData(file)) == hash;
        }
        catch (FileNotFoundException) { return false; }
    }

    private static Stream Resource(Assembly bundle, string name)
        => bundle.GetManifestResourceStream("Fpas.Portable." + name)
            ?? throw new InvalidDataException("The executable is missing its embedded " + name.ToLowerInvariant() + ".");
}
