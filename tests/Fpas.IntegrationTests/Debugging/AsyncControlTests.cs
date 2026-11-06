using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;
using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class AsyncControlTests
{
    [Theory]
    [InlineData("over")]
    [InlineData("into")]
    public async Task A_real_await_step_follows_its_instance_onto_a_different_clr_thread(string kind)
    {
        using var program = new Debuggee("own-async-await");
        await program.Compile("""
            async program AsyncControl;
            uses System as Sys, System.Console as Console, System.Threading.Tasks as Tasks,
              System.IO.File as File, System.Threading.Thread as Thread;
            async function Work(Input: int; Gate: Tasks.Task<int>): Tasks.Task<int>;
            begin
              var Value: int := Input + 1;
              const Release: int := await Gate;
              Value := Value + Release;
              return Value;
            end function;
            begin
              const Gate: Tasks.TaskCompletionSource<int> := new Tasks.TaskCompletionSource<int>(Tasks.TaskCreationOptions.RunContinuationsAsynchronously);
              const Pending: Tasks.Task<int> := Work(19, Gate.Task);
              if Pending.IsCompleted then raise new Sys.InvalidOperationException('The task must suspend'); end if;
              const Other: Tasks.Task<int> := Work(100, Tasks.Task.FromResult<int>(1));
              Console.WriteLine(await Other);
              Console.WriteLine('pending');
              const Releaser: Tasks.Task<int> := Tasks.Task.Run<int>(function(): int
              begin
                while not File.Exists('release.txt') do Thread.Sleep(10); end while;
                Gate.SetResult(22);
                return 0;
              end function);
              Console.WriteLine(await Pending);
              discard await Releaser;
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, 7, deadline.Token);
        var before = host.Snapshot;
        var original = await Top(host, before, deadline.Token);
        Assert.EndsWith(".Work", original.Frame.Name);
        Assert.NotNull(original.Frame.Async);
        Assert.Equal("executing", original.Frame.Async.State);
        Assert.Contains(original.Values, item => item.Name == "Input" && item.Scalar == "19");
        Assert.Contains(original.Values, item => item.Name == "Value" && item.Scalar == "20");
        await host.SetBreakpoints(new(program.Source, [], before.StopId), deadline.Token);
        await host.Resume(before.StopId!.Value, kind, deadline.Token);
        while (!string.Concat(host.Output().Items.Select(item => item.Text)).Contains("pending", StringComparison.Ordinal))
        {
            Assert.Equal("running", host.Snapshot.State);
            await Task.Delay(10, deadline.Token);
        }
        Assert.Equal("running", (await host.Status(deadline.Token)).State);
        var waiting = host.Snapshot;
        while (waiting.AsyncStep?.Phase != "yielded") waiting = await host.WaitForChange(waiting.Revision, deadline.Token);
        Assert.Equal(original.Frame.Async.Id, waiting.AsyncStep!.Id);
        Assert.Contains("102", string.Concat(host.Output().Items.Select(item => item.Text)));
        var stale = await Assert.ThrowsAsync<DebugException>(() => host.Stack(new(before.StopId.Value, original.Frame.ThreadId), deadline.Token));
        Assert.Equal("stale_reference", stale.Code);
        File.WriteAllText(program.FilePath("release.txt"), "go");
        var resumed = await host.WaitForStop(deadline.Token);
        Assert.Equal("step", resumed.Reason);
        Assert.Null(resumed.AsyncStep);
        var after = await Top(host, resumed, deadline.Token);
        Assert.NotEqual(original.Frame.ThreadId, after.Frame.ThreadId);
        Assert.Equal(original.Frame.Async.Id, after.Frame.Async!.Id);
        Assert.Equal(8, after.Frame.Source!.Line);
        Assert.Contains(after.Values, item => item.Name == "Input" && item.Scalar == "19");
        Assert.Contains(after.Values, item => item.Name == "Value" && item.Scalar == "20");
        Assert.Contains(after.Values, item => item.Name == "Release" && item.Scalar == "22");
        await host.Resume(resumed.StopId!.Value, "over", deadline.Token);
        var returned = await host.WaitForStop(deadline.Token);
        Assert.Equal("step", returned.Reason);
        var final = await Top(host, returned, deadline.Token);
        Assert.Equal(9, final.Frame.Source!.Line);
        Assert.Contains(final.Values, item => item.Name == "Value" && item.Scalar == "42");
        await host.Resume(returned.StopId!.Value, "continue", deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        await host.Stop(deadline.Token);
        Assert.Contains("42", string.Concat(host.Output().Items.Select(item => item.Text)));
    }

    [Theory]
    [InlineData("over", "Tasks.Task.FromResult<int>(22)")]
    [InlineData("into", "Tasks.Task.FromResult<int>(22)")]
    [InlineData("over", "Tasks.Task.FromResult<int>(22).ConfigureAwait(false)")]
    [InlineData("into", "Tasks.Task.FromResult<int>(22).ConfigureAwait(false)")]
    [InlineData("over", "new Tasks.ValueTask<int>(22)")]
    [InlineData("into", "new Tasks.ValueTask<int>(22)")]
    [InlineData("over", "new Tasks.ValueTask<int>(22).ConfigureAwait(false)")]
    [InlineData("into", "new Tasks.ValueTask<int>(22).ConfigureAwait(false)")]
    public async Task An_already_completed_await_stops_at_the_following_source_statement(string kind, string awaited)
    {
        using var program = new Debuggee("own-completed-await");
        await program.Compile("""
            async program CompletedAwait;
            uses System.Console as Console, System.Threading.Tasks as Tasks;
            async function Work(Input: int): Tasks.Task<int>;
            begin
              var Value: int := Input + 1;
              const Release: int := await Tasks.Task.FromResult<int>(22);
              Value := Value + Release;
              return Value;
            end function;
            begin
              Console.WriteLine(await Work(19));
            end;
            end program;
            """.Replace("await Tasks.Task.FromResult<int>(22)", $"await {awaited}", StringComparison.Ordinal));
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, 6, deadline.Token);
        var before = host.Snapshot; var original = await Top(host, before, deadline.Token);
        await host.SetBreakpoints(new(program.Source, [], before.StopId), deadline.Token);
        await host.Resume(before.StopId!.Value, kind, deadline.Token);
        var stop = await host.WaitForStop(deadline.Token);
        Assert.Equal("step", stop.Reason);
        var after = await Top(host, stop, deadline.Token);
        Assert.Equal(original.Frame.Async!.Id, after.Frame.Async!.Id);
        Assert.Equal(7, after.Frame.Source!.Line);
        Assert.Contains(after.Values, item => item.Name == "Release" && item.Scalar == "22");
        Assert.Contains(after.Values, item => item.Name == "Value" && item.Scalar == "20");
        await host.Resume(stop.StopId!.Value, "continue", deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        await host.Stop(deadline.Token);
        Assert.Contains("42", string.Concat(host.Output().Items.Select(item => item.Text)));
    }

    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task A_waiting_async_step_can_be_stopped_or_paused_without_retaining_owned_processes(bool pause)
    {
        using var program = new Debuggee("own-cancel-await");
        await program.Compile("""
            program CancelAwait;
            uses System as Sys, System.Console as Console, System.Threading.Tasks as Tasks, System.Threading.Thread as Thread;
            async function Work(Gate: Tasks.Task<int>): Tasks.Task<int>;
            begin
              const Value: int := await Gate;
              return Value;
            end function;
            begin
              const Gate: Tasks.TaskCompletionSource<int> := new Tasks.TaskCompletionSource<int>();
              const Pending: Tasks.Task<int> := Work(Gate.Task);
              if Pending.IsCompleted then raise new Sys.InvalidOperationException('Must suspend'); end if;
              Console.WriteLine('pending');
              Thread.Sleep(60000);
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, 5, deadline.Token);
        using var worker = System.Diagnostics.Process.GetProcessById(host.WorkerProcessId); _ = worker.SafeHandle;
        using var target = System.Diagnostics.Process.GetProcessById(host.Snapshot.ProcessId!.Value); _ = target.SafeHandle;
        await host.SetBreakpoints(new(program.Source, [], host.Snapshot.StopId), deadline.Token);
        await host.Resume(host.Snapshot.StopId!.Value, "over", deadline.Token);
        var waiting = host.Snapshot;
        while (waiting.AsyncStep?.Phase != "yielded") waiting = await host.WaitForChange(waiting.Revision, deadline.Token);
        Assert.Equal("running", waiting.State);
        if (pause)
        {
            await host.Pause(deadline.Token);
            var paused = await host.WaitForStop(deadline.Token);
            Assert.Equal("pause", paused.Reason); Assert.Null(paused.AsyncStep);
        }
        await host.Stop(deadline.Token);
        Assert.Null(host.Snapshot.AsyncStep);
        if (OperatingSystem.IsLinux())
        {
            Assert.False(Directory.Exists($"/proc/{worker.Id}")); Assert.False(Directory.Exists($"/proc/{target.Id}"));
        }
        else { Assert.True(worker.HasExited); Assert.True(target.HasExited); }
    }

    private static async Task<DebugSessionHost> Launch(Debuggee program, int line, CancellationToken token)
    {
        var launcher = WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost());
        var host = await DebugSessionHost.Start(launcher, program.DirectoryPath, token);
        try
        {
            await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath), token);
            await host.Initialize(token);
            Assert.Equal("entry", (await host.WaitForStop(token)).Reason);
            await host.SetBreakpoints(new(program.Source, [new(line)], host.Snapshot.StopId), token);
            await host.Resume(host.Snapshot.StopId!.Value, "continue", token);
            var stop = await host.WaitForStop(token);
            Assert.True(stop.Reason == "breakpoint", $"Expected a breakpoint stop, got {stop.State}/{stop.Reason}: {stop.Error}");
            return host;
        }
        catch { await host.DisposeAsync(); throw; }
    }

    private static async Task<(DebugFrame Frame, DebugBinding[] Values)> Top(DebugSessionHost host, DebugSnapshot stop, CancellationToken token)
    {
        var thread = Assert.Single(await host.Threads(stop.StopId!.Value, token), item => item.IsStopThread);
        var frame = (await host.Stack(new(stop.StopId.Value, thread.Id), token)).Items[0];
        return (frame, (await host.FrameBindings(new(stop.StopId.Value, frame.Id), token)).Items);
    }
}
