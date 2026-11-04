using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using System.Text.Json;
using Fpas.IntegrationTests.Workspaces.Support;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class QueryCliTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Packaged_context_queries_return_typed_JSON_for_source_metadata_types_and_references()
    {
        using var project = sdk.CreateProject();
        foreach (var (operation, line, column) in new[] { ("definition", 8, 26), ("definition", 8, 17), ("type", 8, 26), ("refs", 6, 25) })
        {
            var result = await Query(project, operation, "Game/Game.fpasproj", "--file", "Game/Bridge.fpas", "--line", line.ToString(), "--column", column.ToString());
            Assert.True(result.ExitCode == 0, result.Output + result.Error); Assert.Equal("", result.Error);
            using var json = JsonDocument.Parse(result.Output); var body = json.RootElement;
            Assert.Equal("ok", body.GetProperty("status").GetString());
            Assert.True(body.GetProperty("coverage").GetProperty("complete").GetBoolean());
            Assert.Single(body.GetProperty("coverage").GetProperty("searched").EnumerateArray());
            var data = body.GetProperty("result");
            if (operation == "type") Assert.Equal("string", data.GetProperty("type").GetProperty("type").GetProperty("display").GetString());
            else if (operation == "refs") Assert.Equal(1, data.GetProperty("refs").GetProperty("returnedCount").GetInt32());
            else Assert.Equal(column == 26 ? "source" : "metadata", data.GetProperty("definition").GetProperty("origin").GetString());
        }
        Assert.False(File.Exists(project.File("Game/bin/Debug/net10.0/Game.dll")));
    }

    [Theory]
    [InlineData("definition", 1, 1, 0, "not_found")]
    [InlineData("definition", 8, 25, 0, "not_found")]
    [InlineData("type", 1, 1, 0, "not_found")]
    [InlineData("definition", 8, 999, 2, "failed")]
    public async Task Whitespace_comments_boundaries_and_invalid_coordinates_have_explicit_outcomes(string operation, int line, int column, int exit, string status)
    {
        using var project = sdk.CreateProject();
        var result = await Query(project, operation, "Game/Game.fpasproj", "--file", "Game/Bridge.fpas", "--line", line.ToString(), "--column", column.ToString());
        Assert.Equal(exit, result.ExitCode); Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output); Assert.Equal(status, json.RootElement.GetProperty("status").GetString());
        if (exit == 2) Assert.Equal("invalid_argument", json.RootElement.GetProperty("error").GetProperty("code").GetString());
    }

    [Theory]
    [InlineData("--line", "0")]
    [InlineData("--line", "abc")]
    [InlineData("--column", "0")]
    [InlineData("--timeout", "abc")]
    [InlineData("--limit", "1001")]
    [InlineData("--unknown", "value")]
    public async Task Query_parse_errors_are_always_one_JSON_document(string option, string value)
    {
        using var project = sdk.CreateProject();
        var arguments = new List<string> { "refs", "Game/Game.fpasproj", "--file", "Game/Bridge.fpas" };
        if (option != "--line") arguments.AddRange(["--line", "6"]);
        if (option != "--column") arguments.AddRange(["--column", "25"]);
        arguments.AddRange([option, value]);
        var result = await Query(project, arguments.ToArray());
        Assert.Equal(2, result.ExitCode); Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output); Assert.Equal("invalid_argument", json.RootElement.GetProperty("error").GetProperty("code").GetString());
    }

    [Fact]
    public async Task Invalid_source_never_returns_apparently_complete_empty_references()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Bridge.fpas"), "unit Invalid; function Read(): int; begin return Missing; end function; end unit;");
        var result = await Query(project, "refs", "Game/Game.fpasproj", "--file", "Game/Bridge.fpas", "--line", "1", "--column", "24");
        Assert.Equal(1, result.ExitCode); Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.Equal("analysis_failed", json.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.False(json.RootElement.GetProperty("coverage").GetProperty("complete").GetBoolean());
        Assert.False(json.RootElement.TryGetProperty("result", out _));
        Assert.Contains(json.RootElement.GetProperty("diagnostics").EnumerateArray(), diagnostic => diagnostic.GetProperty("code").GetString() == "FP3001");
    }

    [Fact]
    public async Task Inspection_does_not_execute_program_initializers_attribute_constructors_or_getters()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/Danger.cs"), """
            namespace Danger;
            public sealed class MarkerAttribute : System.Attribute
            {
                public MarkerAttribute() => System.IO.File.WriteAllText("attribute-executed", "bad");
                public string Name { get => throw new System.Exception("Getter executed"); set => System.IO.File.WriteAllText("setter-executed", "bad"); }
            }
            public static class Api
            {
                static Api() => System.IO.File.WriteAllText("initializer-executed", "bad");
                public static int Value { get { System.IO.File.WriteAllText("getter-executed", "bad"); return 42; } }
            }
            """);
        const string source = """
            unit Probe;
            uses Danger as D;
            [D.MarkerAttribute(Name := 'example')]
            public function Read(): int; begin return D.Api.Value; end function;
            public const Initial: int := Read();
            end unit;
            """;
        File.WriteAllText(project.File("Game/Bridge.fpas"), source);
        var selection = SourcePosition.Select(project, "Game/Bridge.fpas", source, "Value;");
        var result = await Query(project, "definition", "Game/Game.fpasproj", "--file", selection.File, "--line", selection.Line.ToString(), "--column", selection.Column.ToString());
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.Equal("Value", json.RootElement.GetProperty("result").GetProperty("definition").GetProperty("symbol").GetProperty("name").GetString());
        Assert.Empty(Directory.GetFiles(project.Path, "*-executed", SearchOption.AllDirectories));
        Assert.False(File.Exists(project.File("Game/bin/Debug/net10.0/Game.dll")));
    }

    [Fact]
    public async Task Oversized_query_responses_are_replaced_with_one_bounded_result_limit_error()
    {
        using var project = sdk.CreateProject();
        var name = "Large" + new string('x', 1_500_000);
        var source = "unit Bounds; public function " + name + "(): int; begin return 42; end function; end unit;";
        File.WriteAllText(project.File("Game/Bridge.fpas"), source);
        var result = await Query(project, "definition", "Game/Game.fpasproj", "--file", "Game/Bridge.fpas", "--line", "1", "--column", "30");
        Assert.Equal(1, result.ExitCode); Assert.Equal("", result.Error);
        Assert.InRange(System.Text.Encoding.UTF8.GetByteCount(result.Output), 1, 4_194_304);
        using var json = JsonDocument.Parse(result.Output);
        Assert.Equal("result_limit", json.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.False(json.RootElement.TryGetProperty("result", out _));
    }

    private static Task<ProcessResult> Query(TemporaryDirectory project, params string[] arguments)
        => ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "query", .. arguments], workingDirectory: project.Path, timeLimit: TimeSpan.FromMinutes(3));
}
