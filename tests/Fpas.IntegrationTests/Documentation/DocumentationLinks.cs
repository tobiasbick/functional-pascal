using System.Text.RegularExpressions;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Documentation;

internal static class DocumentationLinks
{
    internal static string Repository => Path.GetFullPath(Path.Combine(Path.GetDirectoryName(TestEnvironment.Metadata("FpasSdkProject"))!, "..", ".."));
    private static readonly Regex Links = new(@"\[[^\]]+\]\(([^)\s]+)\)", RegexOptions.CultureInvariant);

    internal static void Validate(IEnumerable<string> paths)
    {
        var errors = new List<string>();
        foreach (var path in paths)
            foreach (Match link in Links.Matches(File.ReadAllText(path)))
            {
                var href = link.Groups[1].Value;
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
                var contents = File.ReadAllText(target);
                var headings = Regex.Matches(contents, @"(?m)^#{1,6} (.+?)\r?$").Cast<Match>()
                    .Select(heading => Regex.Replace(Regex.Replace(heading.Groups[1].Value.ToLowerInvariant(), @"[^\p{L}\p{N}_\s-]", ""), @"\s", "-"));
                var explicitAnchors = Regex.Matches(contents, @"<a\b[^>]*\bid\s*=\s*[""']([^""']+)[""'][^>]*>", RegexOptions.IgnoreCase | RegexOptions.CultureInvariant)
                    .Cast<Match>().Select(anchor => anchor.Groups[1].Value);
                if (!headings.Concat(explicitAnchors).Contains(Uri.UnescapeDataString(pieces[1])))
                    errors.Add("Missing anchor: " + description);
            }
        Assert.True(errors.Count == 0, string.Join(Environment.NewLine, errors));
    }
}
