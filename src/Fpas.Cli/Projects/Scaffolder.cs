using System.Text;

namespace Fpas.Cli.Projects;

internal static class Scaffolder
{
    internal static ProjectReport Create(string kind, string path, string? requestedName, string? sdkVersion, string? feed, string[] projects, bool dryRun)
    {
        var directory = Path.GetFullPath(path);
        var name = requestedName ?? Path.GetFileName(Path.TrimEndingDirectorySeparator(directory));
        var identifier = ProjectFiles.ValidateName(name);
        if (kind != "solution" && projects.Length != 0) throw new ArgumentException("--project belongs to init solution.");
        if (kind == "solution" && (sdkVersion is not null || feed is not null)) throw new ArgumentException("Select the SDK/feed on init project or init library, not on the solution container.");
        var files = new List<(string Path, string Content)>();
        if (kind == "solution")
        {
            var members = projects.Select(Path.GetFullPath).ToArray();
            if (members.Distinct(OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal).Count() != members.Length)
                throw new ArgumentException("Solution projects must be unique.");
            foreach (var project in members)
                if (!File.Exists(project) || Path.GetExtension(project).ToLowerInvariant() is not (".fpasproj" or ".csproj" or ".fsproj"))
                    throw new ArgumentException("Each --project must identify an existing .fpasproj, .csproj or .fsproj.");
            files.Add((Path.Combine(directory, name + ".slnx"), ProjectFiles.Solution(members, directory)));
        }
        else
        {
            files.Add((Path.Combine(directory, name + ".fpasproj"), ProjectFiles.Project(ToolSdk.ValidateVersion(sdkVersion), kind == "library")));
            files.Add((Path.Combine(directory, kind == "library" ? name + ".fpas" : "main.fpas"), kind == "library" ?
                $"unit {identifier};\n\nend unit;\n" :
                $"program {identifier};\n\nuses System.Console as Console;\n\nbegin\n  Console.WriteLine('Hello from FPAS');\nend;\nend program;\n"));
            if (feed is not null) files.Add((Path.Combine(directory, "NuGet.Config"), ProjectFiles.Feed(feed, directory)));
        }
        var plan = files.Select(file => new PlannedFile(file.Path, File.Exists(file.Path)
            ? Identical(file.Path, file.Content) ? "unchanged" : "conflict" : Directory.Exists(file.Path) ? "conflict" : "create", file.Content)).ToArray();
        if (plan.Any(file => file.State == "conflict"))
            return new("init " + kind, false, "conflict", directory) { DryRun = dryRun, Files = plan, Error = new("file_conflict", "Existing files differ from the template; no files were written.") };
        if (!dryRun) Publish(plan);
        return new("init " + kind, true, dryRun ? "preview" : "ok", directory)
            { DryRun = dryRun, Files = plan.Select(file => file with { State = file.State == "create" && !dryRun ? "created" : file.State }).ToArray() };
    }

    private static bool Identical(string path, string content)
    {
        var expected = Encoding.UTF8.GetBytes(content);
        return new FileInfo(path).Length == expected.Length && File.ReadAllBytes(path).AsSpan().SequenceEqual(expected);
    }

    private static void Publish(IReadOnlyList<PlannedFile> plan)
    {
        var created = new List<string>(); var directories = new Stack<string>();
        try
        {
            foreach (var file in plan.Where(file => file.State == "create"))
            {
                var missing = new Stack<string>(); var parent = Path.GetDirectoryName(file.Path)!;
                while (!Directory.Exists(parent)) { missing.Push(parent); parent = Path.GetDirectoryName(parent)!; }
                while (missing.TryPop(out var directory)) { Directory.CreateDirectory(directory); directories.Push(directory); }
                using var stream = new FileStream(file.Path, FileMode.CreateNew, FileAccess.Write, FileShare.None);
                created.Add(file.Path);
                using var writer = new StreamWriter(stream, new UTF8Encoding(false)); writer.Write(file.Content);
            }
        }
        catch
        {
            foreach (var file in created) File.Delete(file);
            while (directories.TryPop(out var directory)) if (!Directory.EnumerateFileSystemEntries(directory).Any()) Directory.Delete(directory);
            throw;
        }
    }
}
