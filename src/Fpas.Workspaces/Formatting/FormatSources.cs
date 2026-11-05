using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Processes;
using Fpas.Workspaces.Solutions;
using Microsoft.VisualStudio.SolutionPersistence.Model;
using System.Text.Json;

namespace Fpas.Workspaces.Formatting;

/// <summary>Selects explicit source ownership without compiling or preparing dependencies.</summary>
public static class FormatSources
{
    private static readonly HashSet<string> IgnoredDirectories = new([".git", ".temp-data", "bin", "obj", "node_modules"], StringComparer.OrdinalIgnoreCase);
    private static StringComparer Paths => OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;

    public static async Task<IReadOnlyList<string>> CollectAsync(IReadOnlyList<string> inputs, string configuration,
        CancellationToken cancellationToken = default)
    {
        if (inputs.Count == 0) throw new ContextFailure("invalid_argument", "Supply a .fpas file, directory, .fpasproj or .slnx path.");
        if (string.IsNullOrWhiteSpace(configuration)) throw new ContextFailure("invalid_argument", "Configuration must not be empty.");
        var files = new HashSet<string>(Paths);
        foreach (var input in inputs)
        {
            cancellationToken.ThrowIfCancellationRequested();
            var path = Path.GetFullPath(input);
            ValidateRegularPath(path);
            if (Directory.Exists(path)) CollectDirectory(path, files, cancellationToken);
            else if (Path.GetExtension(path).Equals(".fpas", StringComparison.OrdinalIgnoreCase)) AddFile(path, files);
            else if (Path.GetExtension(path).ToLowerInvariant() is ".fpasproj" or ".slnx")
                await CollectProjects(path, configuration, files, cancellationToken);
            else throw new ContextFailure("invalid_argument", "Expected a .fpas file, directory, .fpasproj or .slnx: " + path);
        }
        if (files.Count == 0) throw new ContextFailure("invalid_argument", "The selected inputs contain no FPAS sources.");
        return Array.AsReadOnly(files.Order(StringComparer.Ordinal).ToArray());
    }

    public static void ValidateRegularPath(string path)
    {
        for (var current = Path.GetFullPath(path); current is not null; current = Path.GetDirectoryName(current))
            if ((File.GetAttributes(current) & FileAttributes.ReparsePoint) != 0)
                throw new ContextFailure("invalid_argument", "Symbolic links and reparse points are not formatted: " + current);
    }

    private static void AddFile(string path, HashSet<string> files)
    {
        ValidateRegularPath(path);
        if (!File.Exists(path)) throw new IOException("Source file does not exist: " + path);
        files.Add(path);
    }

    private static void CollectDirectory(string directory, HashSet<string> files, CancellationToken token)
    {
        var pending = new Stack<string>(); pending.Push(directory);
        while (pending.TryPop(out var current))
        {
            token.ThrowIfCancellationRequested();
            foreach (var path in Directory.EnumerateFileSystemEntries(current))
            {
                token.ThrowIfCancellationRequested();
                var attributes = File.GetAttributes(path);
                if ((attributes & FileAttributes.ReparsePoint) != 0) continue;
                if ((attributes & FileAttributes.Directory) != 0)
                {
                    if (!IgnoredDirectories.Contains(Path.GetFileName(path))) pending.Push(path);
                }
                else if (Path.GetExtension(path).Equals(".fpas", StringComparison.OrdinalIgnoreCase)) files.Add(path);
            }
        }
    }

    private static async Task CollectProjects(string path, string configuration, HashSet<string> files, CancellationToken token)
    {
        SolutionScope scope;
        try { scope = await SolutionScope.Read(path, configuration, token); }
        catch (SolutionException exception) { throw new ContextFailure("invalid_argument", "Cannot read solution: " + exception.Message); }
        foreach (var project in scope.Projects.Where(project => Path.GetExtension(project.Path).Equals(".fpasproj", StringComparison.OrdinalIgnoreCase)))
        {
            ValidateRegularPath(project.Path);
            var args = new List<string> { "msbuild", project.Path, "-nologo", "-nr:false", "-getItem:Compile",
                "-p:Configuration=" + MsBuildSyntax.Escape(project.Configuration), "-p:Platform=" + MsBuildSyntax.Escape(project.Platform) };
            if (scope.ConfigurationXml is not null)
                args.AddRange(["-p:BuildingSolutionFile=true", "-p:SolutionDir=" + MsBuildSyntax.Escape(scope.Directory!),
                    "-p:CurrentSolutionConfigurationContents=" + MsBuildSyntax.Escape(scope.ConfigurationXml)]);
            var result = await BuildProcess.Run(Path.GetDirectoryName(project.Path)!, args, token);
            if (result.ExitCode != 0 || result.Truncated)
                throw new ContextFailure("evaluation_failed", "Cannot evaluate format inputs: " + result.Output + result.Error, result.Truncated);
            using var json = JsonDocument.Parse(result.Output);
            foreach (var item in json.RootElement.GetProperty("Items").GetProperty("Compile").EnumerateArray())
            {
                token.ThrowIfCancellationRequested();
                var source = Path.GetFullPath(item.GetProperty("FullPath").GetString()!, Path.GetDirectoryName(project.Path)!);
                if (Path.GetExtension(source).Equals(".fpas", StringComparison.OrdinalIgnoreCase)) AddFile(source, files);
            }
        }
    }
}
