using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Build.Support;
using Fpas.IntegrationTests.Processes.Support;
using Fpas.Tests.Support;
using System.Diagnostics;
using System.Text.Json;

namespace Fpas.IntegrationTests.Projects;

public sealed class ProjectSignalTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Console_interrupt_cancels_build_and_run_preparation_with_one_JSON_result_and_joined_children()
    {
        foreach (var command in new[] { "build", "run" })
        {
            using var directory = new TemporaryDirectory();
            var project = await ProjectCliFixture.Init(directory, sdk);
            using var waiting = await WaitingBuild.Prepare(project);
            await Cancel(directory, [command, project, "--report", "json"], waiting.Ready);
        }
    }

    [Fact]
    public async Task Console_interrupt_cancels_an_actual_running_program_after_joining_the_target()
    {
        using var directory = new TemporaryDirectory();
        var project = await ProjectCliFixture.Init(directory, sdk);
        File.WriteAllText(directory.File("App/main.fpas"), """
            program Waiting;
            uses System as S, System.IO.File as File, System.Threading.Thread as Thread;
            begin
              File.WriteAllText(args[0] + '.tmp', S.Environment.ProcessId.ToString());
              File.Move(args[0] + '.tmp', args[0]);
              while true do Thread.Sleep(20); end while;
            end;
            end program;
            """);
        var built = await ProjectCliFixture.Cli(directory, "build", project, "--report", "json"); Assert.Equal(0, built.ExitCode);
        var ready = directory.File("program.pid");
        await Cancel(directory, ["run", project, "--no-build", "--report", "json", "--", ready], ready);
    }

    private static async Task Cancel(TemporaryDirectory directory, string[] arguments, string ready)
    {
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        await using var process = InterruptibleProcess.Start(TestEnvironment.DotnetHost, [TestEnvironment.Metadata("FpasCliPath"), .. arguments], directory.Path);
        process.Input.Dispose();
        using var stdout = new StreamReader(process.Output); using var stderr = new StreamReader(process.Error);
        var output = stdout.ReadToEndAsync(deadline.Token); var error = stderr.ReadToEndAsync(deadline.Token);
        int? child = null;
        try
        {
            while (!File.Exists(ready))
            {
                if (process.HasExited) Assert.Fail("CLI exited before its child was ready: " + await output + await error);
                await Task.Delay(20, deadline.Token);
            }
            child = int.Parse(await File.ReadAllTextAsync(ready, deadline.Token), System.Globalization.CultureInfo.InvariantCulture);
            await process.Interrupt();
            await process.WaitForExit(deadline.Token);
            Assert.Equal(130, process.ExitCode); Assert.Equal("", await error);
            using var json = JsonDocument.Parse(await output);
            Assert.Equal("cancelled", json.RootElement.GetProperty("status").GetString());
            Assert.Throws<ArgumentException>(() => Process.GetProcessById(child.Value));
        }
        finally
        {
            await process.TerminateAsync();
        }
    }
}
