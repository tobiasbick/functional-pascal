using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Debugging;
using Fpas.IntegrationTests.Projects;
using Fpas.Processes;
using Fpas.Tests.Support;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text.Json;

namespace Fpas.IntegrationTests.Testing;

public sealed class TestLifetimeTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [LinuxFact]
    public Task SIGINT_joins_an_actual_waiting_test_host() => WaitingHost(cancel: true);

    [Fact]
    public Task The_deadline_joins_an_actual_waiting_test_host() => WaitingHost(cancel: false);

    private async Task WaitingHost(bool cancel)
    {
        using var directory = new TemporaryDirectory();
        var project = await TestProjectFixture.Create(directory, sdk, "xunit");
        var ready = directory.File("ready.pid");
        File.WriteAllText(directory.File("Tests/Tests.fpas"), """
            unit Waiting;
            uses Xunit as X, System as S, System.IO.File as File, System.Threading.Thread as Thread;
            [X.FactAttribute]
            public procedure Wait();
            begin
              const Ready: string := S.Environment.GetEnvironmentVariable('FPAS_TEST_READY') ?? 'ready.pid';
              File.WriteAllText(Ready + '.tmp', S.Environment.ProcessId.ToString());
              File.Move(Ready + '.tmp', Ready);
              while true do Thread.Sleep(20); end while;
            end procedure;
            end unit;
            """);
        var built = await ProjectCliFixture.Cli(directory, "build", project, "--report", "json");
        Assert.True(built.ExitCode == 0, built.Output + built.Error);
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        await using var process = OwnedProcess.Start(TestEnvironment.DotnetHost,
            [TestEnvironment.Metadata("FpasCliPath"), "test", project, "--no-build", "--timeout", "15", "--report", "json"],
            directory.Path, new Dictionary<string, string> { ["FPAS_TEST_READY"] = ready });
        process.Input.Dispose();
        using var stdout = new StreamReader(process.Output); using var stderr = new StreamReader(process.Error);
        var output = stdout.ReadToEndAsync(deadline.Token); var error = stderr.ReadToEndAsync(deadline.Token);
        try
        {
            while (!File.Exists(ready))
            {
                if (process.HasExited) Assert.Fail("Test host did not become ready: " + await output + await error);
                await Task.Delay(20, deadline.Token);
            }
            var child = int.Parse(await File.ReadAllTextAsync(ready, deadline.Token), System.Globalization.CultureInfo.InvariantCulture);
            if (cancel) Assert.Equal(0, Kill(process.Id, 2));
            await process.WaitForExit(deadline.Token);
            Assert.Equal(cancel ? 130 : 1, process.ExitCode); Assert.Empty(await error);
            using var report = JsonDocument.Parse(await output);
            Assert.Equal(cancel ? "cancelled" : "timeout", report.RootElement.GetProperty("status").GetString());
            Assert.Throws<ArgumentException>(() => Process.GetProcessById(child));
        }
        finally { await process.TerminateAsync(); }
    }

    [DllImport("libc", EntryPoint = "kill", SetLastError = true)] private static extern int Kill(int pid, int signal);
}
