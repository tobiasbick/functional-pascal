// [Documentation](../../../docs/pascal/tools/documentation-examples.md)
using System.Text.RegularExpressions;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Documentation;

internal static class DocumentationLinks
{
    private const string Label = @"[^\[\]\r\n]+";
    private const string Destination = @"(?:<(?<path>[^>\r\n]+)>|(?<path>[^\s)<>]+))";
    private const string OptionalTitle = @"(?:[ \t]+(?:""[^""\r\n]*""|'[^'\r\n]*'|\([^()\r\n]*\)))?";
    private static readonly Regex Inline = new(@"(?<![!\\])(?<image>!)?\[" + Label + @"\]\([ \t]*" + Destination + OptionalTitle + @"[ \t]*\)", RegexOptions.CultureInvariant);
    private static readonly Regex Definition = new(@"(?m)^ {0,3}\[(?<label>" + Label + @")\]:[ \t]*" + Destination + OptionalTitle + @"[ \t]*$", RegexOptions.CultureInvariant);
    private static readonly Regex Reference = new(@"(?<![!\\])(?<image>!)?\[(?<label>" + Label + @")\](?:\[(?<reference>" + Label + @")?\]|(?![\[(]))", RegexOptions.CultureInvariant);
    private static readonly Regex Fence = new(@"^ {0,3}(?<delimiter>```+|~~~+)(?<tail>.*)$", RegexOptions.CultureInvariant);
    private static readonly Regex InlineCode = new(@"(?<!`)(?<ticks>`+)(?!`)[\s\S]*?(?<!`)\k<ticks>(?!`)", RegexOptions.CultureInvariant);
    private static readonly Regex Comments = new(@"<!--.*?(?:-->|\z)", RegexOptions.Singleline | RegexOptions.CultureInvariant);
    private static readonly Regex Headings = new(@"(?m)^ {0,3}#{1,6}[ \t]+(.+?)$", RegexOptions.CultureInvariant);
    private static readonly Regex ExplicitAnchors = new(@"<a\b[^>]*\bid\s*=\s*[""']([^""']+)[""'][^>]*>", RegexOptions.IgnoreCase | RegexOptions.CultureInvariant);

    internal static string Repository => Path.GetFullPath(Path.Combine(Path.GetDirectoryName(TestEnvironment.Metadata("FpasSdkProject"))!, "..", ".."));

    internal static void Validate(IEnumerable<string> paths)
    {
        var errors = new List<string>();
        var pages = new Dictionary<string, Page>(StringComparer.Ordinal);
        foreach (var path in paths)
            foreach (var href in PageAt(path).Links)
            {
                if (Uri.TryCreate(href, UriKind.Absolute, out var uri) && uri.Scheme is "http" or "https") continue;
                var pieces = href.Split('#', 2);
                var target = pieces[0].Length == 0 ? path : Path.GetFullPath(Path.Combine(Path.GetDirectoryName(path)!, Uri.UnescapeDataString(pieces[0])));
                var description = Path.GetRelativePath(Repository, path) + ": " + href;
                if (!File.Exists(target) && !Directory.Exists(target))
                {
                    errors.Add("Missing link target: " + description);
                    continue;
                }
                if (pieces.Length == 1) continue;
                if (!File.Exists(target))
                {
                    errors.Add("An anchor must target a file: " + description);
                    continue;
                }
                if (!PageAt(target).Anchors.Contains(Uri.UnescapeDataString(pieces[1])))
                    errors.Add("Missing anchor: " + description);
            }
        Assert.True(errors.Count == 0, string.Join(Environment.NewLine, errors));

        Page PageAt(string path)
        {
            if (!pages.TryGetValue(path, out var page))
                pages.Add(path, page = Read(File.ReadAllText(path), includeImages: true));
            return page;
        }
    }

    internal static Page Read(string markdown, bool includeImages = false)
    {
        var prose = Prose(markdown);
        var linkText = InlineCode.Replace(prose, "code");
        var definitions = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
        linkText = Definition.Replace(linkText, match =>
        {
            definitions.TryAdd(NormalizeLabel(match.Groups["label"].Value), match.Groups["path"].Value);
            return "";
        });
        var links = new List<string>();
        linkText = Inline.Replace(linkText, match =>
        {
            if (includeImages || !match.Groups["image"].Success)
                links.Add(match.Groups["path"].Value);
            return "";
        });
        foreach (Match match in Reference.Matches(linkText))
        {
            if (!includeImages && match.Groups["image"].Success) continue;
            var label = match.Groups["reference"].Success ? match.Groups["reference"].Value : match.Groups["label"].Value;
            if (definitions.TryGetValue(NormalizeLabel(label), out var path))
                links.Add(path);
        }

        var headings = Headings.Matches(prose)
            .Select(heading => Regex.Replace(Regex.Replace(heading.Groups[1].Value.Trim().ToLowerInvariant(), @"[^\p{L}\p{N}_\s-]", ""), @"\s", "-"));
        var anchors = ExplicitAnchors.Matches(prose).Select(anchor => anchor.Groups[1].Value);
        return new Page(links, headings.Concat(anchors).ToHashSet(StringComparer.Ordinal));
    }

    private static string NormalizeLabel(string label) => Regex.Replace(label.Trim(), @"\s+", " ");

    private static string Prose(string markdown)
    {
        using var reader = new StringReader(markdown);
        var lines = new List<string>();
        string? openFence = null;
        while (reader.ReadLine() is { } line)
        {
            var fence = Fence.Match(line);
            if (fence.Success)
            {
                var delimiter = fence.Groups["delimiter"].Value;
                if (openFence is null)
                {
                    openFence = delimiter;
                    continue;
                }
                if (delimiter[0] == openFence[0] && delimiter.Length >= openFence.Length && string.IsNullOrWhiteSpace(fence.Groups["tail"].Value))
                    openFence = null;
                continue;
            }
            if (openFence is null) lines.Add(line);
        }
        return Comments.Replace(string.Join('\n', lines), "");
    }

    internal sealed record Page(IReadOnlyList<string> Links, IReadOnlySet<string> Anchors);
}
