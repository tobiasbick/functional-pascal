using System.Text.Json;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Debugging;

public sealed class TracepointCliTests
{
    [Theory]
    [InlineData(10, 0, 3, true)]
    [InlineData(2, 4, 2, false)]
    public async Task Tracepoints_capture_independent_values_and_continue_until_exit_or_the_hit_limit(int limit, int exit, int hits, bool finished)
    {
        using var target = new Debuggee("tracepoints");
        await target.Compile("""
            program TracedLoop;
            uses System as Sys;
            begin
              for Index: int := 1 to 3 do
                Sys.Console.WriteLine(Index);
              end for;
              Sys.IO.File.WriteAllText('finished.txt', 'done');
            end;
            end program;
            """);
        var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "debug", target.Assembly, "--cwd", target.DirectoryPath,
            "--tracepoint", "Program.fpas:5", "--print", "Index", "--max-hits", limit.ToString(System.Globalization.CultureInfo.InvariantCulture)]);
        Assert.True(result.ExitCode == exit, result.Output + result.Error);
        using var json = JsonDocument.Parse(result.Output); var observations = json.RootElement.GetProperty("tracepoints").EnumerateArray().ToArray();
        Assert.Equal(hits, observations.Length);
        Assert.Equal(Enumerable.Range(1, hits).Select(value => value.ToString(System.Globalization.CultureInfo.InvariantCulture)),
            observations.Select(item => item.GetProperty("expressions")[0].GetProperty("value").GetProperty("scalar").GetString()));
        Assert.Equal(finished, File.Exists(target.FilePath("finished.txt")));
    }

    [Fact]
    public async Task Diagnose_failure_reports_the_real_unhandled_exception_and_source_frames()
    {
        using var target = new Debuggee("diagnose-failure");
        await target.Compile("""
            program FailureReport;
            uses System as Sys;
            procedure Fail(); begin raise new Sys.InvalidOperationException('CLI_FAILURE_PROBE'); end procedure;
            begin Fail(); end;
            end program;
            """);
        var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "diagnose", "failure", target.Assembly]);
        Assert.True(result.ExitCode == 3, result.Output + result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.Equal("exception", json.RootElement.GetProperty("outcome").GetString());
        Assert.Contains("CLI_FAILURE_PROBE", json.RootElement.GetProperty("exception").ToString(), StringComparison.Ordinal);
        Assert.Contains("Fail", json.RootElement.GetProperty("frames").ToString(), StringComparison.Ordinal);
    }
}
