// [Documentation](../../../docs/architecture/debugger-dap.md)
using System.Text.Json.Nodes;
using Fpas.IntegrationTests.Processes.Support;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class DapDebuggerTests
{
    /// <summary>Verifies that a rejected shutdown leaves initialization and later shutdown available.</summary>
    /// <param name="command">The DAP shutdown command sent before initialization.</param>
    [Theory]
    [InlineData("disconnect")]
    [InlineData("terminate")]
    public async Task Rejected_shutdown_before_initialization_leaves_the_adapter_available(string command)
    {
        await using var client = new DapProbe();
        var rejected = await client.Request(command);
        Assert.False(rejected.GetProperty("success").GetBoolean());
        Assert.Equal("Initialize the adapter first.", rejected.GetProperty("body").GetProperty("error").GetProperty("format").GetString());

        await client.Success("initialize");
        await client.Success("disconnect");
        await client.End();
    }

    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task Client_eof_and_console_interrupt_reap_target_and_worker_before_harness_cleanup(bool interrupt)
    {
        using var program = new Debuggee("dap-client-end");
        await WaitingProgram(program);
        await using var client = new DapProbe(interruptible: interrupt);
        await LaunchRunning(client, program);
        var pid = await Ready(program); using var target = ProcessExit.Track(pid);
        var parent = ProcessIdentity.Parent(pid); using var worker = ProcessExit.Track(OperatingSystem.IsLinux() ? ProcessIdentity.Parent(parent) : parent);
        if (interrupt) { await client.Interrupt(); await client.End(130); }
        else await client.EndInput();
        ProcessExit.AssertGone(target); ProcessExit.AssertGone(worker);
    }

    [Theory]
    [InlineData(0)]
    [InlineData(64)]
    public async Task Terminate_joins_an_unresponsive_worker_with_and_without_a_full_ordinary_queue(int requestCount)
    {
        using var program = new Debuggee("dap-control-reserve");
        await WaitingProgram(program);
        await using var client = new DapProbe(); await LaunchRunning(client, program);
        var pid = await Ready(program); using var target = ProcessExit.Track(pid);
        var parent = ProcessIdentity.Parent(pid); using var worker = ProcessExit.Track(OperatingSystem.IsLinux() ? ProcessIdentity.Parent(parent) : parent);
        await client.Success("pause"); await client.Event("stopped");
        using var suspended = new WorkerSuspension(worker);
        var requests = Enumerable.Range(0, requestCount).Select(_ => client.Request("threads")).ToArray();
        await Task.Delay(100);
        await client.Success("terminate");
        foreach (var request in requests)
        {
            var response = await request;
            Assert.False(response.GetProperty("success").GetBoolean(), response.GetRawText());
        }
        await client.End(); ProcessExit.AssertGone(target); ProcessExit.AssertGone(worker);
    }

    private static async Task WaitingProgram(Debuggee program) => await program.Compile("""
        program Waiting;
        uses System.Environment as Environment; uses System.IO.File as File; uses System.Threading.Thread as Thread;
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
}
