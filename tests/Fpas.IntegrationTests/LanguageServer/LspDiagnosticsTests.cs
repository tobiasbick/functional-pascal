using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using System.Text.Json;

namespace Fpas.IntegrationTests.LanguageServer;

public sealed class LspDiagnosticsTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    private const string Source = "unit Demo.Bridge;\r\npublic function Read(): int;\r\nbegin\r\n  const Text: string := '😀'; return Missing;\r\nend function;\r\nend unit;";

    private async Task Prepare(TemporaryDirectory project)
    {
        File.WriteAllText(project.File("Game/Game.fpasproj"), $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Library</OutputType></PropertyGroup></Project>");
        File.WriteAllText(project.File("Game/Bridge.fpas"), Source.Replace("Missing", "42", StringComparison.Ordinal));
        var result = await sdk.Run(project, "restore", "Game/Game.fpasproj"); Assert.True(result.ExitCode == 0, result.Output + result.Error);
    }

    [Fact]
    public async Task Unsaved_UTF16_incremental_changes_save_close_and_reopen_publish_current_versions_without_writing()
    {
        using var project = sdk.CreateProject(); await Prepare(project); var path = project.File("Game/Bridge.fpas");
        var disk = File.ReadAllBytes(path);
        // Exercise the installed SDK's shipped CLI rather than a server object in this process.
        var location = await sdk.Run(project, "msbuild", "Game/Game.fpasproj", "-nologo", "-getProperty:NuGetPackageRoot");
        Assert.Equal(0, location.ExitCode);
        var cli = Path.Combine(location.Output.Trim(), "fpas.net.sdk", sdk.Version.ToLowerInvariant(), "tools", "compiler", "Fpas.Cli.dll");
        await using var client = new LspProbe(cli);
        await client.Initialize(project.File("Game/Game.fpasproj")); await client.Notify("initialized", new { }); await client.Open(path, Source, 7);
        var broken = Values(await client.Diagnostics(path, 7));
        var missing = broken.EnumerateArray().Single(value => value.GetProperty("message").GetString()!.Contains("Missing", StringComparison.Ordinal));
        Assert.Equal("fpas", missing.GetProperty("source").GetString()); Assert.Equal(1, missing.GetProperty("severity").GetInt32());
        Assert.Equal(JsonValueKind.String, missing.GetProperty("code").ValueKind);
        var character = Source.Split("\r\n")[3].IndexOf("Missing", StringComparison.Ordinal);
        var start = missing.GetProperty("range").GetProperty("start"); Assert.Equal(3, start.GetProperty("line").GetInt32()); Assert.Equal(character, start.GetProperty("character").GetInt32());
        await client.Notify("textDocument/didChange", new { textDocument = new { uri = new Uri(path).AbsoluteUri, version = 8 }, contentChanges = new[]
            { new { range = new { start = new { line = 3, character }, end = new { line = 3, character = character + 7 } }, text = "42" } } });
        Assert.Empty(Values(await client.Diagnostics(path, 8)).EnumerateArray());
        await client.Notify("textDocument/didSave", new { textDocument = new { uri = new Uri(path).AbsoluteUri }, text = Source });
        Assert.Empty(Values(await client.Diagnostics(path, 8)).EnumerateArray());
        await client.Close(path);
        await client.Open(path, Source, 1);
        var closed = await client.Notification("textDocument/publishDiagnostics", message => !message.GetProperty("params").TryGetProperty("version", out _));
        Assert.Empty(Values(closed).EnumerateArray()); Assert.NotEmpty(Values(await client.Diagnostics(path, 1)).EnumerateArray());
        Assert.Equal(disk, File.ReadAllBytes(path)); Assert.Empty(Directory.GetFiles(project.File("Game"), "*.dll", SearchOption.AllDirectories));
        Assert.Empty(Directory.GetFiles(project.File("Game"), "*.pdb", SearchOption.AllDirectories));
        await client.Stop();
    }

    [Fact]
    public async Task An_unsaved_mixed_logical_chain_reports_the_offending_operator()
    {
        using var project = sdk.CreateProject();
        await Prepare(project);
        var path = project.File("Game/Bridge.fpas");
        var disk = File.ReadAllText(path);
        const string statement = "  const Text: string := '😀'; if true and false or true then return 1; end if; return 42;";
        var broken = disk.Replace("  const Text: string := '😀'; return 42;", statement, StringComparison.Ordinal);
        await using var client = new LspProbe();
        await client.Initialize(project.File("Game/Game.fpasproj"));
        await client.Notify("initialized", new { });
        await client.Open(path, disk);
        Assert.Empty(Values(await client.Diagnostics(path, 1)).EnumerateArray());
        await client.FullChange(path, broken, 2);
        var error = Assert.Single(Values(await client.Diagnostics(path, 2)).EnumerateArray());
        Assert.Equal("FP2003", error.GetProperty("code").GetString());
        var start = error.GetProperty("range").GetProperty("start");
        Assert.Equal(3, start.GetProperty("line").GetInt32());
        Assert.Equal(statement.IndexOf("or true", StringComparison.Ordinal), start.GetProperty("character").GetInt32());
        await client.Stop();
    }

    [Fact]
    public async Task Stale_versions_invalid_ranges_and_oversized_text_clear_diagnostics_until_full_resynchronization()
    {
        using var project = sdk.CreateProject(); await Prepare(project); var path = project.File("Game/Bridge.fpas");
        await using var client = new LspProbe(); await client.Initialize(project.File("Game/Game.fpasproj")); await client.Notify("initialized", new { });
        await client.Open(path, Source); Assert.NotEmpty(Values(await client.Diagnostics(path, 1)).EnumerateArray());
        await client.FullChange(path, "invalid stale text", 1);
        Assert.Empty(Values(await client.Diagnostics(path, 1)).EnumerateArray());
        Assert.Contains("version", (await client.Notification("window/logMessage")).GetProperty("params").GetProperty("message").GetString(), StringComparison.OrdinalIgnoreCase);
        await client.FullChange(path, Source, 2); Assert.NotEmpty(Values(await client.Diagnostics(path, 2)).EnumerateArray());
        await client.Notify("textDocument/didChange", new { textDocument = new { uri = new Uri(path).AbsoluteUri, version = 3 }, contentChanges = new[]
            { new { range = new { start = new { line = 99, character = 0 }, end = new { line = 99, character = 0 } }, text = "x" } } });
        Assert.Empty(Values(await client.Diagnostics(path, 2)).EnumerateArray()); await client.Notification("window/logMessage");
        await client.FullChange(path, new string('x', 4_194_305), 3);
        await client.Notification("window/logMessage", message => message.GetProperty("params").GetProperty("message").GetString()!.Contains("budget", StringComparison.Ordinal));
        await client.FullChange(path, Source.Replace("Missing", "42", StringComparison.Ordinal), 4);
        Assert.Empty(Values(await client.Diagnostics(path, 4)).EnumerateArray()); await client.Stop();
    }

    [Fact]
    public async Task Watched_disk_changes_refresh_diagnostics_for_open_dependants()
    {
        using var project = sdk.CreateProject(); await Prepare(project);
        var dependency = project.File("Game/Bridge.fpas"); var path = project.File("Game/Consumer.fpas");
        const string source = "unit Consumer; uses Demo.Bridge as B; public function Run(): int; begin return B.Read(); end function; end unit;";
        File.WriteAllText(path, source);
        await using var client = new LspProbe(); await client.Initialize(project.File("Game/Game.fpasproj")); await client.Notify("initialized", new { }); await client.Open(path, source);
        Assert.Empty(Values(await client.Diagnostics(path, 1)).EnumerateArray());
        File.WriteAllText(dependency, File.ReadAllText(dependency).Replace("Read()", "Read(Value: int)", StringComparison.Ordinal));
        await client.Notify("workspace/didChangeWatchedFiles", new { changes = new[] { new { uri = new Uri(dependency).AbsoluteUri, type = 2 } } });
        Assert.NotEmpty(Values(await client.Diagnostics(path, 1)).EnumerateArray());
        Assert.Equal(source, File.ReadAllText(path)); await client.Stop();
    }

    [Fact]
    public async Task Missing_assets_are_reported_as_setup_failures_without_restore_or_fabricated_ranges()
    {
        using var project = sdk.CreateProject(); await Prepare(project);
        Directory.Delete(project.File("Game/obj"), recursive: true);
        await using var client = new LspProbe(); await client.Initialize(project.File("Game/Game.fpasproj")); await client.Notify("initialized", new { });
        await client.Open(project.File("Game/Bridge.fpas"), Source);
        var notification = await client.Notification("window/logMessage"); var parameters = notification.GetProperty("params");
        Assert.False(parameters.TryGetProperty("range", out _)); Assert.Contains("restore", parameters.GetProperty("message").GetString(), StringComparison.OrdinalIgnoreCase);
        Assert.False(File.Exists(project.File("Game/obj/project.assets.json"))); await client.Stop();
    }

    private static JsonElement Values(JsonElement message) => message.GetProperty("params").GetProperty("diagnostics");

    [Fact]
    public async Task Syntax_errors_preserve_independent_body_diagnostics_and_repairs_replace_the_current_version()
    {
        using var project = sdk.CreateProject(); await Prepare(project); var path = project.File("Game/Bridge.fpas"); var original = File.ReadAllBytes(path);
        const string broken = "unit Demo.Bridge; public function Broken(): int; begin return Target(; end function; " +
            "function Other(): int; begin return Unknown; end function; public function Target(): int; begin return 42; end function; end unit;";
        await using var client = new LspProbe(); await client.Initialize(project.File("Game/Game.fpasproj")); await client.Notify("initialized", new { }); await client.Open(path, broken);
        var first = Values(await client.Diagnostics(path, 1)).EnumerateArray().ToArray();
        Assert.Contains(first, value => value.GetProperty("code").GetString() == "FP2003");
        Assert.Contains(first, value => value.GetProperty("code").GetString() == "FP3001" && value.GetProperty("message").GetString()!.Contains("Unknown", StringComparison.Ordinal));
        var syntaxFixed = broken.Replace("Target(;", "Target();", StringComparison.Ordinal); await client.FullChange(path, syntaxFixed, 2);
        Assert.Equal("FP3001", Values(await client.Diagnostics(path, 2)).EnumerateArray().Single().GetProperty("code").GetString());
        await client.FullChange(path, syntaxFixed.Replace("Unknown", "42", StringComparison.Ordinal), 3);
        Assert.Empty(Values(await client.Diagnostics(path, 3)).EnumerateArray()); Assert.Equal(original, File.ReadAllBytes(path)); await client.Stop();
    }

    [Fact]
    public async Task Related_declarations_and_expected_types_keep_structured_diagnostic_details()
    {
        using var project = sdk.CreateProject(); await Prepare(project); var path = project.File("Game/Bridge.fpas");
        const string duplicate = "unit Demo.Bridge; function Read(): int; begin const Value: int := 1; const Value: int := 2; return Value; end function; end unit;";
        await using var client = new LspProbe(); await client.Initialize(project.File("Game/Game.fpasproj")); await client.Notify("initialized", new { });
        await client.Open(path, duplicate);
        var diagnostic = Values(await client.Diagnostics(path, 1)).EnumerateArray().Single();
        Assert.True(diagnostic.TryGetProperty("relatedInformation", out var related), diagnostic.GetRawText());
        var previous = related.EnumerateArray().Single();
        LspProbe.AssertFileUri(path, previous.GetProperty("location").GetProperty("uri"));
        Assert.Equal(duplicate.IndexOf("Value", StringComparison.Ordinal), previous.GetProperty("location").GetProperty("range").GetProperty("start").GetProperty("character").GetInt32());
        await client.FullChange(path, "unit Demo.Bridge; function Read(): int; begin const Value: int := 'text'; return Value; end function; end unit;", 2);
        var conversion = Values(await client.Diagnostics(path, 2)).EnumerateArray().Single();
        var expected = conversion.GetProperty("data").GetProperty("expectation");
        Assert.Equal("type", expected.GetProperty("kind").GetString());
        Assert.Equal("int", expected.GetProperty("expected")[0].GetString()); Assert.Equal("string", expected.GetProperty("found").GetString());
        await client.Stop();
    }
}
