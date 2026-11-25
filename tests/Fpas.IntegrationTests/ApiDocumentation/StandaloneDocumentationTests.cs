// [Documentation](../../../docs/pascal/tools/api-documentation.md#names-and-scope)
using Fpas.IntegrationTests.Documentation;
using Fpas.Tests.Support;
using System.Text.Json;

namespace Fpas.IntegrationTests.ApiDocumentation;

public sealed class StandaloneDocumentationTests
{
    [Fact]
    public async Task Broken_standalone_source_can_look_up_a_framework_overload_and_generate_checked_usage()
    {
        using var directory = new TemporaryDirectory(Path.Combine(DocumentationLinks.Repository, ".temp-data"));
        var source = directory.File("Broken helper.fpas");
        File.WriteAllText(source, "program Broken; begin Missing(); end; end program;");

        using var lookedUp = await Doc("--name", "System.IO.File.OpenRead");
        var page = lookedUp.RootElement.GetProperty("result");
        var entry = Assert.Single(page.GetProperty("entries").EnumerateArray());
        Assert.Equal(source, lookedUp.RootElement.GetProperty("context").GetString());
        Assert.True(page.GetProperty("coverage").GetProperty("complete").GetBoolean());
        Assert.Empty(page.GetProperty("searchedProjects").EnumerateArray());
        var id = entry.GetProperty("documentationId").GetString()!;
        Assert.Equal("M:System.IO.File.OpenRead(System.String)", id);

        using var selected = await Doc("--usage", id);
        var usage = Assert.Single(selected.RootElement.GetProperty("result").GetProperty("entries").EnumerateArray()).GetProperty("usage");
        Assert.Equal("template", usage.GetProperty("status").GetString());
        Assert.Contains("IO.File.OpenRead(Path)", usage.GetProperty("source").GetString());
        Assert.Equal(source, Assert.Single(Directory.EnumerateFileSystemEntries(directory.Path)));

        async Task<JsonDocument> Doc(params string[] args)
        {
            var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "doc", source, "--report", "json", .. args],
                workingDirectory: directory.Path);
            Assert.True(result.ExitCode == 0, result.Output + result.Error);
            Assert.Empty(result.Error);
            return JsonDocument.Parse(result.Output);
        }
    }
}
