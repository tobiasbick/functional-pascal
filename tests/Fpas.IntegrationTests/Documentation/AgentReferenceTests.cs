// [Documentation](../../../docs/pascal/agents/maintenance.md)
using System.Text.RegularExpressions;
using Microsoft.ML.Tokenizers;
using Fpas.Tests.Support;
using Xunit.Abstractions;

namespace Fpas.IntegrationTests.Documentation;

public sealed class AgentReferenceTests(ITestOutputHelper output)
{
    private static string Repository => DocumentationLinks.Repository;
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
        Assert.InRange(total, 1, 13_000);
    }

    [Fact]
    public async Task Cli_index_matches_the_public_command_tree()
    {
        var index = File.ReadAllText(Path.Combine(Root, "cli.md"));
        var documented = Regex.Matches(index, @"(?m)^\| `fpas ([a-z][a-z0-9-]*(?: [a-z][a-z0-9-]*)*)` \|")
            .Select(match => match.Groups[1].Value).Order(StringComparer.Ordinal);
        var cli = TestEnvironment.Metadata("FpasCliPath");
        var registered = new List<string>();
        await ReadCommands([]);
        Assert.Equal(registered.Order(StringComparer.Ordinal), documented);
        output.WriteLine("Public CLI commands and subcommands: {0}", registered.Count);

        async Task ReadCommands(string[] path)
        {
            var help = await ProcessRunner.RunAsync([cli, .. path, "--help"]);
            Assert.True(help.ExitCode == 0, help.Output + help.Error);
            var lines = help.Output.ReplaceLineEndings("\n").Split('\n');
            var commands = lines.SkipWhile(line => line != "Commands:").Skip(1);
            foreach (var line in commands)
            {
                var entry = Regex.Match(line, @"^  ([a-z][a-z0-9-]*) ");
                if (!entry.Success) continue;
                string[] child = [.. path, entry.Groups[1].Value];
                registered.Add(string.Join(' ', child));
                await ReadCommands(child);
            }
        }
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
        => DocumentationLinks.Validate(Directory.GetFiles(Root, "*.md").Append(Path.Combine(Repository, "docs", "build", "development.md")));

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
            Assert.Single(project.Select(example => example.Document).Distinct(StringComparer.Ordinal));
            Assert.Single(project, example => !example.Library);
            Assert.Contains(project, example => example.Library);
        }
    }
}
