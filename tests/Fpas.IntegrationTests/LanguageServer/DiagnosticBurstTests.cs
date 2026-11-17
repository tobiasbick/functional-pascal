using Fpas.IntegrationTests.Build;

namespace Fpas.IntegrationTests.LanguageServer;

public sealed class DiagnosticBurstTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Diagnostics_for_one_hundred_open_documents_preserve_the_session()
    {
        using var project = sdk.CreateProject();
        var context = project.File("Game/Game.fpasproj");
        File.WriteAllText(context, $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Library</OutputType></PropertyGroup></Project>");
        File.WriteAllText(project.File("Game/Bridge.fpas"), "unit Demo.Bridge; public function Answer(): int; begin return 42; end function; end unit;");
        var restored = await sdk.Run(project, "restore", "Game/Game.fpasproj");
        Assert.True(restored.ExitCode == 0, restored.Output + restored.Error);

        await using var client = new LspProbe();
        await client.Initialize(context);
        await client.Notify("initialized", new { });
        var paths = Enumerable.Range(0, 100).Select(index => project.File($"Loose{index:D3}.fpas")).ToArray();
        // Unowned buffers receive real syntax diagnostics without enlarging the compilation.
        foreach (var path in paths) await client.Open(path, "unit Loose;");

        var remaining = paths.ToHashSet(OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal);
        await client.Notification("textDocument/publishDiagnostics", message =>
        {
            var parameters = message.GetProperty("params");
            if (!parameters.TryGetProperty("version", out var version) || version.GetInt32() != 1) return false;
            var path = remaining.FirstOrDefault(path => LspProbe.IsFileUri(parameters.GetProperty("uri"), path));
            if (path is null) return false;
            Assert.NotEmpty(parameters.GetProperty("diagnostics").EnumerateArray());
            remaining.Remove(path);
            return remaining.Count == 0;
        });

        var response = await client.Request("workspace/symbol", new { query = "Answer" });
        Assert.True(response.TryGetProperty("result", out var symbols), response.GetRawText());
        Assert.Contains(symbols.EnumerateArray(), symbol => symbol.GetProperty("name").GetString() == "Answer");
        await client.Stop();
    }
}
