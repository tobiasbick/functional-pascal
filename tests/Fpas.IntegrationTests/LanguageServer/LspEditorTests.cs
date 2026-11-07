using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using System.Text.Json;
using static Fpas.IntegrationTests.LanguageServer.LspFeatureTests;

namespace Fpas.IntegrationTests.LanguageServer;

public sealed class LspEditorTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    private const string Library = """
        unit Demo.Bridge;
        public type Box = record
          public Value: int;
          private Secret: int;
        end record;
        /// Returns the selected value.
        public function Echo<T>(Value: T): T;
        begin
          return Value;
        end function;
        public function Read(Value: Box): int;
        begin
          var @type: int := 0;
          @type := Value.Value;
          const Label: string := '😀'; return Echo(@type);
        end function;
        end unit;
        """;

    [Fact]
    public async Task Outline_navigation_tokens_call_hierarchy_and_views_use_the_checked_overlay()
    {
        using var directory = sdk.CreateProject();
        var (project, path) = await Prepare(directory);
        var source = Library.ReplaceLineEndings("\r\n");
        await using var client = new LspProbe();
        var capabilities = Result(await client.InitializeResponse(project)).GetProperty("capabilities");
        await client.Notify("initialized", new { });
        await client.Open(path, source);
        var document = Document(path);
        var point = Position(source, "Echo(@type)");
        var outline = Result(await client.Request("textDocument/documentSymbol", new { textDocument = document }));
        Assert.Equal("Demo.Bridge", outline[0].GetProperty("name").GetString());
        var children = outline[0].GetProperty("children").EnumerateArray().ToArray();
        Assert.Contains(children, item => item.GetProperty("name").GetString() == "Box" && item.GetProperty("children").GetArrayLength() == 2);
        Assert.True(Result(await client.Request("textDocument/foldingRange", new { textDocument = document })).GetArrayLength() >= 3);
        var selection = Result(await client.Request("textDocument/selectionRange", new { textDocument = document, positions = new[] { point } }))[0];
        Assert.True(selection.TryGetProperty("parent", out _));
        var hover = Result(await client.Request("textDocument/hover", new { textDocument = document, position = point }));
        Assert.Contains("Echo(Value: int): int", hover.GetProperty("contents").GetProperty("value").GetString());
        Assert.Contains("selected value", hover.GetProperty("contents").GetProperty("value").GetString());
        var type = Result(await client.Request("textDocument/typeDefinition", new { textDocument = document, position = Position(source, "Value.Value") }));
        Assert.Equal(1, type.GetProperty("range").GetProperty("start").GetProperty("line").GetInt32());
        var highlights = Result(await client.Request("textDocument/documentHighlight", new { textDocument = document, position = Position(source, "@type);") }));
        Assert.Equal(new[] { 1, 3, 2 }, highlights.EnumerateArray().Select(item => item.GetProperty("kind").GetInt32()));
        var semantic = Result(await client.Request("textDocument/semanticTokens/full", new { textDocument = document })).GetProperty("data");
        var legend = capabilities.GetProperty("semanticTokensProvider").GetProperty("legend").GetProperty("tokenTypes");
        var function = Array.FindIndex(legend.EnumerateArray().ToArray(), item => item.GetString() == "function");
        Assert.Contains(DecodeTokens(semantic), item => item == (14, source.Split("\r\n")[14].IndexOf("Echo", StringComparison.Ordinal), 4, function));

        var call = Result(await client.Request("textDocument/prepareCallHierarchy", new { textDocument = document, position = point }))[0];
        var incoming = Result(await client.Request("callHierarchy/incomingCalls", new { item = call }));
        Assert.Equal("Read", incoming[0].GetProperty("from").GetProperty("name").GetString());
        var outgoing = Result(await client.Request("callHierarchy/outgoingCalls", new { item = incoming[0].GetProperty("from") }));
        Assert.Equal("Echo", outgoing[0].GetProperty("to").GetProperty("name").GetString());
        var version = new { uri = new Uri(path).AbsoluteUri, version = 1 };
        var signatures = Result(await client.Request("fpas/publicSignatures", new { schemaVersion = 1, textDocument = version }));
        Assert.True(signatures.GetProperty("value").GetProperty("complete").GetBoolean());
        Assert.Contains("Echo", signatures.GetProperty("value").GetProperty("declarations").GetRawText());
        var inspection = Result(await client.Request("fpas/inspect", new { schemaVersion = 1, textDocument = version, position = point }));
        Assert.Contains("return Value;", inspection.GetProperty("value").GetProperty("inspect").GetProperty("source").GetProperty("body").GetProperty("text").GetString());
        var metadata = Result(await client.Request("fpas/searchMetadata", new { schemaVersion = 1, textDocument = version, query = "System.Math.Abs" }));
        var entries = metadata.GetProperty("value").GetProperty("entries").EnumerateArray().ToArray();
        Assert.True(entries.Length > 1);
        Assert.Equal(entries.Length, entries.Select(item => item.GetProperty("identity").GetString()).Distinct().Count());
        Assert.All(entries, item => Assert.Equal("System.Math.Abs", item.GetProperty("qualifiedName").GetString()));

        var broken = source.Replace("Echo(@type)", "Unknown(", StringComparison.Ordinal);
        await client.FullChange(path, broken, 2);
        await client.Notification("fpas/viewsInvalidated");
        Error(await client.Request("callHierarchy/incomingCalls", new { item = call }), -32801);
        Error(await client.Request("fpas/publicSignatures", new { schemaVersion = 1, textDocument = version }), -32801);
        var recovered = Result(await client.Request("textDocument/documentSymbol", new { textDocument = document }));
        Assert.Contains("Echo", recovered.GetRawText());
        var tokens = DecodeTokens(Result(await client.Request("textDocument/semanticTokens/full", new { textDocument = document })).GetProperty("data"));
        Assert.DoesNotContain(tokens, item => item.Line == 14 && item.Column == broken.Split("\r\n")[14].IndexOf("Unknown", StringComparison.Ordinal));
        Assert.Equal(Library, File.ReadAllText(path));
        await client.Stop();
    }

    [Fact]
    public async Task Incomplete_member_and_named_call_use_visible_checked_candidates()
    {
        using var directory = sdk.CreateProject();
        var (project, path) = await Prepare(directory);
        var source = Library.Replace("Value.Value;", "Value.;", StringComparison.Ordinal);
        await using var client = new LspProbe();
        await client.Initialize(project);
        await client.Notify("initialized", new { });
        await client.Open(path, source);
        var document = Document(path);
        var completion = Result(await client.Request("textDocument/completion", new
            { textDocument = document, position = Position(source, ";\n  const Label") })).GetProperty("items");
        var field = Assert.Single(completion.EnumerateArray(), item => item.GetProperty("label").GetString() == "Value");
        Assert.DoesNotContain(completion.EnumerateArray(), item => item.GetProperty("label").GetString() == "Secret");
        var incomplete = Library.Replace("Echo(@type)", "Echo(Value := @type", StringComparison.Ordinal);
        await client.FullChange(path, incomplete, 2);
        Error(await client.Request("completionItem/resolve", field), -32801);
        var signature = Result(await client.Request("textDocument/signatureHelp", new { textDocument = document, position = Position(incomplete, ";\nend function;", true) }));
        Assert.Equal(0, signature.GetProperty("activeParameter").GetInt32());
        Assert.Contains("Echo<int>", signature.GetProperty("signatures")[0].GetProperty("label").GetString());
        await client.Stop();
    }

    [Fact]
    public async Task Import_requires_a_single_verified_receiver_and_resolves_to_versioned_edits()
    {
        using var directory = sdk.CreateProject();
        var (project, path) = await Prepare(directory);
        const string source = "unit Demo.Bridge; public function Read(): int; begin return Math.Abs(-1); end function; end unit;";
        await using var client = new LspProbe();
        await client.Initialize(project);
        await client.Notify("initialized", new { });
        await client.Open(path, source, 7);
        var document = Document(path);
        var parameters = new { textDocument = document, range = new { start = Position(source, "Math"), end = Position(source, ".Abs") },
            context = new { diagnostics = Array.Empty<object>(), only = new[] { "quickfix" } } };
        var actions = Result(await client.Request("textDocument/codeAction", parameters));
        var action = Assert.Single(actions.EnumerateArray());
        Assert.Contains("System.Math", action.GetProperty("title").GetString());
        var resolved = Result(await client.Request("codeAction/resolve", action));
        var edits = resolved.GetProperty("edit").GetProperty("documentChanges");
        Assert.Equal(7, edits[0].GetProperty("textDocument").GetProperty("version").GetInt32());
        Assert.Contains("uses System.Math as Math;", edits[0].GetProperty("edits")[0].GetProperty("newText").GetString());
        await client.FullChange(path, source.Replace("Math.Abs", "Math.Unavailable", StringComparison.Ordinal), 8);
        Error(await client.Request("codeAction/resolve", action), -32801);
        Assert.Empty(Result(await client.Request("textDocument/codeAction", parameters)).EnumerateArray());
        const string existingAlias = "unit Demo.Bridge; uses System.Convert as Math; public function Read(): int; begin return Math.ToInt32('1') + Math.Abs(-1); end function; end unit;";
        await client.FullChange(path, existingAlias, 9);
        Assert.Empty(Result(await client.Request("textDocument/codeAction", new
        {
            textDocument = document, range = new { start = Position(existingAlias, "Math.Abs"), end = Position(existingAlias, ".Abs") },
            parameters.context
        })).EnumerateArray());
        await client.Stop();
    }

    [Fact]
    public async Task Superseded_parallel_editor_requests_leave_the_session_and_diagnostics_responsive()
    {
        using var directory = sdk.CreateProject();
        var (project, path) = await Prepare(directory);
        await using var client = new LspProbe();
        await client.Initialize(project);
        await client.Notify("initialized", new { });
        await client.Open(path, Library);
        var document = Document(path);
        var position = Position(Library, "Echo(@type)");
        var methods = new[] { "textDocument/hover", "textDocument/typeDefinition", "textDocument/documentHighlight",
            "textDocument/semanticTokens/full", "textDocument/documentSymbol" };
        var pending = new List<Task<JsonElement>>();
        for (var version = 2; version <= 7; version++)
        {
            foreach (var method in methods)
                pending.Add(client.Request(method, new { textDocument = document, position }, version + ":" + method));
            await client.FullChange(path, Library + new string('\n', version), version);
            await client.Notify("$/cancelRequest", new { id = version + ":" + methods[0] });
        }
        var responses = await Task.WhenAll(pending).WaitAsync(TimeSpan.FromSeconds(30));
        Assert.All(responses, response =>
        {
            if (response.TryGetProperty("result", out _)) return;
            Assert.Contains(response.GetProperty("error").GetProperty("code").GetInt32(), new[] { -32800, -32801 });
        });
        var diagnostics = await client.Diagnostics(path, 7);
        Assert.Empty(diagnostics.GetProperty("params").GetProperty("diagnostics").EnumerateArray());
        Assert.Contains("Echo", Result(await client.Request("textDocument/documentSymbol", new { textDocument = document })).GetRawText());
        await client.Stop();
    }

    private async Task<(string Project, string Source)> Prepare(TemporaryDirectory directory)
    {
        var project = directory.File("Game/Game.fpasproj");
        var source = directory.File("Game/Bridge.fpas");
        File.WriteAllText(project, $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Library</OutputType></PropertyGroup></Project>");
        File.WriteAllText(source, Library);
        var restored = await sdk.Run(directory, "restore", "Game/Game.fpasproj");
        Assert.True(restored.ExitCode == 0, restored.Output + restored.Error);
        return (project, source);
    }

    private static IReadOnlyList<(int Line, int Column, int Length, int Kind)> DecodeTokens(JsonElement value)
    {
        var numbers = value.EnumerateArray().Select(item => item.GetInt32()).ToArray();
        Assert.Equal(0, numbers.Length % 5);
        var tokens = new List<(int, int, int, int)>();
        var line = 0; var column = 0;
        for (var index = 0; index < numbers.Length; index += 5)
        {
            line += numbers[index];
            column = numbers[index] == 0 ? column + numbers[index + 1] : numbers[index + 1];
            tokens.Add((line, column, numbers[index + 2], numbers[index + 3]));
        }
        return tokens;
    }
}
