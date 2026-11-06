using System.Diagnostics;
using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;
using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class CoreSessionTests
{
    [Fact]
    public async Task A_supervised_session_reads_a_real_stop_rejects_stale_stops_and_reaps_its_worker()
    {
        using var program = new Debuggee("own-core-session");
        await program.Compile("""
            program Supervised;
            uses System.Console as Console;
            function Calculate(Input: int): int;
            begin
              var Value: int := Input + 1;
              Value := Value * 2;
              return Value + 2;
            end function;
            begin
              Console.WriteLine(Calculate(19));
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        var launcher = WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost());
        await using var host = await DebugSessionHost.Start(launcher, program.DirectoryPath, deadline.Token);
        using var worker = Process.GetProcessById(host.WorkerProcessId); _ = worker.SafeHandle;
        var starting = await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath, new(program.Source, 6)), deadline.Token);
        Assert.Equal("starting", starting.State);
        Assert.True(starting.ProcessId > 0);
        Assert.NotEqual(host.WorkerProcessId, starting.ProcessId);
        using var target = Process.GetProcessById(starting.ProcessId!.Value); _ = target.SafeHandle;
        await host.Initialize(deadline.Token);
        var stop = await host.WaitForStop(deadline.Token);
        Assert.Equal("stopped", stop.State);
        Assert.Contains(await host.Bindings(stop.StopId!.Value, deadline.Token), value => value.Name == "Value" && value.Scalar == "20");
        await host.Resume(stop.StopId.Value, step: true, deadline.Token);
        var next = await host.WaitForStop(deadline.Token);
        Assert.Equal("stopped", next.State);
        Assert.NotEqual(stop.StopId, next.StopId);
        Assert.Contains(await host.Bindings(next.StopId!.Value, deadline.Token), value => value.Name == "Value" && value.Scalar == "40");
        var stale = await Assert.ThrowsAsync<DebugException>(() => host.Bindings(stop.StopId.Value, deadline.Token));
        Assert.Equal("stale_reference", stale.Code);
        await host.Resume(next.StopId.Value, step: false, deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        Assert.Equal(0, host.Snapshot.ExitCode);
        await host.Stop(deadline.Token);
        Gone(worker); Gone(target);
        Assert.Equal(0, worker.ExitCode);
        Assert.Contains("42", string.Concat(host.Output().Items.Select(item => item.Text)));
    }
}
