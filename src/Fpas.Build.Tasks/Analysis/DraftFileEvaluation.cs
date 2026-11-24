// [Documentation](../../../docs/architecture/workspaces.md#context-preparation)
using Microsoft.Build.Definition;
using Microsoft.Build.Evaluation;
using Microsoft.Build.Evaluation.Context;
using Microsoft.Build.FileSystem;
using System.IO.Enumeration;

namespace Fpas.Build.Tasks.Analysis;

internal sealed class DraftFileEvaluation(IEnumerable<string> paths) : MSBuildFileSystemBase
{
    private static StringComparer Paths => OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
    private readonly HashSet<string> files = paths.ToHashSet(Paths);

    internal static string[] Sources(Project original, IReadOnlyDictionary<string, string> globals, IEnumerable<string> paths)
    {
        var filesystem = new DraftFileEvaluation(paths);
        using var collection = new ProjectCollection();
        var draft = Project.FromFile(original.FullPath, new ProjectOptions
        {
            ProjectCollection = collection,
            GlobalProperties = globals.ToDictionary(pair => pair.Key, pair => pair.Value, StringComparer.OrdinalIgnoreCase),
            EvaluationContext = EvaluationContext.Create(EvaluationContext.SharingPolicy.Shared, filesystem)
        });

        var originalFiles = Compile(original).Where(path => !filesystem.files.Contains(path)).Order(StringComparer.Ordinal);
        var proposed = Compile(draft);
        if (!originalFiles.SequenceEqual(proposed.Where(path => !filesystem.files.Contains(path)).Order(StringComparer.Ordinal), Paths) ||
            !original.Imports.Select(import => import.ImportedProject.FullPath).SequenceEqual(draft.Imports.Select(import => import.ImportedProject.FullPath), Paths) ||
            !Properties(original).SequenceEqual(Properties(draft)) ||
            !Items(original).SequenceEqual(Items(draft)))
            throw new InvalidOperationException("New draft files change evaluated settings or items beyond their own source membership.");
        return proposed.Where(filesystem.files.Contains).Distinct(Paths).Order(StringComparer.Ordinal).ToArray();

        static string[] Compile(Project project) => project.GetItems("Compile").Select(item => item.GetMetadataValue("FullPath")).ToArray();
        static IEnumerable<(string Name, string Value)> Properties(Project project) => project.AllEvaluatedProperties
            .Select(property => (property.Name, property.EvaluatedValue));
        IEnumerable<string> Items(Project project) => project.Items.Where(item => item.ItemType != "Compile" &&
            !(item.ItemType == "None" && filesystem.files.Contains(item.GetMetadataValue("FullPath"))))
            .Select(item => item.ItemType + "\0" + item.EvaluatedInclude + "\0" + string.Join('\0', item.Metadata.Select(metadata => metadata.Name + "=" + metadata.EvaluatedValue)));
    }

    public override bool FileExists(string path) => files.Contains(Path.GetFullPath(path)) || base.FileExists(path);
    public override bool FileOrDirectoryExists(string path) => files.Contains(Path.GetFullPath(path)) || base.FileOrDirectoryExists(path);
    public override FileAttributes GetAttributes(string path) => files.Contains(Path.GetFullPath(path)) ? FileAttributes.Normal : base.GetAttributes(path);

    public override IEnumerable<string> EnumerateFiles(string path, string searchPattern, SearchOption searchOption)
        => WithDrafts(base.EnumerateFiles(path, searchPattern, searchOption), path, searchPattern, searchOption);

    public override IEnumerable<string> EnumerateFileSystemEntries(string path, string searchPattern, SearchOption searchOption)
        => WithDrafts(base.EnumerateFileSystemEntries(path, searchPattern, searchOption), path, searchPattern, searchOption);

    private IEnumerable<string> WithDrafts(IEnumerable<string> entries, string directory, string pattern, SearchOption option)
    {
        var root = Path.TrimEndingDirectorySeparator(Path.GetFullPath(directory));
        var proposed = files.Where(file => (option == SearchOption.TopDirectoryOnly ? Paths.Equals(Path.GetDirectoryName(file), root) :
            file.StartsWith(Path.EndsInDirectorySeparator(root) ? root : root + Path.DirectorySeparatorChar,
                OperatingSystem.IsWindows() ? StringComparison.OrdinalIgnoreCase : StringComparison.Ordinal)) &&
            FileSystemName.MatchesSimpleExpression(pattern, Path.GetFileName(file), ignoreCase: OperatingSystem.IsWindows()));
        return entries.Concat(proposed).Distinct(Paths);
    }
}
