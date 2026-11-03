using System.Text.RegularExpressions;
using Microsoft.ML.Tokenizers;
using Fpas.Tests.Support;
using Xunit.Abstractions;

namespace Fpas.IntegrationTests;

public sealed class AgentReferenceTests(ITestOutputHelper output)
{
    private static string Repository => Path.GetFullPath(Path.Combine(Path.GetDirectoryName(TestEnvironment.Metadata("FpasSdkProject"))!, "..", ".."));
    private static string Root => Path.Combine(Repository, "docs", "pascal", "agents");
    private static readonly Regex Links = new(@"\[[^\]]+\]\(([^)\s]+)\)", RegexOptions.CultureInvariant);

    [Fact]
    public void Entrypoint_and_reference_respect_exact_o200k_base_budgets()
    {
        var tokenizer = TiktokenTokenizer.CreateForEncoding("o200k_base");
        var paths = Directory.GetFiles(Root, "*.md", SearchOption.AllDirectories).Order(StringComparer.Ordinal).ToArray();
        Assert.NotEmpty(paths);
        var total = 0;
        foreach (var path in paths)
        {
            var count = tokenizer.CountTokens(File.ReadAllText(path));
            output.WriteLine("{0}: {1} o200k_base tokens", Path.GetFileName(path), count);
            total += count;
            Assert.Equal(Root, Path.GetDirectoryName(path));
            if (Path.GetFileName(path) == "README.md") Assert.InRange(count, 1, 999);
        }
        output.WriteLine("Total: {0} o200k_base tokens", total);
        Assert.InRange(total, 1, 9_999);
    }

    [Fact]
    public void Repository_policy_has_one_entry_link_and_keeps_commands_in_documentation()
    {
        var policy = File.ReadAllText(Path.Combine(Repository, "AGENTS.md"));
        Assert.Equal("docs/pascal/agents/README.md", Assert.Single(Links.Matches(policy).Cast<Match>()).Groups[1].Value);
        Assert.DoesNotContain("```", policy);
        Assert.DoesNotContain("dotnet ", policy);
        Assert.DoesNotContain("cargo ", policy);
    }

    [Fact]
    public void Local_links_and_fragments_resolve_in_the_checkout()
    {
        var paths = Directory.GetFiles(Root, "*.md").Append(Path.Combine(Repository, "docs", "build", "development.md"));
        foreach (var path in paths)
            foreach (Match link in Links.Matches(File.ReadAllText(path)))
            {
                var href = link.Groups[1].Value;
                if (Uri.TryCreate(href, UriKind.Absolute, out var uri) && uri.Scheme is "http" or "https") continue;
                var pieces = href.Split('#', 2);
                var target = Path.GetFullPath(Path.Combine(Path.GetDirectoryName(path)!, Uri.UnescapeDataString(pieces[0])));
                Assert.True(File.Exists(target), Path.GetFileName(path) + ": missing target " + href);
                if (pieces.Length == 1) continue;
                var headings = Regex.Matches(File.ReadAllText(target), @"(?m)^#{1,6} (.+?)\r?$").Cast<Match>()
                    .Select(heading => Regex.Replace(Regex.Replace(heading.Groups[1].Value.ToLowerInvariant(), @"[^\p{L}\p{N}_\s-]", ""), @"\s", "-"));
                Assert.Contains(Uri.UnescapeDataString(pieces[1]), headings);
            }
    }

    [Fact]
    public void Every_agent_fence_is_a_complete_unique_example_with_explicit_companions()
    {
        var examples = MarkdownExamples.Read(Root);
        var count = Directory.GetFiles(Root, "*.md").Sum(path => Regex.Matches(File.ReadAllText(path), @"(?m)^```pascal\b").Count);
        Assert.NotEmpty(examples); Assert.Equal(count, examples.Count);
        Assert.Equal(examples.Count, examples.Select(example => example.Name).Distinct(StringComparer.Ordinal).Count());
        foreach (var example in examples)
            Assert.EndsWith(example.Library ? "end unit;" : "end program;", example.Source.Trim());
        foreach (var project in examples.Where(example => example.Project is not null).GroupBy(example => example.Project))
        {
            Assert.Single(project, example => !example.Library);
            Assert.Contains(project, example => example.Library);
        }
    }
}
