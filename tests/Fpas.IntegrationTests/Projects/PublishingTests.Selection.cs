using System.Text.Json;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Projects;

public sealed partial class PublishingTests
{
    [Fact]
    public async Task Solution_selects_a_foreign_host_and_no_build_reuses_matching_outputs()
    {
        using var directory = sdk.CreateProject();
        var first = await Cli(directory, "publish", "Game.slnx", "--output", directory.File("first"), "--mode", "apphost", "--report", "json");
        Assert.True(first.ExitCode == 0, first.Output + first.Error);
        using var report = JsonDocument.Parse(first.Output);
        Assert.Equal(directory.File("Host/Host.csproj"), report.RootElement.GetProperty("target").GetProperty("project").GetString());
        var reused = await Cli(directory, "publish", "Game.slnx", "--project", directory.File("Host/Host.csproj"), "--output", directory.File("reused"),
            "--mode", "apphost", "--no-build", "--no-restore", "--report", "json");
        Assert.True(reused.ExitCode == 0, reused.Output + reused.Error);
        var executable = directory.File(OperatingSystem.IsWindows() ? "reused/Host.exe" : "reused/Host");
        var ran = await ProcessRunner.RunExecutableAsync(executable, [], workingDirectory: directory.Path);
        Assert.Equal(0, ran.ExitCode); Assert.Equal("C#:FPAS\n", ran.Output.ReplaceLineEndings("\n"));
    }

    [Fact]
    public async Task Standalone_source_publication_uses_the_explicit_matching_SDK_feed()
    {
        using var directory = new TemporaryDirectory();
        var source = directory.File("Source.fpas");
        File.WriteAllText(source, "program Source; uses System.Console as Console; begin Console.WriteLine('standalone'); end; end program;");
        var result = await Cli(directory, "publish", source, "--sdk-version", sdk.Version, "--sdk-feed", sdk.Feed,
            "--mode", "apphost", "--output", directory.File("delivery"), "--report", "json");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        var ran = await ProcessRunner.RunExecutableAsync(directory.File(OperatingSystem.IsWindows() ? "delivery/Source.exe" : "delivery/Source"), [], workingDirectory: directory.Path);
        Assert.Equal(0, ran.ExitCode); Assert.Equal("standalone\n", ran.Output.ReplaceLineEndings("\n"));
    }
}
