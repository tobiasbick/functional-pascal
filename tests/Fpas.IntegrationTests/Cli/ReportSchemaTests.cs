using System.Text.Json.Nodes;
using System.Text.Json;
using Json.Schema;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Cli;

public sealed class ReportSchemaTests
{
    [Theory]
    [InlineData("operation", "profile", "summary", "missing.nettrace")]
    [InlineData("project", "restore", "missing.csproj", "--report", "json")]
    [InlineData("test", "test", "missing.csproj", "--report", "json")]
    [InlineData("debug", "debug", "missing.dll")]
    [InlineData("debug", "diagnose", "failure", "--unknown")]
    [InlineData("format", "fmt", "missing.fpas", "--report", "json")]
    [InlineData("doc", "doc", "missing.csproj", "--name", "Example", "--report", "json")]
    [InlineData("refactor", "refactor", "apply", "--report", "json")]
    [InlineData("environment", "env", "--json", "--unknown")]
    [InlineData("compile", "compile", "--diagnostics", "json")]
    [InlineData("check", "check", "missing.csproj", "--diagnostics", "json")]
    [InlineData("query", "query", "definition", "missing.csproj", "--file", "missing.fpas", "--line", "1", "--column", "1")]
    public async Task Exported_schemas_accept_actual_CLI_failure_reports_and_reject_a_wrong_property_type(string name, params string[] arguments)
    {
        using var directory = new TemporaryDirectory();
        var cli = TestEnvironment.Metadata("FpasCliPath");
        var exported = await ProcessRunner.RunAsync([cli, "schema", name], workingDirectory: directory.Path);
        Assert.True(exported.ExitCode == 0, exported.Output + exported.Error);
        var schema = JsonSchema.FromText(exported.Output, new BuildOptions { SchemaRegistry = new SchemaRegistry() });
        var response = await ProcessRunner.RunAsync([cli, .. arguments], workingDirectory: directory.Path);
        Assert.NotEqual(0, response.ExitCode);
        var report = JsonNode.Parse(response.Output)!;
        var result = schema.Evaluate(JsonSerializer.SerializeToElement(report));
        Assert.True(result.IsValid, "Schema rejected the actual report: " + response.Output);
        var property = name == "debug" ? "exitCode" : name == "query" ? "status" : "success";
        report[property] = new JsonArray("invalid-type");
        Assert.False(schema.Evaluate(JsonSerializer.SerializeToElement(report)).IsValid);
    }
}
