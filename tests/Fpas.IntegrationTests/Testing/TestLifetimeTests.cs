// [Documentation](../../coverage.md#native-test-frameworks)
using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Processes.Support;
using Fpas.IntegrationTests.Projects;
using Fpas.Tests.Support;
using System.Diagnostics;
using System.Text.Json;

namespace Fpas.IntegrationTests.Testing;

public sealed class TestLifetimeTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public Task Console_interrupt_joins_an_actual_waiting_test_host() => WaitingHost(cancel: true);

    [Fact]
    public Task The_deadline_joins_an_actual_waiting_test_host() => WaitingHost(cancel: false);

    private async Task WaitingHost(bool cancel)
    {
        using var directory = new TemporaryDirectory();
        var ready = directory.File("ready.pid");
        var project = await TestProjectFixture.CreateWaiting(directory, sdk, ready);
        var built = await ProjectCliFixture.Cli(directory, "build", project, "--report", "json");
        Assert.True(built.ExitCode == 0, built.Output + built.Error);
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        await using var process = InterruptibleProcess.Start(TestEnvironment.DotnetHost,
            [TestEnvironment.Metadata("FpasCliPath"), "test", project, "--no-build", "--timeout", "15", "--report", "json"],
            directory.Path);
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
            if (cancel) await process.Interrupt();
            await process.WaitForExit(deadline.Token);
            Assert.Equal(cancel ? 130 : 1, process.ExitCode); Assert.Empty(await error);
            using var report = JsonDocument.Parse(await output);
            Assert.Equal(cancel ? "cancelled" : "timeout", report.RootElement.GetProperty("status").GetString());
            Assert.Throws<ArgumentException>(() => Process.GetProcessById(child));
        }
        finally { await process.TerminateAsync(); }
    }
}
