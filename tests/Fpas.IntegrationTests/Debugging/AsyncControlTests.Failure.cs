namespace Fpas.IntegrationTests.Debugging;

public sealed partial class AsyncControlTests
{
    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task A_fault_or_cancellation_while_stepping_an_await_releases_the_owned_step(bool cancelled)
    {
        using var program = new Debuggee("own-async-failure");
        var source = """
            program AsyncFailure;
            uses System as Sys, System.Console as Console, System.Threading.Tasks as Tasks,
              System.IO.File as File, System.Threading.Thread as Thread;
            async function Work(Gate: Tasks.Task<int>): Tasks.Task<int>;
            begin
              const Value: int := await Gate;
              return Value;
            end function;
            begin
              const Gate: Tasks.TaskCompletionSource<int> := new Tasks.TaskCompletionSource<int>(Tasks.TaskCreationOptions.RunContinuationsAsynchronously);
              const Pending: Tasks.Task<int> := Work(Gate.Task);
              if Pending.IsCompleted then raise new Sys.InvalidOperationException('Must suspend'); end if;
              Console.WriteLine('pending');
              const Releaser: Tasks.Task<int> := Tasks.Task.Run<int>(function(): int
              begin
                while not File.Exists('release.txt') do Thread.Sleep(10); end while;
                Gate.SetException(new Sys.InvalidOperationException('stored failure'));
                return 0;
              end function);
              try
                discard Pending.GetAwaiter().GetResult();
              catch Error: Sys.Exception do
                Console.WriteLine('caught');
              end try;
              discard Releaser.GetAwaiter().GetResult();
            end;
            end program;
            """;
        if (cancelled) source = source.Replace("Gate.SetException(new Sys.InvalidOperationException('stored failure'));", "Gate.SetCanceled();", StringComparison.Ordinal);
        await program.Compile(source);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, 6, deadline.Token);
        var before = host.Snapshot;
        await host.SetExceptions(new(Raised: true), deadline.Token);
        await host.SetBreakpoints(new(program.Source, [], before.StopId), deadline.Token);
        await host.Resume(before.StopId!.Value, "over", deadline.Token);
        while (!string.Concat(host.Output().Items.Select(item => item.Text)).Contains("pending", StringComparison.Ordinal))
            await Task.Delay(10, deadline.Token);
        var waiting = host.Snapshot;
        while (waiting.AsyncStep?.Phase != "yielded") waiting = await host.WaitForChange(waiting.Revision, deadline.Token);
        File.WriteAllText(program.FilePath("release.txt"), "go");
        var stopped = await host.WaitForStop(deadline.Token);
        Assert.Equal("exception", stopped.Reason); Assert.Null(stopped.AsyncStep);
        var exception = await host.ExceptionInfo(stopped.StopId!.Value, deadline.Token);
        Assert.Equal(cancelled ? "System.Threading.Tasks.TaskCanceledException" : "System.InvalidOperationException", exception.Type);
        if (!cancelled) Assert.Equal("stored failure", exception.Message.Text);
        Assert.Contains(exception.CurrentStack, frame => frame.Name.EndsWith(".Work", StringComparison.Ordinal));
        await host.SetExceptions(new(), deadline.Token);
        await host.Resume(stopped.StopId.Value, "continue", deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        Assert.Null(host.Snapshot.AsyncStep); Assert.Equal(0, host.Snapshot.ExitCode);
        await host.Stop(deadline.Token);
        Assert.Contains("caught", string.Concat(host.Output().Items.Select(item => item.Text)));
    }
}
