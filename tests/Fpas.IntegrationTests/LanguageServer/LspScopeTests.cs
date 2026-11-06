using Fpas.IntegrationTests.Build;
using System.Text;
using static Fpas.IntegrationTests.LanguageServer.LspFeatureTests;

namespace Fpas.IntegrationTests.LanguageServer;

public sealed class LspScopeTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Linked_source_owners_are_ambiguous_and_never_select_the_first_compilation()
    {
        using var directory = sdk.CreateProject(); var path = directory.File("Game/Bridge.fpas");
        const string source = "unit Shared; public function Answer(): int; begin return 42; end function; end unit;";
        File.WriteAllText(path, source); File.WriteAllText(directory.File("Game/Game.fpasproj"), Project());
        Directory.CreateDirectory(directory.File("Other")); File.WriteAllText(directory.File("Other/Other.fpasproj"), Project("<ItemGroup><Compile Include=\"../Game/Bridge.fpas\" /></ItemGroup>"));
        var solution = File.ReadAllText(directory.File("Game.slnx")).Replace("  <Project Path=\"Shared/Shared.csproj\" />", "", StringComparison.Ordinal)
            .Replace("  <Project Path=\"Host/Host.csproj\" />", "  <Project Path=\"Other/Other.fpasproj\" />", StringComparison.Ordinal);
        File.WriteAllText(directory.File("Game.slnx"), solution);
        var restore = await sdk.Run(directory, "restore", "Game.slnx"); Assert.True(restore.ExitCode == 0, restore.Output + restore.Error);
        await using var client = new LspProbe(); await client.Initialize(directory.File("Game.slnx")); await client.Notify("initialized", new { }); await client.Open(path, source);
        var definition = await client.Request("textDocument/definition", new { textDocument = Document(path), position = Position(source, "Answer():") });
        Error(definition, -32803); Assert.Contains("ambiguous_project", definition.GetProperty("error").GetProperty("message").GetString());
        Error(await client.Request("textDocument/rename", new { textDocument = Document(path), position = Position(source, "Answer():"), newName = "Fetch" }), -32803);
        Assert.Equal(source, File.ReadAllText(path)); await client.Stop();
    }

    [Fact]
    public async Task Symbol_results_cannot_be_silently_truncated_and_large_formatting_replies_fail_per_request()
    {
        using var directory = sdk.CreateProject(); var path = directory.File("Game/Bridge.fpas");
        var source = "unit Many; " + string.Join(' ', Enumerable.Range(0, 1001).Select(index => "const Value" + index + ": int := " + index + ";")) + " end unit;";
        File.WriteAllText(path, source); File.WriteAllText(directory.File("Game/Game.fpasproj"), Project());
        var restore = await sdk.Run(directory, "restore", "Game/Game.fpasproj"); Assert.True(restore.ExitCode == 0, restore.Output + restore.Error);
        await using var client = new LspProbe(); await client.Initialize(directory.File("Game/Game.fpasproj")); await client.Notify("initialized", new { });
        var overflow = await client.Request("workspace/symbol", new { query = "" }); Error(overflow, -32803);
        Assert.Contains("1,000", overflow.GetProperty("error").GetProperty("message").GetString());
        Assert.Single(Result(await client.Request("workspace/symbol", new { query = "Value1000" })).EnumerateArray());
        var loose = directory.File("large.fpas"); await client.Open(loose, "//" + new string('\\', 2_100_000) + "\nunit Loose; end unit;");
        var oversized = await client.Request("textDocument/formatting", new { textDocument = Document(loose), options = new { tabSize = 2, insertSpaces = true } }); Error(oversized, -32803);
        Assert.Contains("4 MiB", oversized.GetProperty("error").GetProperty("message").GetString());
        await client.Stop();
    }
    private string Project(string extra = "") => $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Library</OutputType></PropertyGroup>{extra}</Project>";
}
