using System.Diagnostics;
using System.Text.Json;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class McpDebugTests
{
    [NetCoreDbgFact]
    public async Task Pause_and_stop_remain_available_during_status_wait_and_new_launch_rejects_old_ids()
    {
        using var program = new Debuggee("mcp-pause");
        await program.Compile("""
            program Busy;
            begin
              var Count: long := 0L;
              while Count < 1000000000000L do
                Count := Count + 1L;
              end while;
            end;
            end program;
            """);
        await using var probe = await McpProbe.Create();
        var entry = await probe.Call("debug_launch", new { program = program.Assembly, cwd = program.DirectoryPath });
        var id = entry.GetProperty("sessionId").GetString()!;
        await probe.Call("debug_launch", new { program = program.Assembly, cwd = program.DirectoryPath }, "invalid_state");
        var running = await probe.Continue(id, entry);
        using (var cancelled = new CancellationTokenSource())
        {
            var status = probe.Client.CallToolAsync("debug_status", new Dictionary<string, object?>
            {
                ["sessionId"] = id, ["afterEvent"] = running.GetProperty("event").GetInt64(), ["waitMilliseconds"] = 30000
            }, cancellationToken: cancelled.Token);
            cancelled.CancelAfter(100);
            await Assert.ThrowsAnyAsync<OperationCanceledException>(() => status.AsTask());
        }
        var waiting = probe.Call("debug_status", new { sessionId = id, afterEvent = running.GetProperty("event").GetInt64(), waitMilliseconds = 30000 });
        await probe.Call("debug_pause", new { sessionId = id });
        await waiting;
        var paused = await probe.Stopped(id, entry.GetProperty("stopId").GetInt64());
        var threads = await probe.Call("debug_threads", new { sessionId = id, stopId = paused.GetProperty("stopId").GetInt64() });
        Assert.NotEmpty(threads.GetProperty("data").EnumerateArray());
        running = await probe.Continue(id, paused);
        waiting = probe.Call("debug_status", new { sessionId = id, afterEvent = running.GetProperty("event").GetInt64(), waitMilliseconds = 30000 });
        await probe.Call("debug_stop", new { sessionId = id }); await waiting;
        var another = await probe.Call("debug_launch", new { program = program.Assembly, cwd = program.DirectoryPath });
        Assert.NotEqual(id, another.GetProperty("sessionId").GetString());
        await probe.Call("debug_status", new { sessionId = id }, "invalid_session");
    }

    [NetCoreDbgFact]
    public async Task MCP_EOF_exits_the_server_and_terminates_its_running_target()
    {
        using var program = new Debuggee("mcp-eof");
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
        var probe = await McpProbe.ObservedServer();
        int pid;
        try
        {
            var entry = await probe.Call("debug_launch", new { program = program.Assembly, cwd = program.DirectoryPath, args = new[] { pidFile } });
            await probe.Continue(entry.GetProperty("sessionId").GetString()!, entry);
            while (!File.Exists(pidFile) || new FileInfo(pidFile).Length == 0) await Task.Delay(10, probe.Token);
            pid = int.Parse(await File.ReadAllTextAsync(pidFile, probe.Token), System.Globalization.CultureInfo.InvariantCulture);
        }
        finally { await probe.DisposeAsync(); }
        Assert.Throws<ArgumentException>(() => Process.GetProcessById(pid));
    }

    [NetCoreDbgFact]
    public async Task Missing_or_changed_symbols_and_source_are_actionable_tool_errors()
    {
        using var program = new Debuggee("mcp-inputs");
        await program.Compile("program Probe; begin end; end program;");
        await using var probe = await McpProbe.Create();
        await probe.Call("debug_launch", new { program = "relative.dll", cwd = program.DirectoryPath }, "invalid_argument");
        File.AppendAllText(program.Source, "\n");
        await probe.Call("debug_launch", new { program = program.Assembly, cwd = program.DirectoryPath }, "missing_source");
        File.Delete(Path.ChangeExtension(program.Assembly, ".pdb"));
        await probe.Call("debug_launch", new { program = program.Assembly, cwd = program.DirectoryPath }, "missing_symbols");
    }
}
