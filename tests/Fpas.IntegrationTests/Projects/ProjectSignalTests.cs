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
    public async Task Run_streams_build_messages_to_stderr_while_the_build_is_still_running()
    {
        using var directory = new TemporaryDirectory();
        var project = await ProjectCliFixture.Init(directory, sdk);
        using var waiting = await WaitingBuild.Prepare(project);
        await Cancel(directory, ["run", project], waiting.Ready, WaitingBuild.ProgressMessage);
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

    private static async Task Cancel(TemporaryDirectory directory, string[] arguments, string ready, string? liveMessage = null)
    {
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        await using var process = InterruptibleProcess.Start(TestEnvironment.DotnetHost, [TestEnvironment.Metadata("FpasCliPath"), .. arguments], directory.Path);
        process.Input.Dispose();
        using var stdout = new StreamReader(process.Output); using var stderr = new StreamReader(process.Error);
        var output = stdout.ReadToEndAsync(deadline.Token);
        var error = liveMessage is null ? stderr.ReadToEndAsync(deadline.Token) : null;
        int? child = null;
        try
        {
            while (!File.Exists(ready))
            {
                if (process.HasExited) Assert.Fail("CLI exited before its child was ready: " + await output + await (error ?? stderr.ReadToEndAsync(deadline.Token)));
                await Task.Delay(20, deadline.Token);
            }
            child = int.Parse(await File.ReadAllTextAsync(ready, deadline.Token), System.Globalization.CultureInfo.InvariantCulture);
            if (liveMessage is not null)
            {
                using var progressDeadline = new CancellationTokenSource(TimeSpan.FromSeconds(5));
                try
                {
                    string? line;
                    do { line = await stderr.ReadLineAsync(progressDeadline.Token); }
                    while (line is not null && !line.Contains(liveMessage, StringComparison.Ordinal));
                    Assert.NotNull(line);
                }
                catch (OperationCanceledException) when (progressDeadline.IsCancellationRequested)
                { Assert.Fail("The waiting build's message did not reach stderr while the build was active."); }
                error = stderr.ReadToEndAsync(deadline.Token);
            }
            await process.Interrupt();
            await process.WaitForExit(deadline.Token);
            var report = await output;
            Assert.True(process.ExitCode == 130, $"{arguments[0]} exited with {process.ExitCode}: {report}");
            if (liveMessage is null)
            {
                Assert.Equal("", await error!);
                using var json = JsonDocument.Parse(report);
                Assert.Equal("cancelled", json.RootElement.GetProperty("status").GetString());
            }
            else
            {
                Assert.Empty(report);
                Assert.DoesNotContain(liveMessage, await error!);
            }
            Assert.Throws<ArgumentException>(() => Process.GetProcessById(child.Value));
        }
        finally
        {
            await process.TerminateAsync();
        }
    }
}
