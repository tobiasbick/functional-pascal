using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

public sealed class HandbookTests
{
    private static string Root => Path.Combine(DocumentationLinks.Repository, "docs", "pascal");

    [Fact]
    public void Every_topic_directory_has_a_hub_and_articles_have_related_navigation()
    {
        foreach (var directory in Directory.GetDirectories(Root, "*", SearchOption.AllDirectories).Prepend(Root))
            Assert.True(File.Exists(Path.Combine(directory, "README.md")), "Missing topic hub: " + Path.GetRelativePath(Root, directory));
        foreach (var path in Directory.GetFiles(Root, "*.md", SearchOption.AllDirectories))
        {
            Assert.StartsWith("# ", File.ReadAllText(path));
            if (Path.GetFileName(path) == "README.md" || Path.GetRelativePath(Root, path).Replace('\\', '/').StartsWith("agents/", StringComparison.Ordinal)) continue;
            Assert.Contains("\n## See also\n", File.ReadAllText(path).Replace("\r\n", "\n", StringComparison.Ordinal));
        }
    }

    [Fact]
    public void Repository_documentation_links_and_anchors_resolve()
    {
        var repository = DocumentationLinks.Repository;
        var paths = new[] { "docs", "examples", "apps", "sdk", "tests" }.SelectMany(directory =>
            Directory.GetFiles(Path.Combine(repository, directory), "*.md", SearchOption.AllDirectories))
            .Where(path => !Path.GetRelativePath(repository, path).Replace('\\', '/').Split('/').Any(part => part is "bin" or "obj"))
            .Concat(new[] { "README.md", "AGENTS.md" }.Select(path => Path.Combine(repository, path)));
        DocumentationLinks.Validate(paths);
    }

    [Fact]
    public void Pascal_fences_are_complete_sources_or_explicit_syntax_fragments()
    {
        var examples = MarkdownExamples.Read(Root);
        foreach (var fence in MarkdownExamples.ReadFences(Root))
        {
            var complete = examples.SingleOrDefault(example => example.Document == fence.Document && example.Fence == fence.Index);
            if (complete is null) Assert.Contains("fragment", fence.Attributes.Split(' ', StringSplitOptions.RemoveEmptyEntries));
            else Assert.EndsWith(complete.Library ? "end unit;" : "end program;", complete.Source.Trim());
        }
        foreach (var project in examples.Where(example => example.Project is not null).GroupBy(example => example.Project))
        {
            Assert.Single(project, example => !example.Library);
            Assert.Contains(project, example => example.Library);
        }
    }

    [Fact]
    public void Copied_documentation_matches_current_sources_without_deleted_pages()
    {
        var copied = Path.Combine(AppContext.BaseDirectory, "Fixtures", "specification");
        static string[] Paths(string root) => Directory.GetFiles(root, "*.md", SearchOption.AllDirectories)
            .Select(path => Path.GetRelativePath(root, path)).Order(StringComparer.Ordinal).ToArray();
        var expected = Paths(Root);
        Assert.Equal(expected, Paths(copied));
        foreach (var path in expected) Assert.Equal(File.ReadAllText(Path.Combine(Root, path)), File.ReadAllText(Path.Combine(copied, path)));
    }
}
