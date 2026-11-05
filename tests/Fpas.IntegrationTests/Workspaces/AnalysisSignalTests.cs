using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Build.Support;
using Fpas.IntegrationTests.Debugging;
using Fpas.IntegrationTests.Workspaces.Support;
using Fpas.Processes;
using Fpas.Tests.Support;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text.Json;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class AnalysisSignalTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [LinuxFact]
    public async Task SIGINT_returns_exit_130_and_one_cancelled_JSON_document_after_joining_build_tasks()
    {
        foreach (var arguments in new[]
        {
            new[] { "check", "Game/Game.fpasproj", "--diagnostics", "json" },
            new[] { "query", "refs", "Game/Game.fpasproj", "--file", "Game/Bridge.fpas", "--line", "6", "--column", "25" },
            new[] { "query", "callees", "Game/Game.fpasproj", "--file", "Game/Bridge.fpas", "--line", "6", "--column", "25" },
            new[] { "query", "callers", "Game/Game.fpasproj", "--file", "Game/Bridge.fpas", "--line", "6", "--column", "25" }
        })
        {
            using var project = sdk.CreateProject();
            using var waiting = await WaitingBuild.Prepare(project.File("Game/Game.fpasproj"));
            using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
            var host = Environment.GetEnvironmentVariable("DOTNET_HOST_PATH") ?? Path.Combine(Environment.GetEnvironmentVariable("DOTNET_ROOT")!, "dotnet");
            await using var process = OwnedProcess.Start(host, [TestEnvironment.Metadata("FpasCliPath"), .. arguments], project.Path);
            process.Input.Dispose();
            using var stdout = new StreamReader(process.Output); using var stderr = new StreamReader(process.Error);
            var output = stdout.ReadToEndAsync(deadline.Token); var error = stderr.ReadToEndAsync(deadline.Token);
            int? child = null;
            try
            {
                while (!File.Exists(waiting.Ready))
                {
                    if (process.HasExited) Assert.Fail("CLI exited before the preparation task became ready: " + await output + await error);
                    await Task.Delay(20, deadline.Token);
                }
                child = int.Parse(await File.ReadAllTextAsync(waiting.Ready, deadline.Token), System.Globalization.CultureInfo.InvariantCulture);
                Assert.Equal(0, Kill(process.Id, 2));
                await process.WaitForExit(deadline.Token);
                Assert.Equal(130, process.ExitCode); Assert.Equal("", await error);
                using var json = JsonDocument.Parse(await output);
                Assert.Equal("cancelled", json.RootElement.GetProperty("status").GetString());
                Assert.Equal("cancelled", json.RootElement.GetProperty("error").GetProperty("code").GetString());
                Assert.Throws<ArgumentException>(() => Process.GetProcessById(child.Value));
            }
            finally
            {
                if (!process.HasExited)
                {
                    Kill(process.Id, 2);
                    if (child is { } pid)
                    {
                        try { using var task = Process.GetProcessById(pid); if (!task.HasExited) task.Kill(true); }
                        catch (ArgumentException) { }
                    }
                    using var cleanup = new CancellationTokenSource(TimeSpan.FromSeconds(10));
                    try { await process.WaitForExit(cleanup.Token); } catch (OperationCanceledException) { }
                }
            }
        }
    }

    [DllImport("libc", EntryPoint = "kill", SetLastError = true)] private static extern int Kill(int pid, int signal);
}
