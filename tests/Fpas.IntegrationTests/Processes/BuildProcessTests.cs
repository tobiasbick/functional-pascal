using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Processes;

/// <summary>Checks bounded capture and live build output through the actual dotnet host.</summary>
public sealed class BuildProcessTests
{
    /// <summary>Streams all build output while keeping report capture bounded.</summary>
    [Fact]
    public async Task LiveOutputKeepsTheEndOfALargeBuildLog()
    {
        using var directory = new TemporaryDirectory();
        var project = directory.File("Output.proj");
        var messages = Enumerable.Range(0, 260).Select(index =>
            $"    <Message Text=\"{index:D3} {new string('x', 4096)}\" Importance=\"high\" />");
        await File.WriteAllTextAsync(project, $"""
            <Project>
              <Target Name="Print">
            {string.Join('\n', messages)}
                <Message Text="FPAS output complete" Importance="high" />
              </Target>
            </Project>
            """);
        string[] arguments = ["msbuild", project, "-nologo", "-t:Print"];
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        using var stdout = new StringWriter();
        using var stderr = new StringWriter();
        var streamed = await BuildProcess.Run(directory.Path, arguments, deadline.Token, liveOutput: new(stdout, stderr));
        Assert.Equal(0, streamed.ExitCode);
        Assert.False(streamed.Truncated);
        Assert.Empty(streamed.Output);
        Assert.Contains("FPAS output complete", stdout.ToString());

        var captured = await BuildProcess.Run(directory.Path, arguments, deadline.Token);
        Assert.Equal(0, captured.ExitCode);
        Assert.True(captured.Truncated);
        Assert.True(captured.Output.Length <= 1_048_576);
        Assert.DoesNotContain("FPAS output complete", captured.Output);
    }
}
