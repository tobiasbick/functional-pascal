using System.Diagnostics;
using System.Globalization;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class McpDebugTests
{
    [LinuxNetCoreDbgFact]
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
        var stat = await File.ReadAllTextAsync("/proc/" + pid + "/stat", probe.Token);
        var fields = stat[(stat.LastIndexOf(')') + 2)..].Split(' ');
        var parent = int.Parse(fields[1], CultureInfo.InvariantCulture);
        var group = int.Parse(fields[2], CultureInfo.InvariantCulture);
        using var adapter = Process.GetProcessById(parent);
        // Verify actual containment before deliberately killing the owned adapter alone.
        Assert.Equal("netcoredbg", adapter.ProcessName);
        Assert.Equal(parent, group);
        adapter.Kill();
        var failed = await probe.State(id, "failed");
        Assert.Equal("backend_exited", failed.GetProperty("error").GetProperty("code").GetString());
        await probe.Call("debug_stop", new { sessionId = id });
        Assert.False(Directory.Exists("/proc/" + pid));
    }
}

internal sealed class LinuxNetCoreDbgFactAttribute : FactAttribute
{
    public LinuxNetCoreDbgFactAttribute()
    {
        if (!OperatingSystem.IsLinux()) Skip = "Linux process-group crash recovery acceptance.";
        else if (string.IsNullOrWhiteSpace(Environment.GetEnvironmentVariable("FPAS_NETCOREDBG_PATH")))
            Skip = "Set FPAS_NETCOREDBG_PATH to the tested NetCoreDbg executable.";
    }
}
