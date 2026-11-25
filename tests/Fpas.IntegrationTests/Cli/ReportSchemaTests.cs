// [Documentation](../../../docs/pascal/tools/report-schemas.md)
using System.Text.Json.Nodes;
using System.Text.Json;
using Json.Schema;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Cli;

public sealed class ReportSchemaTests
{
    [Theory]
    [InlineData("workspace", "workspace", "missing.slnx")]
    [InlineData("operation", "profile", "summary", "missing.nettrace")]
    [InlineData("project", "restore", "missing.csproj", "--report", "json")]
    [InlineData("test", "test", "missing.csproj", "--report", "json")]
    [InlineData("debug", "debug", "missing.dll")]
    [InlineData("format", "fmt", "missing.fpas", "--report", "json")]
    [InlineData("doc", "doc", "missing.csproj", "--name", "Example", "--report", "json")]
    [InlineData("refactor", "refactor", "apply", "--report", "json")]
    [InlineData("environment", "env", "--report", "json", "--unknown")]
    [InlineData("compile", "compile", "--report", "json")]
    [InlineData("check", "check", "missing.csproj", "--report", "json")]
    [InlineData("lint", "lint", "missing.fpasproj", "--report", "json")]
    [InlineData("query-batch", "query", "batch")]
    [InlineData("query-outline", "query", "outline", "missing.fpas")]
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
        var property = name == "debug" ? "exitCode" : name is "query" or "query-batch" or "query-outline" or "workspace" ? "status" : "success";
        report[property] = new JsonArray("invalid-type");
        Assert.False(schema.Evaluate(JsonSerializer.SerializeToElement(report)).IsValid);
    }

    [Theory]
    [InlineData("check", "--report", "json")]
    [InlineData("lint", "--report", "json")]
    [InlineData("doc", "--report", "json", "--name", "String")]
    [InlineData("test", "--report", "json")]
    [InlineData("query", "type")]
    public async Task Missing_required_context_returns_the_command_error_report(params string[] arguments)
    {
        var response = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), .. arguments]);
        Assert.Equal(2, response.ExitCode);
        Assert.Empty(response.Error);
        using var report = JsonDocument.Parse(response.Output);
        var error = report.RootElement.GetProperty("error");
        Assert.Equal("invalid_argument", error.GetProperty("code").GetString());
        Assert.Contains("context", error.GetProperty("message").GetString(), StringComparison.OrdinalIgnoreCase);
    }

    [Theory]
    [InlineData("operation")]
    [InlineData("query-batch")]
    [InlineData("query-batch-input")]
    [InlineData("query-outline")]
    [InlineData("check-overlay-input")]
    [InlineData("lint")]
    [InlineData("lint-baseline")]
    [InlineData("workspace")]
    [InlineData("commands")]
    public async Task Schema_exports_work_without_JSON_reflection(string name)
    {
        using var directory = new TemporaryDirectory();
        var cli = TestEnvironment.Metadata("FpasCliPath");
        var config = JsonNode.Parse(File.ReadAllText(Path.ChangeExtension(cli, ".runtimeconfig.json")))!;
        var options = config["runtimeOptions"]!;
        if (options["configProperties"] is null) options["configProperties"] = new JsonObject();
        options["configProperties"]!["System.Text.Json.JsonSerializer.IsReflectionEnabledByDefault"] = false;
        var configPath = directory.File("fpas.runtimeconfig.json");
        File.WriteAllText(configPath, config.ToJsonString());
        var result = await ProcessRunner.RunAsync(["exec", "--runtimeconfig", configPath, cli, "schema", name], workingDirectory: directory.Path);
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        using var report = JsonDocument.Parse(result.Output);
        if (name == "commands") Assert.Equal(1, report.RootElement.GetProperty("schemaVersion").GetInt32());
        else Assert.Equal("urn:fpas:cli:" + name + ":1", report.RootElement.GetProperty("$id").GetString());
    }

    [Fact]
    public async Task Filtered_catalogue_exposes_the_selected_command_constraints_and_input_schema()
    {
        var cli = TestEnvironment.Metadata("FpasCliPath");
        var find = await ProcessRunner.RunAsync([cli, "schema", "commands", "query", "find"]);
        Assert.True(find.ExitCode == 0, find.Output + find.Error);
        using var document = JsonDocument.Parse(find.Output);
        var entry = Assert.Single(document.RootElement.GetProperty("commands").EnumerateArray());
        Assert.Equal("fpas query find", entry.GetProperty("command").GetString());
        var options = entry.GetProperty("options").EnumerateArray().ToDictionary(option => option.GetProperty("name").GetString()!);
        Assert.Equal(new[] { "exact", "prefix", "contains" }, options["--match"].GetProperty("allowedValues").EnumerateArray().Select(value => value.GetString()));
        Assert.Equal(1, options["--limit"].GetProperty("minimum").GetInt32());
        Assert.Equal(1000, options["--limit"].GetProperty("maximum").GetInt32());

        var check = await ProcessRunner.RunAsync([cli, "schema", "commands", "check"]);
        using var checkDocument = JsonDocument.Parse(check.Output);
        var rules = checkDocument.RootElement.GetProperty("commands")[0].GetProperty("rules").EnumerateArray().ToArray();
        var required = Assert.Single(rules, rule => rule.GetProperty("kind").GetString() == "requires" &&
            rule.GetProperty("options")[0].GetString() == "--dry-run");
        Assert.Equal(new[] { "--dry-run", "--fix" }, required.GetProperty("options").EnumerateArray().Select(value => value.GetString()));
        Assert.Contains(rules, rule => rule.GetProperty("kind").GetString() == "requires" &&
            rule.GetProperty("options")[0].GetString() == "--fix-id" && rule.GetProperty("options")[1].GetString() == "--fix");
        var exclusive = Assert.Single(rules, rule => rule.GetProperty("kind").GetString() == "excludes");
        Assert.Equal(new[] { "--fix", "--overlay" }, exclusive.GetProperty("options").EnumerateArray().Select(value => value.GetString()));

        var missing = await ProcessRunner.RunAsync([cli, "schema", "commands", "query", "missing"]);
        Assert.Equal(2, missing.ExitCode);
        var batch = await ProcessRunner.RunAsync([cli, "schema", "commands", "query", "batch"]);
        using var batchDocument = JsonDocument.Parse(batch.Output);
        Assert.Equal("urn:fpas:cli:query-batch-input:1", batchDocument.RootElement.GetProperty("commands")[0].GetProperty("inputSchema").GetString());
    }

    [Fact]
    public async Task Batch_input_schema_checks_operation_specific_fields_and_chained_selectors()
    {
        var cli = TestEnvironment.Metadata("FpasCliPath");
        var response = await ProcessRunner.RunAsync([cli, "schema", "query-batch-input"]);
        Assert.True(response.ExitCode == 0, response.Output + response.Error);
        var schema = JsonSchema.FromText(response.Output, new BuildOptions { SchemaRegistry = new SchemaRegistry() });
        Assert.True(IsValid("""
            [{"operation":"find","name":"A","match":"exact","kinds":["routine"]},
             {"operation":"refs","from":0,"limit":20,"offset":1},
             {"operation":"refs","symbol":"exact-id","project":"Library.fpasproj"},
             {"operation":"inspect","file":"a.fpas","line":1,"column":1}]
            """));
        Assert.False(IsValid("""[{"operation":"find","name":"A","match":"regex"}]"""));
        Assert.False(IsValid("""[{"operation":"inspect","from":0,"limit":101}]"""));
        Assert.False(IsValid("""[{"operation":"refs","from":0,"file":"a.fpas","line":1,"column":1}]"""));
        Assert.False(IsValid("""[{"operation":"type","from":0}]"""));
        Assert.False(IsValid("""[{"operation":"refs","symbol":"exact-id"}]"""));
        Assert.False(IsValid("""[{"operation":"refs","symbol":"exact-id","project":"Library.fpasproj","line":1}]"""));

        bool IsValid(string input)
        {
            using var document = JsonDocument.Parse(input);
            return schema.Evaluate(document.RootElement).IsValid;
        }
    }

    [Fact]
    public async Task Command_catalogue_exposes_registered_options_and_report_contracts()
    {
        var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "schema", "commands"]);
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        using var document = JsonDocument.Parse(result.Output);
        Assert.Equal(1, document.RootElement.GetProperty("schemaVersion").GetInt32());
        var commands = document.RootElement.GetProperty("commands").EnumerateArray()
            .ToDictionary(entry => entry.GetProperty("command").GetString()!);
        var batch = commands["fpas query batch"];
        Assert.Equal("urn:fpas:cli:query-batch:1", batch.GetProperty("reportSchema").GetString());
        Assert.True(Assert.Single(batch.GetProperty("arguments").EnumerateArray()).GetProperty("required").GetBoolean());
        Assert.False(batch.TryGetProperty("reportOption", out _));
        var refs = commands["fpas query refs"];
        var lint = commands["fpas lint"];
        Assert.Equal("urn:fpas:cli:lint:1", lint.GetProperty("reportSchema").GetString());
        Assert.Equal("--report json", lint.GetProperty("reportOption").GetString());
        Assert.Equal("urn:fpas:cli:lint-baseline:1", lint.GetProperty("optionInputSchemas").GetProperty("--baseline").GetString());
        Assert.Contains(lint.GetProperty("rules").EnumerateArray(), rule => rule.GetProperty("kind").GetString() == "excludes" &&
            rule.GetProperty("options").EnumerateArray().Select(option => option.GetString()).SequenceEqual(["--baseline", "--write-baseline"]));
        foreach (var (name, expected) in new[] { ("--held-in-mind", 7), ("--max-directory-entries", 15), ("--max-lines", 2000) })
        {
            var option = Option(lint, name);
            Assert.Equal(1, option.GetProperty("minimum").GetInt32());
            Assert.Equal(expected, option.GetProperty("default").GetInt32());
        }
        Assert.False(Option(refs, "--line").GetProperty("required").GetBoolean());
        Assert.Equal("string", Option(refs, "--symbol").GetProperty("type").GetString());
        Assert.Contains(refs.GetProperty("rules").EnumerateArray(), rule => rule.GetProperty("kind").GetString() == "requires" &&
            rule.GetProperty("options")[0].GetString() == "--symbol" && rule.GetProperty("options")[1].GetString() == "--expected-snapshot");
        Assert.Equal(100, Option(refs, "--limit").GetProperty("default").GetInt32());
        Assert.Equal(0, Option(refs, "--offset").GetProperty("default").GetInt32());
        Assert.False(Option(refs, "--existing-inputs").GetProperty("default").GetBoolean());
        var init = commands["fpas init project"];
        Assert.Equal("--report json", init.GetProperty("reportOption").GetString());
        Assert.Equal("string", Option(init, "--report").GetProperty("type").GetString());
        Assert.Equal("string[]", Option(commands["fpas workspace"], "--affected").GetProperty("type").GetString());
        Assert.Equal("string", Option(commands["fpas run"], "--cwd").GetProperty("type").GetString());
        Assert.Equal("string", Option(commands["fpas run"], "--stdin").GetProperty("type").GetString());
        Assert.DoesNotContain(commands["fpas refactor draft"].GetProperty("options").EnumerateArray(), entry =>
            entry.GetProperty("name").GetString() is "--no-restore" or "--existing-inputs");

        static JsonElement Option(JsonElement command, string name) => Assert.Single(command.GetProperty("options").EnumerateArray(),
            option => option.GetProperty("name").GetString() == name);
    }
}
