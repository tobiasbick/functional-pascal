using System.Security.Cryptography;
using System.Text.Json;
using System.Text.Json.Serialization;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Processes;

namespace Fpas.Workspaces.Projects;

public sealed record PublishedFile(string Path, long Bytes, string Sha256);
internal sealed record PublishManifest(int Version, PublishedFile[] Files, string[]? EmptyDirectories = null);
public sealed record PublishResult(BuildResult Build, IReadOnlyList<PublishedFile> Files);

public static class PublishDirectory
{
    private const string Manifest = ".fpas-publish.json";

    public static async Task<PublishResult> Publish(ProjectOutput project, PublishOptions options, string destination, CancellationToken token,
        string? context = null, ProcessOutput? liveOutput = null)
    {
        options.Validate(); destination = Path.GetFullPath(destination);
        await Check(destination, token);
        var parent = Path.GetDirectoryName(destination) ?? throw new ArgumentException("Publish output cannot be a filesystem root.");
        Directory.CreateDirectory(parent);
        var stage = Path.Combine(parent, ".fpas-publish-" + Guid.NewGuid().ToString("N"));
        var backup = stage + ".previous";
        var committed = false;
        Directory.CreateDirectory(stage);
        try
        {
            var arguments = options.Arguments(project, stage).ToList();
            if (context is not null && Path.GetExtension(context).Equals(".slnx", StringComparison.OrdinalIgnoreCase))
            {
                var scope = await Solutions.SolutionScope.Read(context, options.Configuration, token);
                if (scope.ConfigurationXml is not null)
                    arguments.AddRange(["-p:BuildingSolutionFile=true", "-p:SolutionDir=" + MsBuildSyntax.Escape(scope.Directory!),
                        "-p:CurrentSolutionConfigurationContents=" + MsBuildSyntax.Escape(scope.ConfigurationXml)]);
            }
            var built = await BuildProcess.Run(Path.GetDirectoryName(project.Project)!, arguments, token, liveOutput: liveOutput);
            if (built.ExitCode != 0 || built.Truncated) return new(built, []);
            var stagedFiles = CheckedFiles(stage, token);
            foreach (var secret in new[] { options.Certificate, options.KeyFile }.OfType<string>())
            {
                var bytes = File.ReadAllBytes(secret);
                foreach (var file in stagedFiles)
                    if (File.Exists(file) && new FileInfo(file).Length == bytes.Length && File.ReadAllBytes(file).AsSpan().SequenceEqual(bytes)) File.Delete(file);
            }
            if (options.Certificate is not null)
            {
                Fpas.Deployment.PublishSignature.Sign(stage, options.Certificate, options.CertificatePasswordEnvironment ?? "");
            }
            var inventory = await Inventory(stage, token);
            if (inventory.Files.Length == 0) throw new IOException("Publish produced no files.");
            var manifest = JsonSerializer.SerializeToUtf8Bytes(inventory, PublishManifestJson.Default.PublishManifest);
            if (manifest.Length > 4 * 1024 * 1024) throw new IOException("Publish ownership manifest exceeds 4 MiB.");
            await File.WriteAllBytesAsync(Path.Combine(stage, Manifest), manifest, token);
            await Check(destination, token);
            token.ThrowIfCancellationRequested();
            if (Directory.Exists(destination)) Directory.Move(destination, backup);
            try { Directory.Move(stage, destination); committed = true; }
            catch { if (Directory.Exists(backup)) Directory.Move(backup, destination); throw; }
            return new(built, inventory.Files);
        }
        finally
        {
            DeleteStage(stage);
            if (committed && Directory.Exists(backup)) Directory.Delete(backup, recursive: true);
        }
    }

    private static void DeleteStage(string stage)
    {
        FileAttributes attributes;
        try { attributes = File.GetAttributes(stage); }
        catch (FileNotFoundException) { return; }
        catch (DirectoryNotFoundException) { return; }
        if ((attributes & FileAttributes.Directory) != 0)
            Directory.Delete(stage, recursive: (attributes & FileAttributes.ReparsePoint) == 0);
        else File.Delete(stage);
    }

    public static async Task Check(string destination, CancellationToken token)
    {
        if (File.Exists(destination)) throw new ArgumentException("Publish output must be a directory.");
        if (!Directory.Exists(destination)) return;
        if ((File.GetAttributes(destination) & FileAttributes.ReparsePoint) != 0) throw new ArgumentException("Publish output must not be a symbolic link.");
        if (!Directory.EnumerateFileSystemEntries(destination).Any()) return;
        var manifest = Path.Combine(destination, Manifest);
        if (!File.Exists(manifest) || new FileInfo(manifest).Length > 4 * 1024 * 1024)
            throw new ContextFailure("output_conflict", "Choose an empty output directory or an unchanged directory previously published by fpas.");
        var recorded = JsonSerializer.Deserialize(await File.ReadAllTextAsync(manifest, token), PublishManifestJson.Default.PublishManifest);
        var current = await Inventory(destination, token);
        if (recorded is not { Version: 1, Files: not null } || !recorded.Files.SequenceEqual(current.Files) ||
            !(recorded.EmptyDirectories ?? []).SequenceEqual(current.EmptyDirectories ?? []))
            throw new ContextFailure("output_conflict", "Publish output contains added, changed or missing entries; choose a new output directory.");
    }

    private static async Task<PublishManifest> Inventory(string root, CancellationToken token)
    {
        var files = new List<PublishedFile>();
        var emptyDirectories = new List<string>();
        foreach (var path in CheckedFiles(root, token, emptyDirectories))
        {
            if (path == Path.Combine(root, Manifest)) continue;
            await using var stream = File.OpenRead(path);
            files.Add(new(Path.GetRelativePath(root, path).Replace('\\', '/'), stream.Length, Convert.ToHexString(await SHA256.HashDataAsync(stream, token))));
        }
        return new(1, files.OrderBy(file => file.Path, StringComparer.Ordinal).ToArray(),
            emptyDirectories.Order(StringComparer.Ordinal).ToArray());
    }

    private static List<string> CheckedFiles(string root, CancellationToken token, List<string>? emptyDirectories = null)
    {
        if ((File.GetAttributes(root) & FileAttributes.ReparsePoint) != 0)
            throw new IOException("Publish directories must not contain symbolic links.");
        var files = new List<string>();
        var count = 0;
        var pending = new Queue<string>(); pending.Enqueue(root);
        while (pending.TryDequeue(out var folder))
        {
            var empty = true;
            foreach (var path in Directory.EnumerateFileSystemEntries(folder).Order(StringComparer.Ordinal))
            {
                empty = false;
                token.ThrowIfCancellationRequested();
                var attributes = File.GetAttributes(path);
                if ((attributes & FileAttributes.ReparsePoint) != 0) throw new IOException("Publish directories must not contain symbolic links.");
                if ((attributes & FileAttributes.Directory) != 0) pending.Enqueue(path);
                else
                {
                    if (path != Path.Combine(root, Manifest) && ++count > 20_000)
                        throw new IOException("Publish output exceeds 20000 files.");
                    files.Add(path);
                }
            }
            if (empty && folder != root) emptyDirectories?.Add(Path.GetRelativePath(root, folder).Replace('\\', '/'));
        }
        return files;
    }
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase)]
[JsonSerializable(typeof(PublishManifest))]
internal partial class PublishManifestJson : JsonSerializerContext;
