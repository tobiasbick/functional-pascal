using Fpas.IntegrationTests.Build;
using System.Diagnostics;
using System.Runtime.InteropServices;
using static Fpas.IntegrationTests.LanguageServer.LspFeatureTests;

namespace Fpas.IntegrationTests.LanguageServer;

public sealed class LspRequestLifecycleTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Theory]
    [InlineData("cancel")]
    [InlineData("change")]
    [InlineData("shutdown")]
    public async Task Accepted_requests_cancel_or_reject_superseded_content_and_shutdown_joins_evaluation(string operation)
    {
        if (!OperatingSystem.IsLinux()) return;
        using var directory = sdk.CreateProject(); var configuration = directory.File("NuGet.Config"); File.Delete(configuration); Assert.Equal(0, MakeFifo(configuration, 384));
        await using var client = new LspProbe(); await client.Initialize(directory.File("Game/Game.fpasproj")); await client.Notify("initialized", new { });
        var path = directory.File("Game/Bridge.fpas"); await client.Open(path, "unit Changed; end unit;");
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(1)); using var child = await Evaluation(client.Id, directory.File("Game"), deadline.Token);
        var pending = client.Request("workspace/symbol", new { query = "" }, stringId: "in-flight");
        await client.Request("unknown");
        if (operation == "cancel") await client.Notify("$/cancelRequest", new { id = "in-flight" });
        else if (operation == "change") await client.FullChange(path, "unit ChangedAgain; end unit;", 2);
        if (operation == "shutdown") await client.Stop();
        Error(await pending, operation == "change" ? -32801 : -32800);
        if (operation != "shutdown") await client.Stop();
        Assert.True(child.HasExited);
    }

    private static async Task<Process> Evaluation(int parent, string cwd, CancellationToken token)
    {
        while (true)
        {
            foreach (var path in Directory.EnumerateDirectories("/proc"))
            {
                if (!int.TryParse(Path.GetFileName(path), out var pid)) continue;
                try
                {
                    if (!File.ReadAllLines(Path.Combine(path, "status")).Any(line => line.StartsWith("PPid:", StringComparison.Ordinal) && int.Parse(line[5..].Trim(), System.Globalization.CultureInfo.InvariantCulture) == parent)) continue;
                    if (Directory.ResolveLinkTarget(Path.Combine(path, "cwd"), true)?.FullName == cwd) return Process.GetProcessById(pid);
                }
                catch (Exception error) when (error is IOException or UnauthorizedAccessException or ArgumentException) { }
            }
            await Task.Delay(10, token);
        }
    }
    [DllImport("libc", EntryPoint = "mkfifo", SetLastError = true)]
    private static extern int MakeFifo(string path, uint mode);
}
