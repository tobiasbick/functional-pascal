using System.Diagnostics;
using System.Text.Json.Nodes;
using Fpas.IntegrationTests.Processes.Support;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class DapDebuggerTests
{
    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task Client_eof_and_console_interrupt_reap_target_and_worker_before_harness_cleanup(bool interrupt)
    {
        using var program = new Debuggee("dap-client-end");
        await WaitingProgram(program);
        await using var client = new DapProbe(interruptible: interrupt);
        await LaunchRunning(client, program);
        var pid = await Ready(program); using var target = Track(pid);
        var parent = ProcessIdentity.Parent(pid); using var worker = Track(OperatingSystem.IsLinux() ? ProcessIdentity.Parent(parent) : parent);
        if (interrupt) { await client.Interrupt(); await client.End(130); }
        else await client.EndInput();
        Gone(target); Gone(worker);
    }

    [Fact]
    public async Task A_full_ordinary_queue_still_accepts_terminate_and_completes_cancelled_request_replies()
    {
        using var program = new Debuggee("dap-control-reserve");
        await WaitingProgram(program);
        await using var client = new DapProbe(); await LaunchRunning(client, program);
        var pid = await Ready(program); using var target = Track(pid);
        var parent = ProcessIdentity.Parent(pid); using var worker = Track(OperatingSystem.IsLinux() ? ProcessIdentity.Parent(parent) : parent);
        await client.Success("pause"); await client.Event("stopped");
        using var suspended = new WorkerSuspension(worker);
        var requests = Enumerable.Range(0, 64).Select(_ => client.Request("threads")).ToArray();
        await Task.Delay(100);
        await client.Success("terminate");
        foreach (var request in requests) Assert.False((await request).GetProperty("success").GetBoolean());
        await client.End(); Gone(target); Gone(worker);
    }

    private static async Task WaitingProgram(Debuggee program) => await program.Compile("""
        program Waiting;
        uses System.Environment as Environment, System.IO.File as File, System.Threading.Thread as Thread;
        begin
          File.WriteAllText('target.pid.tmp', Environment.ProcessId.ToString());
          File.Move('target.pid.tmp', 'target.pid');
          while true do Thread.Sleep(50); end while;
        end;
        end program;
        """);

    private static async Task LaunchRunning(DapProbe client, Debuggee program)
    {
        await client.Success("initialize"); await client.Success("launch", new() { ["program"] = program.Assembly, ["cwd"] = program.DirectoryPath });
        await client.Event("initialized"); await client.Success("configurationDone");
    }
    private static async Task<int> Ready(Debuggee program)
    {
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(20));
        while (!File.Exists(program.FilePath("target.pid"))) await Task.Delay(10, deadline.Token);
        return int.Parse(File.ReadAllText(program.FilePath("target.pid")), System.Globalization.CultureInfo.InvariantCulture);
    }
    private static Process Track(int pid) { var result = Process.GetProcessById(pid); _ = result.SafeHandle; return result; }
    private static void Gone(Process process)
    {
        if (OperatingSystem.IsLinux()) Assert.False(Directory.Exists($"/proc/{process.Id}")); else Assert.True(process.HasExited);
    }
}
