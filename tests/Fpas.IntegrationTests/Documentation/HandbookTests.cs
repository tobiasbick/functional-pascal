using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Documentation;

public sealed class HandbookTests
{
    private static string Root => Path.Combine(DocumentationLinks.Repository, "docs", "pascal");

    [Theory]
    [InlineData("true")]
    [InlineData("false")]
    public async Task Project_preparation_does_not_delete_staged_documentation(string designTime)
    {
        using var directory = new TemporaryDirectory();
        var fixture = directory.File("Fixtures/specification/probe.md");
        Directory.CreateDirectory(Path.GetDirectoryName(fixture)!); File.WriteAllText(fixture, "staged documentation");
        var project = Path.Combine(DocumentationLinks.Repository, "tests", "Fpas.IntegrationTests", "Fpas.IntegrationTests.csproj");
        var result = await ProcessRunner.RunAsync(["msbuild", project, "-nologo", "-target:PrepareForBuild", "-property:Configuration=Debug",
            "-property:DesignTimeBuild=" + designTime, "-property:OutputPath=" + directory.Path.Replace('\\', '/') + "/"], timeLimit: TimeSpan.FromSeconds(30));
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        Assert.True(File.Exists(fixture), "Project preparation removed staged documentation before any copy target ran.");
        Assert.Equal("staged documentation", File.ReadAllText(fixture));
    }

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
        var paths = new[] { "docs", "examples", "apps", "integrations", "sdk", "src", "tests", "third-party", ".agents", ".claude" }
            .Where(directory => Directory.Exists(Path.Combine(repository, directory))).SelectMany(directory =>
            Directory.GetFiles(Path.Combine(repository, directory), "*.md", SearchOption.AllDirectories))
            .Where(path => !Path.GetRelativePath(repository, path).Replace('\\', '/').Split('/').Any(part => part is "bin" or "obj" or "node_modules"))
            .Concat(new[] { "README.md", "CONTRIBUTING.md", "AGENTS.md" }.Select(path => Path.Combine(repository, path)));
        DocumentationLinks.Validate(paths);
    }

    [Fact]
    public void Pascal_fences_are_complete_sources_or_explicit_syntax_fragments()
    {
        foreach (var root in new[] { Root, Path.Combine(DocumentationLinks.Repository, "README.md") })
        {
            var examples = MarkdownExamples.Read(root);
            foreach (var fence in MarkdownExamples.ReadFences(root))
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
        Assert.Equal(File.ReadAllText(Path.Combine(DocumentationLinks.Repository, "README.md")),
            File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "Fixtures", "repository", "README.md")));
        Assert.Equal(File.ReadAllText(Path.Combine(DocumentationLinks.Repository, "docs", "build", "standalone-sdk.md")),
            File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "Fixtures", "standalone-sdk.md")));
    }
}
