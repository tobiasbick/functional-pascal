using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using System.Text.Json;

namespace Fpas.IntegrationTests.LanguageServer;

public sealed class LspFeatureTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    private async Task Prepare(TemporaryDirectory directory, string source)
    {
        File.WriteAllText(directory.File("Game/Game.fpasproj"), $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Library</OutputType></PropertyGroup></Project>");
        File.WriteAllText(directory.File("Game/Bridge.fpas"), source);
        var restored = await sdk.Run(directory, "restore", "Game/Game.fpasproj"); Assert.True(restored.ExitCode == 0, restored.Output + restored.Error);
    }
    internal static object Document(string path) => new { uri = new Uri(path).AbsoluteUri };
    internal static object Position(string text, string marker, bool last = false)
    {
        var offset = last ? text.LastIndexOf(marker, StringComparison.Ordinal) : text.IndexOf(marker, StringComparison.Ordinal); Assert.True(offset >= 0);
        var line = 0; var character = 0;
        for (var index = 0; index < offset; index++)
        {
            if (text[index] == '\r') { if (index + 1 < offset && text[index + 1] == '\n') index++; line++; character = 0; }
            else if (text[index] == '\n') { line++; character = 0; } else character++;
        }
        return new { line, character };
    }
    internal static JsonElement Result(JsonElement response) { Assert.True(response.TryGetProperty("result", out var result), response.GetRawText()); return result; }
    internal static void Error(JsonElement response, int code) => Assert.Equal(code, response.GetProperty("error").GetProperty("code").GetInt32());

    [Fact]
    public async Task Definition_references_symbols_and_preparation_use_unsaved_UTF16_text_and_versioned_rename_edits()
    {
        using var directory = sdk.CreateProject();
        const string disk = "unit Demo.Bridge; function Old(): int; begin return 42; end function; end unit;";
        await Prepare(directory, disk); var path = directory.File("Game/Bridge.fpas");
        const string source = "unit Demo.Bridge;\r\nfunction Answer(): int; begin return 42; end function;\r\nfunction Run(): int; begin const Label: string := '😀'; return Answer(); end function;\r\nend unit;";
        await using var client = new LspProbe(); await client.Initialize(directory.File("Game/Game.fpasproj")); await client.Notify("initialized", new { }); await client.Open(path, source, 7);
        var definition = Result(await client.Request("textDocument/definition", new { textDocument = Document(path), position = Position(source, "Answer();", true) }));
        LspProbe.AssertFileUri(path, definition.GetProperty("uri")); Assert.Equal(1, definition.GetProperty("range").GetProperty("start").GetProperty("line").GetInt32());
        var refs = Result(await client.Request("textDocument/references", new { textDocument = Document(path), position = Position(source, "Answer():"), context = new { includeDeclaration = true } }));
        Assert.Equal(2, refs.GetArrayLength()); Assert.Contains(refs.EnumerateArray(), location => location.GetProperty("range").GetProperty("start").GetProperty("character").GetInt32() == source.Split("\r\n")[2].IndexOf("Answer", StringComparison.Ordinal));
        var symbols = Result(await client.Request("workspace/symbol", new { query = "" })); Assert.Contains(symbols.EnumerateArray(), symbol => symbol.GetProperty("name").GetString() == "Answer");
        var prepared = Result(await client.Request("textDocument/prepareRename", new { textDocument = Document(path), position = Position(source, "Answer();", true) }));
        Assert.Equal("Answer", prepared.GetProperty("placeholder").GetString()); Assert.Equal(2, prepared.GetProperty("range").GetProperty("start").GetProperty("line").GetInt32());
        var edit = Result(await client.Request("textDocument/rename", new { textDocument = Document(path), position = Position(source, "Answer():"), newName = "type" }));
        var changes = edit.GetProperty("documentChanges"); Assert.Equal(7, changes[0].GetProperty("textDocument").GetProperty("version").GetInt32());
        Assert.All(changes[0].GetProperty("edits").EnumerateArray(), item => Assert.Equal("@type", item.GetProperty("newText").GetString()));
        Assert.False(edit.TryGetProperty("changes", out _)); Assert.Equal(disk, File.ReadAllText(path)); await client.Stop();
    }

    [Fact]
    public async Task Partial_analysis_allows_checked_definitions_but_rejects_incomplete_searches_and_rename()
    {
        using var directory = sdk.CreateProject(); const string source = "unit Demo.Bridge; function Answer(): int; begin return 42; end function; function Broken(): int; begin return Missing; end function; end unit;";
        await Prepare(directory, source); var path = directory.File("Game/Bridge.fpas");
        await using var client = new LspProbe(); await client.Initialize(directory.File("Game/Game.fpasproj")); await client.Notify("initialized", new { }); await client.Open(path, source);
        Assert.Equal(JsonValueKind.Object, Result(await client.Request("textDocument/definition", new { textDocument = Document(path), position = Position(source, "Answer():") })).ValueKind);
        Error(await client.Request("textDocument/references", new { textDocument = Document(path), position = Position(source, "Answer():"), context = new { includeDeclaration = false } }), -32803);
        Error(await client.Request("workspace/symbol", new { query = "Absent" }), -32803);
        Error(await client.Request("textDocument/prepareRename", new { textDocument = Document(path), position = Position(source, "Answer():") }), -32803);
        Error(await client.Request("textDocument/definition", new { textDocument = Document(path), position = Position(source, "Missing") }), -32803);
        await client.Stop();
    }

    [Fact]
    public async Task Formatting_is_syntax_only_supports_unowned_open_files_and_rejects_invalid_syntax()
    {
        using var directory = sdk.CreateProject(); var path = directory.File("unsaved ü.fpas"); const string source = "unit Loose;function A():int;begin return 42;end function;end unit;";
        await using var client = new LspProbe(); await client.Initialize(directory.File("Game/Game.fpasproj")); await client.Notify("initialized", new { }); await client.Open(path, source);
        var edits = Result(await client.Request("textDocument/formatting", new { textDocument = Document(path), options = new { tabSize = 8, insertSpaces = false } }));
        Assert.Single(edits.EnumerateArray()); Assert.Contains("function A(): int;", edits[0].GetProperty("newText").GetString());
        Assert.Equal(source.Length, edits[0].GetProperty("range").GetProperty("end").GetProperty("character").GetInt32());
        await client.FullChange(path, "unit Loose;", 2);
        Error(await client.Request("textDocument/formatting", new { textDocument = Document(path), options = new { tabSize = 2, insertSpaces = true } }), -32803);
        Assert.False(File.Exists(path)); await client.Stop();
    }

    [Fact]
    public async Task Rename_requires_documentChanges_support_and_invalid_parameters_do_not_break_the_connection()
    {
        using var directory = sdk.CreateProject(); const string source = "unit Demo.Bridge; function A(): int; begin return 42; end function; end unit;";
        await Prepare(directory, source); var path = directory.File("Game/Bridge.fpas"); await using var client = new LspProbe();
        var initialized = Result(await client.InitializeResponse(directory.File("Game/Game.fpasproj"), rename: false));
        Assert.False(initialized.GetProperty("capabilities").TryGetProperty("renameProvider", out _));
        await client.Notify("initialized", new { }); await client.Open(path, source);
        Error(await client.Request("textDocument/rename", new { textDocument = Document(path), position = Position(source, "A():"), newName = "B" }), -32601);
        Error(await client.Request("textDocument/definition", new { textDocument = Document(path), position = new { line = -1, character = 0 } }), -32602);
        Error(await client.Request("textDocument/references", new { textDocument = Document(path), position = Position(source, "A():"), context = new { includeDeclaration = "yes" } }), -32602);
        Assert.Equal(JsonValueKind.Null, Result(await client.Request("textDocument/definition", new { textDocument = Document(path), position = new { line = 0, character = 0 } })).ValueKind);
        await client.Stop();
    }

    [Fact]
    public async Task Unwatched_disk_changes_are_rejected_and_refresh_the_next_semantic_request()
    {
        using var directory = sdk.CreateProject(); const string source = "unit Demo.Bridge; function Answer(): int; begin return 42; end function; end unit;";
        await Prepare(directory, source); var path = directory.File("Game/Bridge.fpas");
        await using var client = new LspProbe(); await client.Initialize(directory.File("Game/Game.fpasproj")); await client.Notify("initialized", new { });
        var marker = directory.File("marker.fpas"); await client.Open(marker, "unit Marker; end unit;"); await client.Diagnostics(marker, 1);
        Assert.Single(Result(await client.Request("workspace/symbol", new { query = "Answer" })).EnumerateArray());
        File.WriteAllText(path, source.Replace("Answer", "Fetch", StringComparison.Ordinal));
        Error(await client.Request("workspace/symbol", new { query = "Answer" }), -32801);
        Assert.Single(Result(await client.Request("workspace/symbol", new { query = "Fetch" })).EnumerateArray());
        await client.Stop();
    }
}
