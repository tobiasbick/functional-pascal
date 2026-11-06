using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using System.Text.Json;
using static Fpas.IntegrationTests.LanguageServer.LspFeatureTests;

namespace Fpas.IntegrationTests.LanguageServer;

public sealed class LspCliParityTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    private const string Source = "unit Demo.Bridge;\r\nfunction Answer():int;begin return 42;end function;\r\nfunction Run():int;begin const Label:string:='😀';return Answer();end function;\r\nend unit;";

    private async Task Prepare(TemporaryDirectory directory, string source)
    {
        File.WriteAllText(directory.File("Game/Game.fpasproj"), $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Library</OutputType></PropertyGroup></Project>");
        File.WriteAllText(directory.File("Game/Bridge.fpas"), source);
        var restored = await sdk.Run(directory, "restore", "Game/Game.fpasproj"); Assert.True(restored.ExitCode == 0, restored.Output + restored.Error);
    }

    [Fact]
    public async Task Disk_dependency_outputs_and_live_private_images_have_identical_cross_project_query_locations()
    {
        using var directory = sdk.CreateProject();
        const string library = "unit Demo.Bridge; public function Answer(): int; begin return 42; end function; end unit;";
        const string consumer = "unit Consumer; uses Demo.Bridge as B; function Run(): int; begin return B.Answer(); end function; end unit;";
        await Prepare(directory, library); Directory.CreateDirectory(directory.File("Consumer"));
        File.WriteAllText(directory.File("Consumer/Consumer.fpasproj"), $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Library</OutputType></PropertyGroup><ItemGroup><ProjectReference Include=\"../Game/Game.fpasproj\" /></ItemGroup></Project>");
        var path = directory.File("Consumer/Main.fpas"); File.WriteAllText(path, consumer);
        var restored = await sdk.Run(directory, "restore", "Consumer/Consumer.fpasproj"); Assert.True(restored.ExitCode == 0, restored.Output + restored.Error);
        var cliResults = new List<JsonElement>();
        foreach (var operation in new[] { "definition", "refs" })
        {
            var found = await Run(directory, "query", operation, "Consumer/Consumer.fpasproj", "--file", path, "--line", "1",
                "--column", (consumer.IndexOf("Answer", StringComparison.Ordinal) + 1).ToString(), "--no-restore");
            Assert.True(found.ExitCode == 0, found.Output + found.Error); Assert.Empty(found.Error);
            using var json = JsonDocument.Parse(found.Output); cliResults.Add(json.RootElement.GetProperty("result").GetProperty(operation).Clone());
        }
        var output = directory.File("Game/bin/Debug/net10.0/Game.dll"); var bytes = File.ReadAllBytes(output); var modified = File.GetLastWriteTimeUtc(output);
        await using var client = new LspProbe(); await client.Initialize(directory.File("Consumer/Consumer.fpasproj"));
        await client.Notify("initialized", new { }); await client.Open(path, consumer); await client.Diagnostics(path, 1);
        AssertLocation(cliResults[0].GetProperty("symbol").GetProperty("declaration"),
            Result(await client.Request("textDocument/definition", new { textDocument = Document(path), position = Position(consumer, "Answer") })));
        var references = Result(await client.Request("textDocument/references", new { textDocument = Document(path), position = Position(consumer, "Answer"), context = new { includeDeclaration = false } }));
        AssertLocation(Assert.Single(cliResults[1].GetProperty("references").EnumerateArray()).GetProperty("span"), Assert.Single(references.EnumerateArray()));
        await client.Stop(); Assert.Equal(bytes, File.ReadAllBytes(output)); Assert.Equal(modified, File.GetLastWriteTimeUtc(output));
        Assert.False(File.Exists(directory.File("Consumer/bin/Debug/net10.0/Consumer.dll")));
    }

    [Fact]
    public async Task Same_saved_snapshot_has_identical_navigation_search_rename_and_formatting_through_both_adapters()
    {
        using var directory = sdk.CreateProject(); await Prepare(directory, Source); var path = directory.File("Game/Bridge.fpas");
        await using var client = new LspProbe(); await client.Initialize(directory.File("Game/Game.fpasproj"));
        await client.Notify("initialized", new { }); await client.Open(path, Source, 7); await client.Diagnostics(path, 7);

        var definition = await Query(directory, "definition", Source, "Answer();");
        AssertLocation(definition.GetProperty("result").GetProperty("definition").GetProperty("symbol").GetProperty("declaration"),
            Result(await client.Request("textDocument/definition", new { textDocument = Document(path), position = Position(Source, "Answer();") })));
        var references = await Query(directory, "refs", Source, "Answer():");
        var locations = Result(await client.Request("textDocument/references", new { textDocument = Document(path), position = Position(Source, "Answer():"), context = new { includeDeclaration = false } }));
        var expected = references.GetProperty("result").GetProperty("refs").GetProperty("references");
        Assert.Equal(expected.GetArrayLength(), locations.GetArrayLength());
        for (var index = 0; index < expected.GetArrayLength(); index++) AssertLocation(expected[index].GetProperty("span"), locations[index]);

        var search = await Run(directory, "query", "find", "Game/Game.fpasproj", "--name", "Answer", "--no-restore");
        Assert.Equal(0, search.ExitCode); using var searchJson = JsonDocument.Parse(search.Output);
        var symbols = Result(await client.Request("workspace/symbol", new { query = "Answer" }));
        var match = Assert.Single(searchJson.RootElement.GetProperty("result").GetProperty("find").GetProperty("matches").EnumerateArray());
        var symbol = Assert.Single(symbols.EnumerateArray()); Assert.Equal(match.GetProperty("name").GetString(), symbol.GetProperty("name").GetString());
        AssertLocation(match.GetProperty("span"), symbol.GetProperty("location"));

        var rename = await Rename(directory, "type", "rename.json"); Assert.Equal(0, rename.ExitCode); Assert.Empty(rename.Error);
        using var plan = JsonDocument.Parse(File.ReadAllBytes(directory.File("rename.json")));
        var edits = Result(await client.Request("textDocument/rename", new { textDocument = Document(path), position = Position(Source, "Answer():"), newName = "type" }));
        var document = Assert.Single(edits.GetProperty("documentChanges").EnumerateArray());
        LspProbe.AssertFileUri(path, document.GetProperty("textDocument").GetProperty("uri"));
        Assert.Equal(7, document.GetProperty("textDocument").GetProperty("version").GetInt32());
        var cliEdits = Assert.Single(plan.RootElement.GetProperty("files").EnumerateArray()).GetProperty("edits");
        var lspEdits = document.GetProperty("edits"); Assert.Equal(cliEdits.GetArrayLength(), lspEdits.GetArrayLength());
        for (var index = 0; index < cliEdits.GetArrayLength(); index++)
        {
            AssertRange(cliEdits[index].GetProperty("location"), lspEdits[index].GetProperty("range"));
            Assert.Equal(cliEdits[index].GetProperty("newText").GetString(), lspEdits[index].GetProperty("newText").GetString());
        }
        var formatted = await Run(directory, "fmt", "--stdout", path); Assert.Equal(0, formatted.ExitCode); Assert.Empty(formatted.Error);
        var formatting = Result(await client.Request("textDocument/formatting", new { textDocument = Document(path), options = new { tabSize = 2, insertSpaces = true } }));
        Assert.Equal(formatted.Output, Assert.Single(formatting.EnumerateArray()).GetProperty("newText").GetString());
        Assert.Equal(Source, File.ReadAllText(path)); Assert.False(File.Exists(directory.File("Game/bin/Debug/net10.0/Game.dll")));
        await client.Stop();
    }

    [Fact]
    public async Task Same_partial_snapshot_keeps_checked_definitions_and_rejects_unavailable_selections_and_rename()
    {
        const string source = "unit Demo.Bridge; function Answer(): int; begin return 42; end function; function Broken(): int; begin return Missing; end function; end unit;";
        using var directory = sdk.CreateProject(); await Prepare(directory, source); var path = directory.File("Game/Bridge.fpas");
        await using var client = new LspProbe(); await client.Initialize(directory.File("Game/Game.fpasproj")); await client.Notify("initialized", new { }); await client.Open(path, source);
        var diagnostics = (await client.Diagnostics(path, 1)).GetProperty("params").GetProperty("diagnostics");
        var safe = await Query(directory, "definition", source, "Answer():");
        AssertLocation(safe.GetProperty("result").GetProperty("definition").GetProperty("symbol").GetProperty("declaration"),
            Result(await client.Request("textDocument/definition", new { textDocument = Document(path), position = Position(source, "Answer():") })));
        foreach (var expected in safe.GetProperty("diagnostics").EnumerateArray())
        {
            var actual = Assert.Single(diagnostics.EnumerateArray(), item => item.GetProperty("code").GetString() == expected.GetProperty("code").GetString());
            Assert.Equal(expected.GetProperty("message").GetString(), actual.GetProperty("message").GetString());
            AssertRange(expected.GetProperty("location"), actual.GetProperty("range"));
        }
        var failed = await Query(directory, "definition", source, "Missing", success: false);
        Assert.Equal("analysis_unavailable", failed.GetProperty("error").GetProperty("code").GetString());
        var lspFailure = await client.Request("textDocument/definition", new { textDocument = Document(path), position = Position(source, "Missing") });
        Error(lspFailure, -32803); Assert.Contains("analysis_unavailable", lspFailure.GetProperty("error").GetProperty("message").GetString());
        var partial = await Query(directory, "refs", source, "Answer():");
        Assert.Contains(partial.GetProperty("coverage").GetProperty("semantics").EnumerateArray(), item => item.GetProperty("availability").GetString() == "partial");
        // Standard LSP arrays have no coverage field: refuse a partial list rather than presenting it as exhaustive.
        Error(await client.Request("textDocument/references", new { textDocument = Document(path), position = Position(source, "Answer():"), context = new { includeDeclaration = false } }), -32803);
        var renamed = await Rename(directory, "Fetch", "invalid.json", source); Assert.Equal(1, renamed.ExitCode); Assert.False(File.Exists(directory.File("invalid.json")));
        using var rename = JsonDocument.Parse(renamed.Output); Assert.Equal("analysis_failed", rename.RootElement.GetProperty("error").GetProperty("code").GetString());
        var refused = await client.Request("textDocument/rename", new { textDocument = Document(path), position = Position(source, "Answer():"), newName = "Fetch" });
        Error(refused, -32803); Assert.Contains("analysis_failed", refused.GetProperty("error").GetProperty("message").GetString());
        Assert.Equal(source, File.ReadAllText(path)); await client.Stop();
    }

    [Fact]
    public async Task Both_adapters_reject_the_same_rename_conflict_and_invalid_format_without_source_changes()
    {
        using var directory = sdk.CreateProject(); await Prepare(directory, Source); var path = directory.File("Game/Bridge.fpas");
        await using var client = new LspProbe(); await client.Initialize(directory.File("Game/Game.fpasproj")); await client.Notify("initialized", new { }); await client.Open(path, Source);
        var renamed = await Rename(directory, "Run", "conflict.json"); Assert.Equal(1, renamed.ExitCode);
        using var rename = JsonDocument.Parse(renamed.Output); var code = rename.RootElement.GetProperty("error").GetProperty("code").GetString()!;
        Assert.Equal("rename_conflict", code); Assert.False(File.Exists(directory.File("conflict.json")));
        var refused = await client.Request("textDocument/rename", new { textDocument = Document(path), position = Position(Source, "Answer():"), newName = "Run" });
        Error(refused, -32803); Assert.Contains(code, refused.GetProperty("error").GetProperty("message").GetString());
        const string invalid = "unit Demo.Bridge;"; File.WriteAllText(path, invalid); await client.FullChange(path, invalid, 2);
        var formatted = await Run(directory, "fmt", path, "--check", "--report", "json"); Assert.Equal(1, formatted.ExitCode);
        using var report = JsonDocument.Parse(formatted.Output); Assert.False(report.RootElement.GetProperty("success").GetBoolean());
        var formatFailure = await client.Request("textDocument/formatting", new { textDocument = Document(path), options = new { tabSize = 2, insertSpaces = true } }); Error(formatFailure, -32803);
        foreach (var diagnostic in report.RootElement.GetProperty("diagnostics").EnumerateArray())
            Assert.Contains(diagnostic.GetProperty("code").GetString()!, formatFailure.GetProperty("error").GetProperty("message").GetString());
        Assert.Equal(invalid, File.ReadAllText(path)); await client.Stop();
    }

    private static void AssertLocation(JsonElement span, JsonElement location)
    { LspProbe.AssertFileUri(span.GetProperty("path").GetString()!, location.GetProperty("uri")); AssertRange(span, location.GetProperty("range")); }
    private static void AssertRange(JsonElement span, JsonElement range)
    {
        var line = span.GetProperty("line").GetInt32() - 1; var column = span.GetProperty("column").GetInt32() - 1;
        Assert.Equal(line, range.GetProperty("start").GetProperty("line").GetInt32()); Assert.Equal(column, range.GetProperty("start").GetProperty("character").GetInt32());
        Assert.Equal(line, range.GetProperty("end").GetProperty("line").GetInt32()); Assert.Equal(column + span.GetProperty("length").GetInt32(), range.GetProperty("end").GetProperty("character").GetInt32());
    }
    private static async Task<JsonElement> Query(TemporaryDirectory directory, string operation, string source, string marker, bool success = true)
    {
        using var position = JsonDocument.Parse(JsonSerializer.Serialize(Position(source, marker))); var value = position.RootElement;
        var result = await Run(directory, "query", operation, "Game/Game.fpasproj", "--file", directory.File("Game/Bridge.fpas"),
            "--line", (value.GetProperty("line").GetInt32() + 1).ToString(), "--column", (value.GetProperty("character").GetInt32() + 1).ToString(), "--no-restore");
        Assert.True(result.ExitCode == (success ? 0 : 1), result.Output + result.Error); Assert.Empty(result.Error);
        using var json = JsonDocument.Parse(result.Output); return json.RootElement.Clone();
    }
    private static Task<ProcessResult> Rename(TemporaryDirectory directory, string name, string plan, string source = Source)
    {
        using var position = JsonDocument.Parse(JsonSerializer.Serialize(Position(source, "Answer():"))); var value = position.RootElement;
        return Run(directory, "refactor", "rename", "Game/Game.fpasproj", "--file", "Game/Bridge.fpas",
            "--line", (value.GetProperty("line").GetInt32() + 1).ToString(), "--column", (value.GetProperty("character").GetInt32() + 1).ToString(),
            "--new-name", name, "--plan", directory.File(plan), "--report", "json", "--no-restore");
    }
    private static Task<ProcessResult> Run(TemporaryDirectory directory, params string[] arguments)
        => ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), .. arguments], workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(2));
}
