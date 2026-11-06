using Fpas.IntegrationTests.Build;
using Fpas.Workspaces.Live;
using System.Diagnostics;
using System.Runtime.InteropServices;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class LiveCancellationTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Caller_cancellation_joins_the_owned_evaluation_process_before_returning()
    {
        if (!OperatingSystem.IsLinux()) return;
        using var directory = sdk.CreateProject();
        var configuration = directory.File("NuGet.Config"); File.Delete(configuration);
        Assert.Equal(0, MakeFifo(configuration, 384));
        await using var workspace = new LiveWorkspace(new(directory.File("Game/Game.fpasproj")));
        using var cancellation = new CancellationTokenSource(); using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(1));
        var work = workspace.AnalyzeAsync(cancellation.Token); Process? child = null;
        try
        {
            while (child is null)
            {
                if (work.IsCompleted) { await work; Assert.Fail("Evaluation ended before opening its blocked input."); }
                foreach (var path in Directory.EnumerateDirectories("/proc"))
                {
                    if (!int.TryParse(Path.GetFileName(path), out var pid)) continue;
                    try
                    {
                        var status = File.ReadAllLines(Path.Combine(path, "status"));
                        if (!status.Any(line => line.StartsWith("PPid:", StringComparison.Ordinal) && int.Parse(line[5..].Trim(), System.Globalization.CultureInfo.InvariantCulture) == Environment.ProcessId)) continue;
                        var cwd = Directory.ResolveLinkTarget(Path.Combine(path, "cwd"), returnFinalTarget: true)?.FullName;
                        if (cwd == directory.File("Game")) { child = Process.GetProcessById(pid); break; }
                    }
                    catch (Exception error) when (error is IOException or UnauthorizedAccessException or ArgumentException) { }
                }
                if (child is null) await Task.Delay(10, deadline.Token);
            }
            cancellation.Cancel(); await Assert.ThrowsAnyAsync<OperationCanceledException>(() => work);
            Assert.True(child.HasExited); Assert.Null(workspace.Current);
        }
        finally
        {
            cancellation.Cancel(); try { await work; } catch (OperationCanceledException) { } child?.Dispose();
        }
    }

    [DllImport("libc", EntryPoint = "mkfifo", SetLastError = true)]
    private static extern int MakeFifo(string path, uint mode);
}
