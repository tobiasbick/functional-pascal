// [Documentation](../../../docs/pascal/program-structure/project-commands.md#create-files)
using Fpas.Workspaces.Loading;
using System.Text;

namespace Fpas.Cli.Projects;

internal sealed class SourceProject
{
    internal string Project { get; }
    internal SourceProject(string source, string version, string? feed)
    {
        var (path, directory, name) = Locate(source);
        Project = Path.Combine(directory, "Source.fpasproj");
        Directory.CreateDirectory(directory);
        WriteIfDifferent(Project, ProjectFiles.Project(version, false, path, Path.Combine(Path.GetDirectoryName(path)!, "bin"), name));
        if (feed is not null) WriteIfDifferent(Path.Combine(directory, "NuGet.Config"), ProjectFiles.Feed(feed, directory));
        else if (File.Exists(Path.Combine(directory, "NuGet.Config"))) File.Delete(Path.Combine(directory, "NuGet.Config"));
    }

    internal static string ExistingProject(string source)
    {
        var (path, directory, _) = Locate(source);
        var project = Path.Combine(directory, "Source.fpasproj");
        if (!File.Exists(project))
            throw new ContextFailure("source_project_missing", "Build or run this source before inspecting its target.", inputPath: path);
        return project;
    }

    private static (string Path, string Directory, string Name) Locate(string source)
    {
        var path = Path.GetFullPath(source);
        if (!File.Exists(path)) throw new ContextFailure("invalid_argument", "The source file does not exist.", inputPath: path);
        var name = Path.GetFileNameWithoutExtension(path);
        if (name.Length == 0 || name.Any(character => char.IsControl(character) || "<>:\"/\\|?*".Contains(character)))
            throw new ContextFailure("invalid_argument", "The source filename must also be a portable assembly filename.", inputPath: path);
        return (path, Path.Combine(Path.GetDirectoryName(path)!, "obj", "fpas-cli", name), name);
    }

    private static void WriteIfDifferent(string path, string text)
    {
        if (!File.Exists(path) || File.ReadAllText(path) != text) File.WriteAllText(path, text, new UTF8Encoding(false));
    }
}
