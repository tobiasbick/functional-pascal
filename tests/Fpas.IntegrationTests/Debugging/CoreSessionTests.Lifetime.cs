using System.Diagnostics;
using Fpas.Debugging.Hosting;
using Fpas.Processes;
using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class CoreSessionTests
{
    [Fact]
    public async Task Cancelling_startup_joins_worker_and_suspended_target_before_returning()
    {
        using var program = new Debuggee("own-core-cancel");
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Prepared(program, deadline.Token);
        using var worker = Track(host.WorkerProcessId);
        using var target = Track(host.Snapshot.ProcessId!.Value);
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => host.Initialize(new CancellationToken(true)));
        Gone(worker); Gone(target);
        Assert.Equal(0, worker.ExitCode);
        using var image = new FileStream(program.Assembly, FileMode.Open, FileAccess.ReadWrite, FileShare.None);
    }

    [Fact]
    public async Task Worker_crash_triggers_owned_cleanup_and_preserves_an_unrelated_process()
    {
        using var sentinel = new Debuggee("own-core-unrelated");
        await sentinel.Compile("program Unrelated; uses System.Threading.Thread as Thread; begin Thread.Sleep(60000); end; end program;");
        await using var unrelated = OwnedProcess.Start(BuildProcess.DotnetHost(), [sentinel.Assembly], sentinel.DirectoryPath);
        using var program = new Debuggee("own-core-crash");
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Prepared(program, deadline.Token);
        using var worker = Track(host.WorkerProcessId);
        using var target = Track(host.Snapshot.ProcessId!.Value);
        await host.Initialize(deadline.Token);
        Assert.Equal("stopped", (await host.WaitForStop(deadline.Token)).State);
        worker.Kill();
        var state = host.Snapshot;
        while (state.State != "failed") state = await host.WaitForChange(state.Revision, deadline.Token);
        Assert.Equal("worker_exited", state.Error!.Code);
        await host.DisposeAsync();
        Gone(worker); Gone(target);
        Assert.False(unrelated.HasExited);
    }

    [Fact]
    public async Task Pause_remains_available_during_execution_and_stop_is_idempotent()
    {
        using var program = new Debuggee("own-core-pause");
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Prepared(program, deadline.Token);
        using var worker = Track(host.WorkerProcessId);
        using var target = Track(host.Snapshot.ProcessId!.Value);
        await host.Initialize(deadline.Token);
        var initial = await host.WaitForStop(deadline.Token);
        await host.Resume(initial.StopId!.Value, step: false, deadline.Token);
        await host.Pause(deadline.Token);
        var paused = await host.WaitForStop(deadline.Token);
        Assert.Equal("stopped", paused.State);
        Assert.Equal("pause", paused.Reason);
        Assert.NotEqual(initial.StopId, paused.StopId);
        await host.Stop(deadline.Token);
        await host.Stop(deadline.Token);
        Gone(worker); Gone(target);
    }

    [Fact]
    public async Task An_unresponsive_worker_is_reaped_after_the_shutdown_deadline()
    {
        using var program = new Debuggee("own-core-unresponsive");
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Prepared(program, deadline.Token, TimeSpan.FromMilliseconds(400));
        using var worker = Track(host.WorkerProcessId);
        using var target = Track(host.Snapshot.ProcessId!.Value);
        await host.Initialize(deadline.Token);
        Assert.Equal("stopped", (await host.WaitForStop(deadline.Token)).State);
        using var suspended = new WorkerSuspension(worker);
        var elapsed = Stopwatch.StartNew();
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => host.Stop(deadline.Token));
        Assert.True(elapsed.Elapsed < TimeSpan.FromSeconds(2), "The short shutdown budget was replaced by the default grace.");
        Assert.False(deadline.IsCancellationRequested);
        Assert.Equal("worker_timeout", host.Snapshot.Error!.Code);
        Gone(worker); Gone(target);
    }

    [Fact]
    public async Task Resume_and_duplicate_pause_requests_leave_one_fresh_stable_stop()
    {
        using var program = new Debuggee("own-core-resume-pause");
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Prepared(program, deadline.Token);
        await host.Initialize(deadline.Token);
        var stop = await host.WaitForStop(deadline.Token);
        for (var iteration = 0; iteration < 20; iteration++)
        {
            var previous = stop.StopId!.Value;
            var resume = host.Resume(previous, step: false, deadline.Token);
            var state = host.Snapshot;
            while (state.State != "running") state = await host.WaitForChange(state.Revision, deadline.Token);
            await Task.WhenAll(resume, host.Pause(deadline.Token), host.Pause(deadline.Token));
            stop = await host.WaitForStop(deadline.Token);
            Assert.Equal("stopped", stop.State);
            Assert.Equal("pause", stop.Reason);
            Assert.Equal(previous + 1, stop.StopId);
        }
        await host.Stop(deadline.Token);
    }

    [Fact]
    public async Task Stop_joins_initialization_while_the_real_target_cannot_start_its_runtime()
    {
        using var program = new Debuggee("own-core-initialize-stop");
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Prepared(program, deadline.Token);
        using var worker = Track(host.WorkerProcessId);
        using var target = Track(host.Snapshot.ProcessId!.Value);
        using var suspendedTarget = new WorkerSuspension(target);
        var initialize = host.Initialize(deadline.Token);
        var state = host.Snapshot;
        while (state.Reason != "runtime_startup") state = await host.WaitForChange(state.Revision, deadline.Token);
        Assert.False(initialize.IsCompleted);
        await host.Stop(deadline.Token);
        var cancelled = await Assert.ThrowsAsync<Fpas.Debugging.Core.DebugException>(async () => await initialize);
        Assert.Equal("cancelled", cancelled.Code);
        Gone(worker); Gone(target);
    }

    private static async Task<DebugSessionHost> Prepared(Debuggee program, CancellationToken token, TimeSpan? shutdownGrace = null)
    {
        await program.Compile("""
            program Held;
            uses System.Console as Console, System.Threading.Thread as Thread;
            begin
              var Value: int := 20;
              Console.WriteLine(Value);
              Thread.Sleep(60000);
            end;
            end program;
            """);
        var launcher = WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost());
        var host = await DebugSessionHost.Start(launcher, program.DirectoryPath, token, shutdownGrace: shutdownGrace);
        try
        {
            await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath, new(program.Source, 5)), token);
            return host;
        }
        catch { await host.DisposeAsync(); throw; }
    }

    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task A_nonzero_target_exit_is_preserved_in_the_final_state(bool abrupt)
    {
        using var program = new Debuggee("own-core-exit-code");
        await program.Compile(abrupt ? """
            program ExitStatus;
            uses System.Diagnostics.Process as Process;
            begin
              using const Current: Process := Process.GetCurrentProcess();
              Current.Kill();
            end;
            end program;
            """ : """
            program ExitStatus;
            uses System.Environment as Env;
            begin
              Env.Exit(17);
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        var launcher = WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost());
        await using var host = await DebugSessionHost.Start(launcher, program.DirectoryPath, deadline.Token);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath, new(program.Source, 4)), deadline.Token);
        await host.Initialize(deadline.Token);
        var stop = await host.WaitForStop(deadline.Token);
        await host.Resume(stop.StopId!.Value, step: false, deadline.Token);
        var ended = await host.WaitForStop(deadline.Token);
        Assert.Equal("exited", ended.State);
        Assert.Equal(abrupt ? OperatingSystem.IsLinux() ? 137 : -1 : 17, ended.ExitCode);
        Assert.Equal(abrupt && OperatingSystem.IsLinux() ? 9 : (int?)null, ended.ExitSignal);
        await host.Stop(deadline.Token);
    }

    private static Process Track(int id) { var process = Process.GetProcessById(id); _ = process.SafeHandle; return process; }
    private static void Gone(Process process)
    {
        if (OperatingSystem.IsLinux()) Assert.False(Directory.Exists($"/proc/{process.Id}"));
        else Assert.True(process.HasExited);
    }
}
