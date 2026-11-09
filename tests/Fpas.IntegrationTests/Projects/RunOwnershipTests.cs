using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using System.Diagnostics;
using System.Text.Json;

namespace Fpas.IntegrationTests.Projects;

public sealed class RunOwnershipTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    /// <summary>Drain output and join descendants after the main target exits in both run modes.</summary>
    /// <param name="captureFailure">Whether the run enables automatic failure capture.</param>
    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task JSON_run_drains_output_and_joins_descendants_after_the_parent_exits(bool captureFailure)
    {
        using var directory = new TemporaryDirectory();
        var source = directory.File("Parent.fpas"); var ready = directory.File("child.pid");
        File.WriteAllText(source, """
            program Parent;
            uses System as S, System.IO.File as File, System.Diagnostics as P,
              System.Threading.Thread as Thread, System.Console as Console;
            begin
              if args[0] = 'child' then
                File.WriteAllText(args[2] + '.tmp', S.Environment.ProcessId.ToString());
                File.Move(args[2] + '.tmp', args[2]);
                while true do Thread.Sleep(20); end while;
              else
                const Start: P.ProcessStartInfo := new P.ProcessStartInfo(args[1]);
                Start.UseShellExecute := false;
                Start.ArgumentList.Add(S.Environment.GetCommandLineArgs()[0]);
                Start.ArgumentList.Add('child');
                Start.ArgumentList.Add(args[1]);
                Start.ArgumentList.Add(args[2]);
                discard P.Process.Start(Start);
                while not File.Exists(args[2]) do Thread.Sleep(20); end while;
                Console.WriteLine('parent done');
              end if;
            end;
            end program;
            """);
        var result = await ProjectCliFixture.Cli(directory, ["run", source, "--sdk-version", sdk.Version, "--sdk-feed", sdk.Feed,
            "--timeout", "30", "--report", "json", .. captureFailure ? new[] { "--failure-report", directory.File("failure.json") } : [],
            "--", "parent", TestEnvironment.DotnetHost, ready]);
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.Equal("parent done" + Environment.NewLine, json.RootElement.GetProperty("program").GetProperty("output").GetString());
        Assert.False(File.Exists(directory.File("failure.json")));
        var child = int.Parse(File.ReadAllText(ready), System.Globalization.CultureInfo.InvariantCulture);
        try
        {
            using var process = Process.GetProcessById(child);
            Assert.True(process.HasExited);
        }
        catch (ArgumentException) { }
        if (OperatingSystem.IsLinux()) Assert.False(Directory.Exists("/proc/" + child));
    }
}
