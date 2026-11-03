using System.Text.RegularExpressions;

namespace Fpas.IntegrationTests;

internal sealed record MarkdownExample(string Document, int Fence, string Name, bool Library, string? Project, string Source);

internal static class MarkdownExamples
{
    private static readonly Regex Fences = new(@"(?ms)^```pascal(?:[ \t]+(?<attributes>[^\r\n]*))?\r?\n(?<source>.*?)^```[ \t]*\r?$", RegexOptions.CultureInvariant);
    private static readonly Regex Header = new(@"(?m)^(?:async )?(program|unit) ([A-Za-z_][A-Za-z_0-9.]*)\s*;", RegexOptions.CultureInvariant);
    private static readonly Regex Project = new(@"(?:^|\s)project=([A-Za-z0-9_-]+)(?:\s|$)", RegexOptions.CultureInvariant);

    internal static IReadOnlyList<MarkdownExample> Read(string root)
    {
        var examples = new List<MarkdownExample>();
        foreach (var path in Directory.GetFiles(root, "*.md", SearchOption.AllDirectories).Order(StringComparer.Ordinal))
            foreach (var (match, index) in Fences.Matches(File.ReadAllText(path)).Cast<Match>().Select((match, index) => (match, index)))
            {
                var source = match.Groups["source"].Value;
                var header = Header.Match(source.Trim());
                if (!header.Success) continue;
                var project = Project.Match(match.Groups["attributes"].Value);
                examples.Add(new(Path.GetRelativePath(root, path), index, header.Groups[2].Value,
                    header.Groups[1].Value == "unit", project.Success ? project.Groups[1].Value : null, source));
            }
        return examples;
    }

    internal static IEnumerable<MarkdownExample> Sources(MarkdownExample example, IReadOnlyList<MarkdownExample> examples)
        => new[] { example }.Concat(example.Project is null ? [] : examples.Where(other => other != example && other.Library && other.Project == example.Project));
}
