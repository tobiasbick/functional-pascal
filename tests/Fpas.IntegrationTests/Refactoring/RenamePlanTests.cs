using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Queries;
using Fpas.Workspaces.Refactoring;
using System.Text.Json;
using System.Text.Json.Nodes;

namespace Fpas.IntegrationTests.Refactoring;

public sealed class RenamePlanTests
{
    private static RenamePlan Plan()
    {
        var project = Path.GetFullPath("Example.fpasproj"); var file = Path.GetFullPath("main.fpas");
        var plan = new RenamePlan(1, "rename", project, "Debug", new string('A', 64), new(file, 1, 1), "NewName",
            [new(project, "local", "OldName", new(file, 0, 7, 1, 1), false)], [], [project], []);
        return plan with { Id = RenamePlanJson.Identity(plan) };
    }

    [Theory]
    [InlineData("targets")]
    [InlineData("files")]
    [InlineData("selection")]
    public void Missing_required_nested_data_fails_as_an_invalid_plan(string property)
    {
        var node = JsonNode.Parse(RenamePlanJson.Serialize(Plan()))!; node[property] = null;
        Assert.Throws<ArgumentException>(() => RenamePlanJson.Read(System.Text.Encoding.UTF8.GetBytes(node.ToJsonString())));
    }

    [Fact]
    public void Unknown_fields_changed_checksums_and_forged_summaries_are_rejected()
    {
        var bytes = RenamePlanJson.Serialize(Plan()); Assert.Equal(0, RenamePlanJson.Read(bytes).EditCount);
        var unknown = JsonNode.Parse(bytes)!; unknown["unknown"] = true;
        Assert.Throws<JsonException>(() => RenamePlanJson.Read(System.Text.Encoding.UTF8.GetBytes(unknown.ToJsonString())));
        var modified = JsonNode.Parse(bytes)!; modified["newName"] = "OtherName";
        Assert.Throws<ArgumentException>(() => RenamePlanJson.Read(System.Text.Encoding.UTF8.GetBytes(modified.ToJsonString())));
        var summary = JsonNode.Parse(bytes)!; summary["changesPublicApi"] = true;
        Assert.Throws<ArgumentException>(() => RenamePlanJson.Read(System.Text.Encoding.UTF8.GetBytes(summary.ToJsonString())));
        summary = JsonNode.Parse(bytes)!; summary["editCount"] = "invalid";
        Assert.Throws<ArgumentException>(() => RenamePlanJson.Read(System.Text.Encoding.UTF8.GetBytes(summary.ToJsonString())));
    }
}
