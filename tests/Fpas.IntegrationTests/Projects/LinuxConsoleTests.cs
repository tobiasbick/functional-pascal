// [Documentation](../../coverage.md#native-linux-cli-acceptance)
using System.Globalization;
using System.Text.Json;
using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Build.Support;
using Fpas.IntegrationTests.Processes.Support;
using Fpas.IntegrationTests.Testing;
using Fpas.Processes.Terminal;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Projects;

/// <summary>Verifies CLI cancellation through the Linux terminal's interrupt character.</summary>
public sealed class LinuxConsoleTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [LinuxConsoleFact]
    public async Task Ctrl_C_cancels_build_and_check_and_reaps_the_build_child()
    {
        foreach (string command in new[] { "build", "check" })
        {
            using var directory = new TemporaryDirectory();
            var project = await ProjectCliFixture.Init(directory, sdk);
            var restored = await ProjectCliFixture.Cli(directory, "restore", project);
            Assert.True(restored.ExitCode == 0, restored.Output + restored.Error);
            using var waiting = await WaitingBuild.Prepare(project);
            string[] report = ["--report", "json"];
            await Cancel(directory, waiting.Ready, command, project, report);
        }
    }

    [LinuxConsoleFact]
    public async Task Ctrl_C_cancels_run_and_reaps_the_program_with_and_without_failure_capture()
    {
        using var directory = new TemporaryDirectory();
        var project = await ProjectCliFixture.Init(directory, sdk);
        File.WriteAllText(directory.File("App/main.fpas"), """
            program Waiting;
            uses System as S; uses System.IO.File as File; uses System.Threading.Thread as Thread;
            begin
              File.WriteAllText(args[0] + '.tmp', S.Environment.ProcessId.ToString());
              File.Move(args[0] + '.tmp', args[0]);
              while true do Thread.Sleep(20); end while;
            end;
            end program;
            """);
        var built = await ProjectCliFixture.Cli(directory, "build", project);
        Assert.True(built.ExitCode == 0, built.Output + built.Error);
        var ready = directory.File("program.pid");
        var failure = directory.File("failure.json");
        foreach (bool capture in new[] { false, true })
        {
            File.Delete(ready);
            string[] failureOptions = capture ? ["--failure-report", failure] : [];
            await Cancel(directory, ready, "run", project,
                ["--no-build", "--report", "json", .. failureOptions, "--", ready]);
            Assert.False(File.Exists(failure));
        }
    }

    [LinuxConsoleFact]
    public async Task Ctrl_C_cancels_test_and_reaps_the_test_host()
    {
        using var directory = new TemporaryDirectory();
        var ready = directory.File("test.pid");
        var project = await TestProjectFixture.CreateWaiting(directory, sdk, ready);
        var built = await ProjectCliFixture.Cli(directory, "build", project);
        Assert.True(built.ExitCode == 0, built.Output + built.Error);
        await Cancel(directory, ready, "test", project, ["--no-build", "--report", "json"]);
    }

    private static async Task Cancel(TemporaryDirectory directory, string ready, string command, string project, string[] options)
    {
        var stdout = directory.File("stdout.json");
        var stderr = directory.File("stderr.txt");
        const string launch = """
            out=$1
            err=$2
            shift 2
            test -t 0 || exit 125
            exec "$@" >"$out" 2>"$err"
            """;
        // Preserve the controlling terminal on stdin and keep JSON and stderr separate from terminal echo.
        await using var terminal = PseudoTerminal.Start("/bin/sh",
            ["-c", launch, "fpas-console-test", stdout, stderr, TestEnvironment.DotnetHost,
                TestEnvironment.Metadata("FpasCliPath"), command, project, .. options], directory.Path);
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(1));
        while (!File.Exists(ready))
        {
            if (terminal.Completion.IsCompleted)
                Assert.Fail($"{command} exited before its child was ready.\n" +
                    (File.Exists(stdout) ? File.ReadAllText(stdout) : "") + (File.Exists(stderr) ? File.ReadAllText(stderr) : ""));
            await Task.Delay(20, deadline.Token);
        }

        using var child = ProcessExit.Track(int.Parse(File.ReadAllText(ready), CultureInfo.InvariantCulture));
        try
        {
            await terminal.Input.WriteAsync(new byte[] { 3 }, deadline.Token);
            await terminal.Input.FlushAsync(deadline.Token);
            int exit = await terminal.Completion.WaitAsync(deadline.Token);
            var output = File.ReadAllText(stdout);
            Assert.True(exit == 130, $"{command} exited with {exit}: {output}\n{File.ReadAllText(stderr)}");
            Assert.Empty(File.ReadAllText(stderr));
            using var json = JsonDocument.Parse(output);
            Assert.Equal("cancelled", json.RootElement.GetProperty("status").GetString());
            ProcessExit.AssertGone(child);
        }
        finally
        {
            if (!child.HasExited) child.Kill(entireProcessTree: true);
        }
    }
}

internal sealed class LinuxConsoleFactAttribute : FactAttribute
{
    public LinuxConsoleFactAttribute()
    {
        if (!OperatingSystem.IsLinux()) Skip = "Linux terminal interrupt acceptance.";
    }
}
