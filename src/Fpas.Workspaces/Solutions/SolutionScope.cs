using Microsoft.VisualStudio.SolutionPersistence.Serializer;
using System.Xml.Linq;

namespace Fpas.Workspaces.Solutions;

internal sealed record ProjectSelection(string Path, string Configuration, string Platform);
internal sealed record SolutionScope(IReadOnlyList<ProjectSelection> Projects, string? ConfigurationXml, string? Directory)
{
    internal static async Task<SolutionScope> Read(string context, string configuration, CancellationToken token)
    {
        if (!Path.GetExtension(context).Equals(".slnx", StringComparison.OrdinalIgnoreCase))
            return new([new(context, configuration, "AnyCPU")], null, null);
        var solution = await SolutionSerializers.SlnXml.OpenAsync(context, token);
        var platform = solution.Platforms.FirstOrDefault(value => value.Replace(" ", "", StringComparison.Ordinal) == "AnyCPU")
            ?? (solution.Platforms.Count == 0 ? "Any CPU" : throw new Loading.ContextFailure("context_failed", "Select a solution with an Any CPU configuration."));
        if (solution.BuildTypes.Count != 0 && !solution.BuildTypes.Contains(configuration))
            throw new Loading.ContextFailure("invalid_argument", "The solution has no selected build configuration.");
        var directory = Path.GetDirectoryName(context)!;
        var selected = new List<ProjectSelection>();
        var mappings = new XElement("SolutionConfiguration");
        foreach (var project in solution.SolutionProjects)
        {
            var mapped = project.GetProjectConfiguration(configuration, platform);
            var path = Path.GetFullPath(project.FilePath.Replace('\\', Path.DirectorySeparatorChar), directory);
            var buildType = mapped.BuildType ?? configuration;
            var projectPlatform = mapped.Platform ?? "AnyCPU";
            mappings.Add(new XElement("ProjectConfiguration", new XAttribute("Project", project.Id.ToString("B")),
                new XAttribute("AbsolutePath", path), new XAttribute("BuildProjectInSolution", mapped.Build), buildType + "|" + projectPlatform));
            if (mapped.Build && Path.GetExtension(path).ToLowerInvariant() is ".fpasproj" or ".csproj" or ".fsproj") selected.Add(new(path, buildType, projectPlatform));
        }
        return new(selected, mappings.ToString(SaveOptions.DisableFormatting), directory + Path.DirectorySeparatorChar);
    }
}
