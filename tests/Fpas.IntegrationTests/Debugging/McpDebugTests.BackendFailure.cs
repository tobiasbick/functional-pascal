using Fpas.IntegrationTests.Processes.Support;
using System.Diagnostics;
using System.Globalization;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class McpDebugTests
{
    [NetCoreDbgFact]
    public async Task Actual_adapter_crash_cleans_up_the_running_target_without_a_live_parent()
    {
        using var program = new Debuggee("mcp-crash");
        await program.Compile("""
            program Busy;
            uses System as S, System.IO.File as File, System.Threading.Thread as Thread;
            begin
              const Args: string[] := S.Environment.GetCommandLineArgs();
              File.WriteAllText(Args[1], S.Environment.ProcessId.ToString());
              while true do Thread.Sleep(20); end while;
            end;
            end program;
            """);
        var pidFile = program.FilePath("target.pid");
        await using var probe = await McpProbe.ObservedServer();
        var entry = await probe.Call("debug_launch", new { program = program.Assembly, cwd = program.DirectoryPath, args = new[] { pidFile } });
        var id = entry.GetProperty("sessionId").GetString()!;
        await probe.Continue(id, entry);
        while (!File.Exists(pidFile) || new FileInfo(pidFile).Length == 0) await Task.Delay(10, probe.Token);
        var pid = int.Parse(await File.ReadAllTextAsync(pidFile, probe.Token), CultureInfo.InvariantCulture);
        using var target = Process.GetProcessById(pid);
        using var adapter = Process.GetProcessById(ProcessIdentity.Parent(pid));
        // Verify actual containment before deliberately killing the owned adapter alone.
        Assert.Equal("netcoredbg", adapter.ProcessName);
        ProcessIdentity.AssertOwned(target, adapter.Id);
        adapter.Kill();
        var failed = await probe.State(id, "failed");
        Assert.Equal("backend_exited", failed.GetProperty("error").GetProperty("code").GetString());
        await probe.Call("debug_stop", new { sessionId = id });
        Assert.True(target.HasExited);
        if (OperatingSystem.IsLinux()) Assert.False(Directory.Exists("/proc/" + pid));
    }
}
