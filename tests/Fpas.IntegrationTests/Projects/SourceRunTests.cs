using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using System.Text.Json;

namespace Fpas.IntegrationTests.Projects;

public sealed class SourceRunTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task A_standalone_source_reuses_persistent_build_artifacts_and_honors_no_restore()
    {
        using var directory = new TemporaryDirectory();
        var source = directory.File("Grüße sample.fpas");
        File.WriteAllText(source, "program Sample; uses System.Console as Console; begin Console.WriteLine(args[0]); end; end program;");
        var build = await ProjectCliFixture.Cli(directory, "build", source, "--sdk-version", sdk.Version, "--sdk-feed", sdk.Feed, "--report", "json");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var assembly = directory.File("bin/Debug/net10.0/Grüße sample.dll");
        Assert.True(File.Exists(assembly));
        var direct = await ProjectCliFixture.Cli(directory, "run", assembly, "--report", "json", "--", "direct");
        Assert.Equal(0, direct.ExitCode);
        using var directJson = JsonDocument.Parse(direct.Output);
        Assert.Equal("direct" + Environment.NewLine, directJson.RootElement.GetProperty("program").GetProperty("output").GetString());
        var run = await ProjectCliFixture.Cli(directory, "run", source, "--sdk-version", sdk.Version, "--sdk-feed", sdk.Feed, "--no-restore", "--report", "json", "--", "source");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        using var json = JsonDocument.Parse(run.Output);
        Assert.Equal("source" + Environment.NewLine, json.RootElement.GetProperty("program").GetProperty("output").GetString());
        Assert.Equal(assembly, json.RootElement.GetProperty("target").GetProperty("assembly").GetString());
        Assert.True(File.Exists(assembly));
        var noBuild = await ProjectCliFixture.Cli(directory, "run", source, "--sdk-version", sdk.Version, "--sdk-feed", sdk.Feed, "--no-build", "--report", "json", "--", "existing");
        Assert.Equal(0, noBuild.ExitCode);
        using var existing = JsonDocument.Parse(noBuild.Output);
        Assert.Equal("existing" + Environment.NewLine, existing.RootElement.GetProperty("program").GetProperty("output").GetString());
        Assert.False(existing.RootElement.TryGetProperty("build", out _));
    }

    [Fact]
    public async Task Timeout_and_output_limit_are_structured()
    {
        using var directory = new TemporaryDirectory();
        var project = await ProjectCliFixture.Init(directory, sdk);
        File.WriteAllText(directory.File("App/main.fpas"), "program LongRunning; uses System.Threading.Thread as Thread; begin while true do Thread.Sleep(20); end while; end; end program;");
        var built = await ProjectCliFixture.Cli(directory, "build", project, "--report", "json"); Assert.Equal(0, built.ExitCode);
        var timeout = await ProjectCliFixture.Cli(directory, "run", project, "--no-build", "--timeout", "2", "--report", "json");
        Assert.Equal(1, timeout.ExitCode); Assert.Equal("", timeout.Error);
        using var timeoutJson = JsonDocument.Parse(timeout.Output);
        Assert.Equal("timeout", timeoutJson.RootElement.GetProperty("error").GetProperty("code").GetString());
        var source = directory.File("Loud.fpas");
        File.WriteAllText(source, "program Loud; uses System as S, System.Console as Console; begin Console.Write(new S.String(#120, 1200000)); end; end program;");
        var loud = await ProjectCliFixture.Cli(directory, "run", source, "--sdk-version", sdk.Version, "--sdk-feed", sdk.Feed, "--report", "json");
        Assert.Equal(1, loud.ExitCode); Assert.Equal("", loud.Error);
        using var json = JsonDocument.Parse(loud.Output);
        Assert.Equal("output_limit", json.RootElement.GetProperty("status").GetString());
        Assert.True(json.RootElement.GetProperty("program").GetProperty("truncated").GetBoolean());
    }

    [Theory]
    [InlineData("build", "missing.fpasproj", "--report", "json")]
    [InlineData("run", "missing.dll", "--report", "json")]
    [InlineData("run", "App.fpasproj", "--timeout", "-1", "--report", "json")]
    [InlineData("run", "App.fpasproj", "argument-without-separator", "--report", "json")]
    public async Task Invalid_selection_and_arguments_are_one_JSON_result(params string[] arguments)
    {
        using var directory = new TemporaryDirectory();
        var result = await ProjectCliFixture.Cli(directory, arguments);
        Assert.Equal(2, result.ExitCode); Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.False(json.RootElement.GetProperty("success").GetBoolean());
        Assert.Empty(Directory.EnumerateFileSystemEntries(directory.Path));
    }
}
