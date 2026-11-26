// [Documentation](../../../docs/architecture/workspaces.md)
using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using System.Text.Json;
using Fpas.IntegrationTests.Workspaces.Support;
using System.IO.Compression;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class QueryCliTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Packaged_find_and_inspect_publish_distinct_typed_results_and_discoverable_help()
    {
        using var project = sdk.CreateProject();
        foreach (var operation in new[] { "find", "inspect" })
        {
            var result = operation == "find"
                ? await Query(project, operation, "Game/Game.fpasproj", "--name", "Message", "--kind", "routine", "--kind", "global", "--match", "prefix")
                : await Query(project, operation, "Game/Game.fpasproj", "--file", "Game/Bridge.fpas", "--line", "14", "--column", "26");
            Assert.True(result.ExitCode == 0, result.Output + result.Error); Assert.Equal("", result.Error);
            using var json = JsonDocument.Parse(result.Output);
            Assert.Equal("ok", json.RootElement.GetProperty("status").GetString());
            var data = json.RootElement.GetProperty("result").GetProperty(operation);
            if (operation == "find")
            {
                Assert.False(json.RootElement.TryGetProperty("selection", out _));
                Assert.Equal("Message", Assert.Single(data.GetProperty("matches").EnumerateArray()).GetProperty("name").GetString());
            }
            else Assert.Equal("available", data.GetProperty("source").GetProperty("declaration").GetProperty("availability").GetString());
            var help = await Query(project, operation, "--help");
            Assert.Equal(0, help.ExitCode); Assert.Contains("Examples:", help.Output); Assert.Contains("fpas query " + operation, help.Output);
        }
        Assert.False(File.Exists(project.File("Game/bin/Debug/net10.0/Game.dll")));
    }

    [Theory]
    [InlineData("find", "--name", "X", "--match", "regex")]
    [InlineData("find", "--name", "X", "--kind", "missing")]
    [InlineData("find", "--name", "", "--offset", "0")]
    [InlineData("find", "--name", "X", "--offset", "-1")]
    [InlineData("inspect", "--file", "X.fpas", "--limit", "101")]
    [InlineData("callees", "--file", "X.fpas", "--offset", "-1")]
    [InlineData("callers", "--file", "X.fpas", "--limit", "1001")]
    public async Task Discovery_argument_errors_are_one_JSON_document_without_preparing_a_context(string operation, string flag, string value, string other, string setting)
    {
        using var project = sdk.CreateProject();
        var result = await Query(project, operation, "Missing.fpasproj", flag, value, other, setting);
        Assert.Equal(2, result.ExitCode); Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.Equal("invalid_argument", json.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.False(Directory.Exists(project.File("Game/obj")));
    }

    [Fact]
    public async Task Packaged_context_queries_return_typed_JSON_for_source_metadata_types_and_references()
    {
        using var project = sdk.CreateProject();
        foreach (var (operation, line, column) in new[] { ("definition", 14, 26), ("definition", 14, 17), ("type", 14, 26), ("refs", 12, 25) })
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
    [InlineData("definition", 14, 25, 0, "not_found")]
    [InlineData("type", 1, 1, 0, "not_found")]
    [InlineData("definition", 14, 999, 2, "failed")]
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
        if (option != "--line") arguments.AddRange(["--line", "12"]);
        if (option != "--column") arguments.AddRange(["--column", "25"]);
        arguments.AddRange([option, value]);
        var result = await Query(project, arguments.ToArray());
        Assert.Equal(2, result.ExitCode); Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output); Assert.Equal("invalid_argument", json.RootElement.GetProperty("error").GetProperty("code").GetString());
    }

    [Theory]
    [InlineData("refs")]
    [InlineData("find")]
    [InlineData("inspect")]
    [InlineData("callees")]
    [InlineData("callers")]
    public async Task Invalid_source_never_returns_apparently_complete_empty_results(string operation)
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Bridge.fpas"), "unit Invalid; function Read(): int; begin return Missing; end function; end unit;");
        var result = operation == "find" ? await Query(project, operation, "Game/Game.fpasproj", "--name", "Absent")
            : await Query(project, operation, "Game/Game.fpasproj", "--file", "Game/Bridge.fpas", "--line", "1", "--column", "24");
        Assert.Equal(operation == "callees" ? 1 : 0, result.ExitCode); Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output);
        if (operation == "callees")
        { Assert.Equal("analysis_unavailable", json.RootElement.GetProperty("error").GetProperty("code").GetString()); Assert.False(json.RootElement.TryGetProperty("result", out _)); }
        else
        {
            Assert.Equal("ok", json.RootElement.GetProperty("status").GetString()); Assert.True(json.RootElement.TryGetProperty("result", out _));
            Assert.Equal("partial", json.RootElement.GetProperty("coverage").GetProperty("semantics")[0].GetProperty("availability").GetString());
            Assert.NotNull(json.RootElement.GetProperty("snapshotId").GetString());
        }
        Assert.False(json.RootElement.GetProperty("coverage").GetProperty("complete").GetBoolean());
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
        foreach (var operation in new[] { "inspect", "callees", "callers" })
        {
            var result = await Query(project, operation, "Game/Game.fpasproj", "--file", selection.File, "--line", selection.Line.ToString(), "--column", selection.Column.ToString());
            Assert.True(result.ExitCode == 0, result.Output + result.Error);
            using var json = JsonDocument.Parse(result.Output);
            var data = json.RootElement.GetProperty("result").GetProperty(operation);
            Assert.Equal("Value", data.GetProperty(operation == "inspect" ? "definition" : "target").GetProperty("symbol").GetProperty("name").GetString());
        }
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

    [Fact]
    public async Task Extracted_SDK_call_commands_publish_typed_pages_and_retain_reference_relationships()
    {
        using var project = sdk.CreateProject();
        const string source = """
            unit Numbers;
            public function Seed(): int; begin return 1; end function;
            public function Work(): int;
            begin
              const Callback: function(): int := Seed;
              return Seed() + Seed() + Callback();
            end function;
            end unit;
            """;
        File.WriteAllText(project.File("Game/Bridge.fpas"), source);
        var tools = project.File("ExtractedSdk");
        ZipFile.ExtractToDirectory(Path.Combine(sdk.Feed, "Fpas.NET.Sdk." + sdk.Version + ".nupkg"), tools);
        var compiler = Assert.Single(Directory.GetFiles(tools, "Fpas.Cli.dll", SearchOption.AllDirectories));
        foreach (var operation in new[] { "callees", "callers" })
        {
            var selection = SourcePosition.Select(project, "Game/Bridge.fpas", source, operation == "callees" ? "Work():" : "Seed():");
            var result = await ProcessRunner.RunAsync([compiler, "query", operation, "Game/Game.fpasproj", "--file", selection.File,
                "--line", selection.Line.ToString(), "--column", selection.Column.ToString(), "--limit", "1", "--offset", "1"], workingDirectory: project.Path, timeLimit: TimeSpan.FromMinutes(3));
            Assert.True(result.ExitCode == 0, result.Output + result.Error); Assert.Equal("", result.Error);
            using var json = JsonDocument.Parse(result.Output);
            Assert.Equal("ok", json.RootElement.GetProperty("status").GetString());
            Assert.True(json.RootElement.GetProperty("coverage").GetProperty("complete").GetBoolean());
            var data = json.RootElement.GetProperty("result").GetProperty(operation);
            var page = data.GetProperty("page");
            Assert.Equal(operation == "callees" ? 4 : 3, page.GetProperty("totalCount").GetInt32());
            Assert.Equal(1, page.GetProperty("returnedCount").GetInt32()); Assert.Equal(2, page.GetProperty("nextOffset").GetInt32());
            Assert.True(json.RootElement.GetProperty("truncated").GetBoolean());
            Assert.Equal("invocation", Assert.Single(page.GetProperty("sites").EnumerateArray()).GetProperty("relationship").GetString());
            Assert.Equal(1, page.GetProperty("callableReferenceCount").GetInt32());
        }
        Assert.False(File.Exists(project.File("Game/bin/Debug/net10.0/Game.dll")));
    }

    /// <summary>Checks that a batch chains unique matches and preserves independent query results after an ambiguity.</summary>
    [Fact]
    public async Task Batch_chains_unique_declarations_and_keeps_other_results_after_an_ambiguous_match()
    {
        using var project = sdk.CreateProject();
        const string context = "Game/Game.fpasproj";
        const string file = "Game/Bridge.fpas";
        const string source = """
            unit Numbers;
            public function Seed(): int; begin return 1; end function;
            public function SeedOther(): int; begin return 2; end function;
            public function Work(): int; begin return Seed() + Seed(); end function;
            end unit;
            """;
        File.WriteAllText(project.File(file), source);
        var prepared = await Query(project, "find", context, "--name", "Seed");
        Assert.True(prepared.ExitCode == 0, prepared.Output + prepared.Error);
        const string input = """
            [
              {"operation":"find","name":"Seed","match":"exact"},
              {"operation":"refs","from":0,"limit":1},
              {"operation":"type","file":"Game/Bridge.fpas","line":"bad","column":1},
              {"operation":"find","name":"Seed","match":"prefix","limit":1},
              {"operation":"inspect","from":3},
              {"operation":"inspect","from":0},
              {"operation":"refs","from":0,"limit":1,"offset":1},
              {"operation":"refs","from":7}
            ]
            """;
        var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "query", "batch", context, "--existing-inputs"], input,
            workingDirectory: project.Path, timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        Assert.Empty(result.Error);
        using var json = JsonDocument.Parse(result.Output);
        var body = json.RootElement;
        Assert.Equal(1, body.GetProperty("schemaVersion").GetInt32());
        Assert.False(string.IsNullOrEmpty(body.GetProperty("snapshotId").GetString()));
        Assert.True(body.GetProperty("coverage").GetProperty("complete").GetBoolean());
        Assert.False(body.GetProperty("preparation").GetProperty("restorePerformed").GetBoolean());
        Assert.False(body.GetProperty("preparation").GetProperty("dependencyBuildsAllowed").GetBoolean());
        var items = body.GetProperty("results");
        Assert.Equal(new[] { "find", "refs", "type", "find", "inspect", "inspect", "refs", "refs" },
            items.EnumerateArray().Select(item => item.GetProperty("operation").GetString()));
        Assert.Equal("invalid_argument", items[2].GetProperty("error").GetProperty("code").GetString());
        Assert.Equal("invalid_argument", items[7].GetProperty("error").GetProperty("code").GetString());
        Assert.Equal("ambiguous_symbol", items[4].GetProperty("error").GetProperty("code").GetString());
        Assert.Equal(2, items[4].GetProperty("candidateCount").GetInt32());
        Assert.Single(items[4].GetProperty("candidates").EnumerateArray());
        var symbol = items[0].GetProperty("result").GetProperty("find").GetProperty("matches")[0].GetProperty("symbol").GetString();
        var inspection = items[5].GetProperty("result").GetProperty("inspect");
        Assert.Equal(symbol, inspection.GetProperty("definition").GetProperty("symbol").GetProperty("id").GetString());
        var first = items[1].GetProperty("result").GetProperty("refs");
        var second = items[6].GetProperty("result").GetProperty("refs");
        Assert.Equal(symbol, first.GetProperty("definition").GetProperty("symbol").GetProperty("id").GetString());
        Assert.Equal(2, first.GetProperty("totalCount").GetInt32());
        Assert.Equal(1, first.GetProperty("nextOffset").GetInt32());
        Assert.Equal(1, second.GetProperty("offset").GetInt32());
        Assert.False(second.TryGetProperty("nextOffset", out _));
        Assert.True(items[1].GetProperty("truncated").GetBoolean());
        Assert.True(items[6].GetProperty("truncated").GetBoolean());
        Assert.NotEqual(first.GetProperty("references")[0].GetProperty("span").GetProperty("offset").GetInt32(),
            second.GetProperty("references")[0].GetProperty("span").GetProperty("offset").GetInt32());
    }

    /// <summary>Checks completion and signature help against partially written source.</summary>
    [Fact]
    public async Task Editor_queries_return_candidates_and_active_parameters_for_incomplete_source()
    {
        using var project = sdk.CreateProject();
        const string context = "Game/Game.fpasproj";
        const string file = "Game/Bridge.fpas";
        const string member = """
            unit Demo.Bridge;
            public type Box = record public Value: int; end record;
            public function Read(Item: Box): int;
            begin
              return Item.Va;
            end function;
            end unit;
            """;
        const string call = """
            unit Demo.Bridge;
            public function Echo of T(Value: T): T; begin return Value; end function;
            public function Read(): int;
            begin
              return Echo(Value := 42;
            end function;
            end unit;
            """;
        File.WriteAllText(project.File(file), member);
        var completion = SourcePosition.Select(project, file, member, ";\nend function;");
        var completed = await Query(project, "complete", context, "--file", file, "--line", completion.Line.ToString(), "--column", completion.Column.ToString());
        Assert.True(completed.ExitCode == 0, completed.Output + completed.Error);
        using var completionJson = JsonDocument.Parse(completed.Output);
        var matches = completionJson.RootElement.GetProperty("result").GetProperty("complete");
        Assert.True(matches.GetProperty("items").EnumerateArray().Any(item => item.GetProperty("name").GetString() == "Value"), matches.GetRawText());
        Assert.Equal(2, matches.GetProperty("replacement").GetProperty("length").GetInt32());

        File.WriteAllText(project.File(file), call);
        var signature = SourcePosition.Select(project, file, call, ";\nend function;");
        var input = JsonSerializer.Serialize(new[] { new { operation = "signature", file, line = signature.Line, column = signature.Column } });
        var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "query", "batch", context, "--existing-inputs", "--diagnostic-limit", "0"],
            input, workingDirectory: project.Path, timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        using var json = JsonDocument.Parse(result.Output);
        var body = json.RootElement;
        Assert.Empty(body.GetProperty("diagnostics").EnumerateArray());
        Assert.True(body.GetProperty("diagnosticPage").GetProperty("totalCount").GetInt32() > 0);
        Assert.False(body.GetProperty("coverage").GetProperty("complete").GetBoolean());
        var signatures = body.GetProperty("results")[0].GetProperty("result").GetProperty("signature");
        var active = signatures.GetProperty("items")[0];
        Assert.Equal(0, active.GetProperty("activeParameter").GetInt32());
        Assert.Contains("Echo of int", active.GetProperty("label").GetString());
    }

    /// <summary>Checks that bounded batch output retains independent answers after an oversized result.</summary>
    [Fact]
    public async Task Oversized_batch_entries_leave_other_results_available()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Bridge.fpas"), "unit Demo.Bridge;\n// " + new string('<', 13_000) +
            "\npublic function Seed(): int; begin return 1; end function;\nend unit;");
        var input = JsonSerializer.Serialize(new object[] { new { operation = "find", name = "Seed" } }
            .Concat(Enumerable.Repeat<object>(new { operation = "inspect", from = 0 }, 62))
            .Append(new { operation = "find", name = "Other" }));
        var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "query", "batch", "Game/Game.fpasproj"], input,
            workingDirectory: project.Path, timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(result.ExitCode == 0, result.Error);
        using var json = JsonDocument.Parse(result.Output);
        var items = json.RootElement.GetProperty("results");
        Assert.Equal(64, items.GetArrayLength());
        Assert.Equal("ok", items[1].GetProperty("status").GetString());
        Assert.Contains(items.EnumerateArray(), item => item.TryGetProperty("error", out var error) &&
            error.GetProperty("code").GetString() == "result_limit");
        Assert.Equal("ok", items[63].GetProperty("status").GetString());
        Assert.InRange(System.Text.Encoding.UTF8.GetByteCount(result.Output), 1, (4 * 1024 * 1024) + 2);
    }

    private static Task<ProcessResult> Query(TemporaryDirectory project, params string[] arguments)
        => ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "query", .. arguments], workingDirectory: project.Path, timeLimit: TimeSpan.FromMinutes(3));
}
