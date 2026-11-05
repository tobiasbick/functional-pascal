using System.Text;

namespace Fpas.Cli.Projects;

internal sealed class SourceProject
{
    internal string Project { get; }
    internal SourceProject(string source, string version, string? feed)
    {
        source = Path.GetFullPath(source);
        if (!File.Exists(source)) throw new ArgumentException("The source file does not exist.");
        var root = Path.GetDirectoryName(source)!;
        var name = Path.GetFileNameWithoutExtension(source);
        if (name.Length == 0 || name.Any(character => char.IsControl(character) || "<>:\"/\\|?*".Contains(character)))
            throw new ArgumentException("The source filename must also be a portable assembly filename.");
        var directory = Path.Combine(root, "obj", "fpas-cli", name);
        Project = Path.Combine(directory, "Source.fpasproj");
        Directory.CreateDirectory(directory);
        WriteIfDifferent(Project, ProjectFiles.Project(version, false, source, Path.Combine(root, "bin"), name));
        if (feed is not null) WriteIfDifferent(Path.Combine(directory, "NuGet.Config"), ProjectFiles.Feed(feed, directory));
        else if (File.Exists(Path.Combine(directory, "NuGet.Config"))) File.Delete(Path.Combine(directory, "NuGet.Config"));
    }

    private static void WriteIfDifferent(string path, string text)
    {
        if (!File.Exists(path) || File.ReadAllText(path) != text) File.WriteAllText(path, text, new UTF8Encoding(false));
    }
}
