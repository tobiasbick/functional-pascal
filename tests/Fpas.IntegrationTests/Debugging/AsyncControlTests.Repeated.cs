namespace Fpas.IntegrationTests.Debugging;

public sealed partial class AsyncControlTests
{
    [Theory]
    [InlineData("over")]
    [InlineData("into")]
    public async Task One_generic_async_instance_can_be_stepped_across_two_real_suspensions(string kind)
    {
        using var program = new Debuggee("own-async-repeated");
        await program.Compile("""
            program Repeated;
            uses System as Sys, System.Console as Console, System.Threading.Tasks as Tasks,
              System.IO.File as File, System.Threading.Thread as Thread;
            async function Work<T>(Input: T; First: Tasks.Task<int>; Second: Tasks.Task<int>): Tasks.Task<int>;
            begin
              const Value: int := await First.ConfigureAwait(false);
              Console.WriteLine(Value);
              const Other: int := await new Tasks.ValueTask<int>(Second).ConfigureAwait(false);
              return Value + Other;
            end function;
            begin
              const First: Tasks.TaskCompletionSource<int> := new Tasks.TaskCompletionSource<int>(Tasks.TaskCreationOptions.RunContinuationsAsynchronously);
              const Second: Tasks.TaskCompletionSource<int> := new Tasks.TaskCompletionSource<int>(Tasks.TaskCreationOptions.RunContinuationsAsynchronously);
              const Pending: Tasks.Task<int> := Work<int>(19, First.Task, Second.Task);
              if Pending.IsCompleted then raise new Sys.InvalidOperationException('Must suspend'); end if;
              Console.WriteLine('first');
              const Releaser: Tasks.Task<int> := Tasks.Task.Run<int>(function(): int
              begin
                while not File.Exists('first.txt') do Thread.Sleep(10); end while;
                First.SetResult(20);
                while not File.Exists('second.txt') do Thread.Sleep(10); end while;
                if Pending.IsCompleted then raise new Sys.InvalidOperationException('Second await must suspend'); end if;
                Console.WriteLine('second');
                Second.SetResult(22);
                return 0;
              end function);
              Console.WriteLine(Pending.GetAwaiter().GetResult());
              discard Releaser.GetAwaiter().GetResult();
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, 6, deadline.Token);
        var first = host.Snapshot; var original = await Top(host, first, deadline.Token);
        await host.SetBreakpoints(new(program.Source, [], first.StopId), deadline.Token);
        await host.Resume(first.StopId!.Value, kind, deadline.Token);
        while (!string.Concat(host.Output().Items.Select(item => item.Text)).Contains("first", StringComparison.Ordinal))
            await Task.Delay(10, deadline.Token);
        File.WriteAllText(program.FilePath("first.txt"), "go");
        var firstResume = await host.WaitForStop(deadline.Token);
        var middle = await Top(host, firstResume, deadline.Token);
        Assert.Equal(7, middle.Frame.Source!.Line);
        Assert.Equal(original.Frame.Async!.Id, middle.Frame.Async!.Id);
        Assert.Contains(middle.Values, item => item.Name == "Input" && item.Scalar == "19");
        Assert.Contains(middle.Values, item => item.Name == "Value" && item.Scalar == "20");
        await host.Resume(firstResume.StopId!.Value, "over", deadline.Token);
        var second = await host.WaitForStop(deadline.Token);
        Assert.Equal(8, (await Top(host, second, deadline.Token)).Frame.Source!.Line);
        await host.Resume(second.StopId!.Value, kind, deadline.Token);
        var waiting = host.Snapshot;
        while (waiting.AsyncStep?.Phase != "yielded") waiting = await host.WaitForChange(waiting.Revision, deadline.Token);
        Assert.Equal(original.Frame.Async.Id, waiting.AsyncStep!.Id);
        Assert.Equal("running", waiting.State);
        File.WriteAllText(program.FilePath("second.txt"), "go");
        var final = await host.WaitForStop(deadline.Token);
        var returned = await Top(host, final, deadline.Token);
        Assert.Equal(9, returned.Frame.Source!.Line);
        Assert.Equal(original.Frame.Async.Id, returned.Frame.Async!.Id);
        Assert.Contains(returned.Values, item => item.Name == "Value" && item.Scalar == "20");
        Assert.Contains(returned.Values, item => item.Name == "Other" && item.Scalar == "22");
        await host.Resume(final.StopId!.Value, "continue", deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        await host.Stop(deadline.Token);
        var output = string.Concat(host.Output().Items.Select(item => item.Text));
        Assert.Contains("second", output); Assert.Contains("42", output);
    }
}
