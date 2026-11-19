// [Documentation](../../../docs/pascal/agents/maintenance.md)

using System.Text.RegularExpressions;

namespace Fpas.IntegrationTests.Documentation;

public sealed class SourceWikiTests
{
    private static readonly string[] SourceRoots = ["src", "apps", "examples", "tests", "sdk"];
    private static readonly HashSet<string> SourceExtensions = new(StringComparer.Ordinal)
    {
        ".cs", ".fpas", ".c", ".h", ".ts", ".mjs", ".py", ".sh", ".ps1",
        ".csproj", ".fpasproj", ".slnx", ".targets", ".props", ".runsettings",
    };
    private static readonly HashSet<string> ExcludedDirectories = new(StringComparer.OrdinalIgnoreCase)
    {
        "third-party", "bin", "obj", "node_modules", ".vscode-test", ".git",
        ".temp-data", "artifacts", "CMakeFiles", "TestResults", "__pycache__",
    };
    private static readonly Regex DocumentationLink = new(@"\[Documentation\]\(([^)\s]+)\)", RegexOptions.CultureInvariant);
    private static readonly StringComparer Paths = OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;

    [Fact]
    public void Backlinks_are_rendered_links()
    {
        var page = DocumentationLinks.Read("""
            # Linked examples

            [`Example.cs`](Example.cs)
            [Reference][source]
            [Shortcut]

            [source]: <Two Words.cs>
            [shortcut]: Shortcut.cs
            [unused]: Unused.cs

            ![Image](Image.cs)
            `[Inline example](Inline.cs)`
            <!-- [Hidden](Hidden.cs) -->

            ````markdown
            ```
            # Hidden heading
            [Fenced](Fenced.cs)
            ````

            ~~~markdown
            [Fenced again](FencedAgain.cs)
            ~~~
            """);

        Assert.Equal(["Example.cs", "Two Words.cs", "Shortcut.cs"], page.Links);
        Assert.Equal("linked-examples", Assert.Single(page.Anchors));
    }

    [Fact]
    public void First_party_sources_and_their_documentation_link_to_each_other()
    {
        var repository = DocumentationLinks.Repository;
        var pages = new Dictionary<string, Page>(Paths);
        var errors = new List<string>();
        foreach (var source in Sources(repository).Order(StringComparer.Ordinal))
        {
            var relative = Path.GetRelativePath(repository, source).Replace('\\', '/');
            var link = ReadHeader(source);
            if (!link.Success)
            {
                errors.Add($"{relative}: missing a first-line [Documentation](relative/path.md) comment.");
                continue;
            }

            var href = link.Groups[1].Value;
            var target = Resolve(source, href);
            if (target is null || Path.GetRelativePath(repository, target).Split(Path.DirectorySeparatorChar)[0] == ".." ||
                !target.EndsWith(".md", StringComparison.OrdinalIgnoreCase) || !File.Exists(target))
            {
                errors.Add($"{relative}: documentation link '{href}' must resolve to a Markdown file in the repository.");
                continue;
            }

            if (!pages.TryGetValue(target, out var page))
                pages.Add(target, page = ReadPage(target));
            var anchor = href.Split('#', 2).Skip(1).SingleOrDefault();
            if (anchor is not null && !page.Anchors.Contains(Uri.UnescapeDataString(anchor)))
                errors.Add($"{relative}: documentation link '{href}' targets a missing anchor.");
            if (!page.SourceLinks.Contains(source))
                errors.Add($"{relative}: '{Path.GetRelativePath(repository, target)}' is missing a Markdown link back to this source.");
        }
        Assert.True(errors.Count == 0, string.Join(Environment.NewLine, errors));
    }

    private static Match ReadHeader(string source)
    {
        using var reader = File.OpenText(source);
        var line = reader.ReadLine() ?? "";
        var extension = Path.GetExtension(source);
        if (line.StartsWith("#!", StringComparison.Ordinal) || line.StartsWith("<?xml ", StringComparison.Ordinal))
            line = reader.ReadLine() ?? "";

        var comment = line.Trim();
        return extension switch
        {
            ".py" or ".sh" or ".ps1" => comment.StartsWith('#') ? DocumentationLink.Match(comment) : Match.Empty,
            ".csproj" or ".fpasproj" or ".slnx" or ".targets" or ".props" or ".runsettings" =>
                comment.StartsWith("<!--", StringComparison.Ordinal) && comment.EndsWith("-->", StringComparison.Ordinal)
                    ? DocumentationLink.Match(comment[4..^3]) : Match.Empty,
            _ when Path.GetFileName(source) == "CMakeLists.txt" => comment.StartsWith('#') ? DocumentationLink.Match(comment) : Match.Empty,
            _ => comment.StartsWith("//", StringComparison.Ordinal) ? DocumentationLink.Match(comment) : Match.Empty,
        };
    }

    private static IEnumerable<string> Sources(string repository)
    {
        var vscode = Path.Combine(repository, "src", "VSCode");
        var cui = Path.Combine(repository, "src", "Fpas.Ui.Gui.Cui", "native");
        var directories = new Stack<string>(SourceRoots.Select(root => Path.Combine(repository, root)));
        while (directories.TryPop(out var directory))
        {
            foreach (var path in Directory.EnumerateFiles(directory))
                if (SourceExtensions.Contains(Path.GetExtension(path)) || Path.GetFileName(path) == "CMakeLists.txt" ||
                    path.EndsWith(".fpas.txt", StringComparison.Ordinal))
                    yield return path;

            foreach (var child in Directory.EnumerateDirectories(directory))
            {
                var name = Path.GetFileName(child);
                if (ExcludedDirectories.Contains(name) || name.StartsWith("cmake-build-", StringComparison.Ordinal) ||
                    directory == vscode && (name is "out" or "dist") ||
                    directory == cui && name.StartsWith("build", StringComparison.Ordinal) ||
                    (File.GetAttributes(child) & FileAttributes.ReparsePoint) != 0)
                    continue;
                directories.Push(child);
            }
        }
        foreach (var path in Directory.EnumerateFiles(repository))
            if (Path.GetFileName(path) is "dist.sh" or "dist.ps1" || Path.GetExtension(path) is ".props" or ".slnx")
                yield return path;
    }

    private static Page ReadPage(string path)
    {
        var markdown = DocumentationLinks.Read(File.ReadAllText(path));
        var links = markdown.Links.Select(href => Resolve(path, href)).OfType<string>().ToHashSet(Paths);
        return new Page(markdown.Anchors, links);
    }

    private static string? Resolve(string origin, string href)
    {
        if (Uri.TryCreate(href, UriKind.Absolute, out _)) return null;
        var path = Uri.UnescapeDataString(href.Split('#', 2)[0]);
        return path.Length == 0 || Path.IsPathRooted(path) ? null : Path.GetFullPath(Path.Combine(Path.GetDirectoryName(origin)!, path));
    }

    private sealed record Page(IReadOnlySet<string> Anchors, HashSet<string> SourceLinks);
}
