using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using System.Text.Json;
using System.Xml.Linq;
using static Fpas.IntegrationTests.LanguageServer.LspFeatureTests;

namespace Fpas.IntegrationTests.LanguageServer;

/// <summary>Checks shared source Markdown and literal imported documentation across CLI and editor features.</summary>
/// <param name="sdk">The isolated SDK package fixture used by external consumers.</param>
public sealed class LspDocumentationTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    private const string Markdown = """
        Returns the supplied value.

        ## Parameters
        - `Value`: The requested value.

        ## Remarks
        - Outer
          - Inner

        ```text
          List<int> & text
        ```
        """;

    /// <summary>Preserve the same Markdown across consumers and recovery edits.</summary>
    /// <param name="newline">The original source line ending.</param>
    /// <param name="declarationAfterUse">Whether recovery must restore a later declaration's offset.</param>
    [Theory]
    [InlineData("\n", false)]
    [InlineData("\r\n", false)]
    [InlineData("\r", false)]
    [InlineData("\n", true)]
    public async Task Source_Markdown_is_identical_in_CLI_hover_completion_and_signature_help(string newline, bool declarationAfterUse)
    {
        using var directory = sdk.CreateProject();
        var comments = string.Join('\n', Markdown.Split('\n').Select(line => "  //" + (line.Length == 0 ? "" : " " + line)));
        var declaration = comments + "\n" + """
            public function Answer(Value: int): int;
            begin return Value; end function;
            """;
        const string caller = "public function Run(): int; begin return Answer(42); end function;";
        var source = ("unit Demo.Bridge;\n" + (declarationAfterUse ? caller + "\n" + declaration : declaration + "\n" + caller) + "\nend unit;").Replace("\n", newline, StringComparison.Ordinal);
        var path = directory.File("Game/Bridge.fpas"); File.WriteAllText(path, source);
        var project = directory.File("Game/Game.fpasproj");
        var cli = await Doc(directory, project, "Demo.Bridge.Answer");
        Assert.Equal(0, cli.ExitCode); Assert.Empty(cli.Error);
        using var report = JsonDocument.Parse(cli.Output);
        var entry = Assert.Single(report.RootElement.GetProperty("result").GetProperty("entries").EnumerateArray());
        Assert.Equal(Markdown, entry.GetProperty("documentation").GetProperty("content").GetString());

        await using var client = new LspProbe();
        await client.Initialize(project); await client.Notify("initialized", new { }); await client.Open(path, source);
        Assert.Empty((await client.Diagnostics(path, 1)).GetProperty("params").GetProperty("diagnostics").EnumerateArray());
        var selection = new { textDocument = Document(path), position = Position(source, "Answer(42)") };
        var hover = Result(await client.Request("textDocument/hover", selection));
        AssertMarkdown(hover.GetProperty("contents"));
        var completion = Result(await client.Request("textDocument/completion", selection));
        var candidate = Assert.Single(completion.GetProperty("items").EnumerateArray(), item => item.GetProperty("label").GetString() == "Answer");
        var resolved = Result(await client.Request("completionItem/resolve", candidate));
        AssertMarkdown(resolved.GetProperty("documentation"));
        var signatures = Result(await client.Request("textDocument/signatureHelp", new { textDocument = Document(path), position = Position(source, "42)") }));
        AssertMarkdown(signatures.GetProperty("signatures")[0].GetProperty("documentation"));
        await client.Stop(); Assert.Equal(source, File.ReadAllText(path));
    }

    /// <summary>Associate signature help with the selected overload's own source comment.</summary>
    [Fact]
    public async Task Signature_help_uses_the_selected_source_overloads_own_documentation()
    {
        using var directory = sdk.CreateProject();
        const string source = """
            unit Demo.Bridge;
            // Integer overload.
            public function Pick(Value: int): int; begin return Value; end function;
            // Text overload.
            public function Pick(Value: string): int; begin return Value.Length; end function;
            public function Run(): int; begin return Pick('text'); end function;
            end unit;
            """;
        var path = directory.File("Game/Bridge.fpas"); File.WriteAllText(path, source);
        var project = directory.File("Game/Game.fpasproj");
        var cli = await Doc(directory, project, "Demo.Bridge.Pick");
        Assert.Equal(0, cli.ExitCode);
        await using var client = new LspProbe(); await client.Initialize(project);
        await client.Notify("initialized", new { }); await client.Open(path, source);
        var signatures = Result(await client.Request("textDocument/signatureHelp", new { textDocument = Document(path), position = Position(source, "'text'") }));
        var items = signatures.GetProperty("signatures"); Assert.Equal(2, items.GetArrayLength());
        var active = signatures.GetProperty("activeSignature").GetInt32();
        var selected = items[active]; Assert.Contains("Value: string", selected.GetProperty("label").GetString());
        Assert.EndsWith("\n\nText overload.", selected.GetProperty("documentation").GetProperty("value").GetString());
        await client.Stop();
    }

    /// <summary>Keep imported XML literal in editor Markdown while preserving CLI XML payloads.</summary>
    [Fact]
    public async Task Imported_XML_text_remains_literal_in_editor_Markdown()
    {
        using var directory = sdk.CreateProject();
        var shared = directory.File("Shared/Shared.csproj"); var xml = XDocument.Load(shared);
        xml.Root!.Add(new XElement("PropertyGroup", new XElement("GenerateDocumentationFile", "true"), new XElement("NoWarn", "1591")));
        xml.Save(shared);
        File.WriteAllText(directory.File("Shared/Documentation.cs"), """
            namespace Demo;
            public static class Literals
            {
                /// <summary>Literal &lt;node&gt; and [link](command:example), *text*.</summary>
                public static int Read() => 42;
            }
            """);
        const string source = "unit Demo.Bridge; uses Demo.Literals as External; public function Run(): int; begin return External.Read(); end function; end unit;";
        var path = directory.File("Game/Bridge.fpas"); File.WriteAllText(path, source);
        var project = directory.File("Game/Game.fpasproj");
        var cli = await Doc(directory, project, "Demo.Literals.Read");
        Assert.Equal(0, cli.ExitCode); Assert.Empty(cli.Error);
        using var report = JsonDocument.Parse(cli.Output);
        var entry = Assert.Single(report.RootElement.GetProperty("result").GetProperty("entries").EnumerateArray());
        Assert.Equal("xml", entry.GetProperty("documentation").GetProperty("format").GetString());
        Assert.Contains("&lt;node&gt;", entry.GetProperty("documentation").GetProperty("content").GetString());
        await using var client = new LspProbe(); await client.Initialize(project);
        await client.Notify("initialized", new { }); await client.Open(path, source);
        var hover = Result(await client.Request("textDocument/hover", new { textDocument = Document(path), position = Position(source, "Read()") }));
        var content = hover.GetProperty("contents").GetProperty("value").GetString();
        Assert.Contains(@"Literal \<node\> and \[link\]\(command:example\), \*text\*\.", content);
        await client.Stop();
    }

    private static void AssertMarkdown(JsonElement contents)
    {
        Assert.Equal("markdown", contents.GetProperty("kind").GetString());
        Assert.EndsWith("\n\n" + Markdown, contents.GetProperty("value").GetString());
    }

    private static Task<ProcessResult> Doc(TemporaryDirectory directory, string project, string name)
        => ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "doc", project, "--name", name, "--report", "json"],
            workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(2));
}
