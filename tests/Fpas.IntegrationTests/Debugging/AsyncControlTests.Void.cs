namespace Fpas.IntegrationTests.Debugging;

public sealed partial class AsyncControlTests
{
    [Theory]
    [InlineData("Gate")]
    [InlineData("Gate.ConfigureAwait(false)")]
    [InlineData("new Tasks.ValueTask(Gate)")]
    [InlineData("new Tasks.ValueTask(Gate).ConfigureAwait(false)")]
    public async Task A_non_generic_task_await_keeps_the_source_context_after_real_suspension(string awaited)
    {
        using var program = new Debuggee("own-async-void");
        await program.Compile("""
            program VoidAwait;
            uses System as Sys, System.Console as Console, System.Threading.Tasks as Tasks,
              System.IO.File as File, System.Threading.Thread as Thread;
            async function Work(Gate: Tasks.Task; Value: int): Tasks.Task;
            begin
              await Gate;
              Console.WriteLine(Value);
            end function;
            begin
              const Gate: Tasks.TaskCompletionSource<int> := new Tasks.TaskCompletionSource<int>(Tasks.TaskCreationOptions.RunContinuationsAsynchronously);
              const Pending: Tasks.Task := Work(Gate.Task, 42);
              if Pending.IsCompleted then raise new Sys.InvalidOperationException('Must suspend'); end if;
              Console.WriteLine('pending');
              const Releaser: Tasks.Task<int> := Tasks.Task.Run<int>(function(): int
              begin
                while not File.Exists('release.txt') do Thread.Sleep(10); end while;
                Gate.SetResult(1);
                return 0;
              end function);
              Pending.GetAwaiter().GetResult();
              discard Releaser.GetAwaiter().GetResult();
            end;
            end program;
            """.Replace("await Gate;", $"await {awaited};", StringComparison.Ordinal));
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, 6, deadline.Token);
        var before = host.Snapshot; var original = await Top(host, before, deadline.Token);
        await host.SetBreakpoints(new(program.Source, [], before.StopId), deadline.Token);
        await host.Resume(before.StopId!.Value, "over", deadline.Token);
        while (!string.Concat(host.Output().Items.Select(item => item.Text)).Contains("pending", StringComparison.Ordinal))
            await Task.Delay(10, deadline.Token);
        File.WriteAllText(program.FilePath("release.txt"), "go");
        var stop = await host.WaitForStop(deadline.Token);
        var resumed = await Top(host, stop, deadline.Token);
        Assert.Equal("step", stop.Reason); Assert.Equal(7, resumed.Frame.Source!.Line);
        Assert.Equal(original.Frame.Async!.Id, resumed.Frame.Async!.Id);
        Assert.Contains(resumed.Values, item => item.Name == "Value" && item.Scalar == "42");
        await host.Resume(stop.StopId!.Value, "continue", deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        await host.Stop(deadline.Token);
        Assert.Contains("42", string.Concat(host.Output().Items.Select(item => item.Text)));
    }
}
