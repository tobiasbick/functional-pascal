using System.Text.RegularExpressions;

namespace Fpas.IntegrationTests.Documentation;

internal sealed record MarkdownExample(string Document, int Fence, string Name, bool Library, string? Project, string Source);
internal sealed record MarkdownFence(string Document, int Index, string Attributes, string Source);

internal static class MarkdownExamples
{
    private static readonly Regex Fences = new(@"(?ms)^```pascal(?:[ \t]+(?<attributes>[^\r\n]*))?\r?\n(?<source>.*?)^```[ \t]*\r?$", RegexOptions.CultureInvariant);
    private static readonly Regex Header = new(@"(?m)^(?:async )?(program|unit) ([A-Za-z_][A-Za-z_0-9.]*)\s*;", RegexOptions.CultureInvariant);
    private static readonly Regex Project = new(@"(?:^|\s)project=([A-Za-z0-9_-]+)(?:\s|$)", RegexOptions.CultureInvariant);

    internal static IEnumerable<MarkdownFence> ReadFences(string root, string prefix = "")
    {
        var file = File.Exists(root);
        var paths = file ? [root] : Directory.GetFiles(root, "*.md", SearchOption.AllDirectories);
        foreach (var path in paths.Order(StringComparer.Ordinal))
            foreach (var (match, index) in Fences.Matches(File.ReadAllText(path)).Cast<Match>().Select((match, index) => (match, index)))
                yield return new(prefix + (file ? Path.GetFileName(path) : Path.GetRelativePath(root, path)),
                    index, match.Groups["attributes"].Value, match.Groups["source"].Value);
    }

    internal static IReadOnlyList<MarkdownExample> Read(string root, string prefix = "")
    {
        var examples = new List<MarkdownExample>();
        foreach (var fence in ReadFences(root, prefix))
        {
            var header = Header.Match(fence.Source.Trim());
            if (!header.Success) continue;
            var project = Project.Match(fence.Attributes);
            examples.Add(new(fence.Document, fence.Index, header.Groups[2].Value,
                header.Groups[1].Value == "unit", project.Success ? project.Groups[1].Value : null, fence.Source));
        }
        return examples;
    }

    internal static IEnumerable<MarkdownExample> Sources(MarkdownExample example, IReadOnlyList<MarkdownExample> examples)
        => new[] { example }.Concat(example.Project is null ? [] : examples.Where(other => other != example && other.Library && other.Project == example.Project));
}
