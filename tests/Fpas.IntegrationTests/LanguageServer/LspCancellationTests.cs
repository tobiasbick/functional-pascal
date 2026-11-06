using Fpas.IntegrationTests.Build;
using System.Diagnostics;
using System.Runtime.InteropServices;

namespace Fpas.IntegrationTests.LanguageServer;

public sealed class LspCancellationTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Theory]
    [InlineData("shutdown")]
    [InlineData("eof")]
    [InlineData("change")]
    public async Task EOF_shutdown_or_superseding_edit_joins_blocked_owned_evaluation(string operation)
    {
        if (!OperatingSystem.IsLinux()) return;
        using var directory = sdk.CreateProject(); var configuration = directory.File("NuGet.Config");
        File.Delete(configuration); Assert.Equal(0, MakeFifo(configuration, 384));
        await using var client = new LspProbe(); await client.Initialize(directory.File("Game/Game.fpasproj")); await client.Notify("initialized", new { });
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(1)); Process? child = null;
        try
        {
            while (child is null)
            {
                foreach (var path in Directory.EnumerateDirectories("/proc"))
                {
                    if (!int.TryParse(Path.GetFileName(path), out var pid)) continue;
                    try
                    {
                        var status = File.ReadAllLines(Path.Combine(path, "status"));
                        if (!status.Any(line => line.StartsWith("PPid:", StringComparison.Ordinal) && int.Parse(line[5..].Trim(), System.Globalization.CultureInfo.InvariantCulture) == client.Id)) continue;
                        if (Directory.ResolveLinkTarget(Path.Combine(path, "cwd"), true)?.FullName == directory.File("Game"))
                        { child = Process.GetProcessById(pid); break; }
                    }
                    catch (Exception error) when (error is IOException or UnauthorizedAccessException or ArgumentException) { }
                }
                if (child is null) await Task.Delay(10, deadline.Token);
            }
            if (operation == "change")
            {
                await client.Open(directory.File("Game/Bridge.fpas"), "unit Changed; end unit;");
                // The response is a reader-order barrier after the document mutation.
                await client.Request("unknown");
                while (!child.HasExited) await Task.Delay(10, deadline.Token);
            }
            if (operation == "eof") { client.Eof(); Assert.True(await client.Exit() == 0, await client.Error); } else await client.Stop();
            Assert.True(child.HasExited);
        }
        finally { child?.Dispose(); }
    }

    [DllImport("libc", EntryPoint = "mkfifo", SetLastError = true)]
    private static extern int MakeFifo(string path, uint mode);
}
