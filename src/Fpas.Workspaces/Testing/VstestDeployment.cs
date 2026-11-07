using System.Security.Cryptography;
using Fpas.Workspaces.Loading;

namespace Fpas.Workspaces.Testing;

// The CoreCLR VSTest host discovers adapters beside its source assembly.
internal static class VstestDeployment
{
    internal static async Task<string> Prepare(TestProject project, string directory, TestRunOptions? options, CancellationToken token)
    {
        var source = Path.GetDirectoryName(project.Assembly)!;
        var comparer = OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
        var adapters = project.VstestExtensionPaths.Concat(options?.AdapterPaths ?? []).Distinct(comparer)
            .Where(path => !comparer.Equals(Path.TrimEndingDirectorySeparator(path), Path.TrimEndingDirectorySeparator(source)) &&
                Directory.Exists(path) && Directory.EnumerateFiles(path, "*TestAdapter.dll").Any()).ToArray();
        if (adapters.Length == 0) return project.Assembly;
        var destination = Path.Combine(directory, "subject");
        foreach (var root in adapters.Prepend(source))
        {
            var relative = Path.GetRelativePath(root, destination);
            if (relative != ".." && !relative.StartsWith(".." + Path.DirectorySeparatorChar, StringComparison.Ordinal) && !Path.IsPathFullyQualified(relative))
                throw new ContextFailure("invalid_argument", "External adapters require a results directory outside the test output and adapter directories.");
        }
        var count = 0; long bytes = 0;
        async Task Copy(string from, string to, int depth = 0)
        {
            if (depth > 32) throw new ContextFailure("extension_limit", "Test/adapter deployment exceeds 32 directory levels.");
            token.ThrowIfCancellationRequested(); Directory.CreateDirectory(to);
            foreach (var entry in Directory.EnumerateFileSystemEntries(from))
            {
                token.ThrowIfCancellationRequested();
                if (++count > 10000) throw new ContextFailure("extension_limit", "Test/adapter deployment exceeds 10000 entries.");
                var attributes = File.GetAttributes(entry);
                if ((attributes & FileAttributes.ReparsePoint) != 0) throw new ContextFailure("extension_failed", "Test deployment cannot follow symbolic links or reparse points.");
                var target = Path.Combine(to, Path.GetFileName(entry));
                if ((attributes & FileAttributes.Directory) != 0) { await Copy(entry, target, depth + 1); continue; }
                if ((bytes += new FileInfo(entry).Length) > 536_870_912) throw new ContextFailure("extension_limit", "Test/adapter deployment exceeds 512 MiB.");
                await using var input = new FileStream(entry, FileMode.Open, FileAccess.Read, FileShare.Read);
                if (File.Exists(target))
                {
                    await using var previous = File.OpenRead(target);
                    var currentHash = await SHA256.HashDataAsync(input, token);
                    var previousHash = await SHA256.HashDataAsync(previous, token);
                    if (input.Length != previous.Length || !currentHash.AsSpan().SequenceEqual(previousHash))
                        throw new ContextFailure("extension_conflict", "Test output and adapter dependencies conflict at " + Path.GetFileName(target) + ". Align their versions.");
                    continue;
                }
                await using var output = new FileStream(target, FileMode.CreateNew, FileAccess.Write, FileShare.None);
                await input.CopyToAsync(output, token);
                if (!OperatingSystem.IsWindows()) File.SetUnixFileMode(target, File.GetUnixFileMode(entry));
            }
        }
        await Copy(source, destination);
        foreach (var adapter in adapters) await Copy(adapter, destination);
        return Path.Combine(destination, Path.GetFileName(project.Assembly));
    }
}
