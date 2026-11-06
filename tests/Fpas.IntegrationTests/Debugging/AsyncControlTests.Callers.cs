using Fpas.Debugging.Core;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class AsyncControlTests
{
    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task Unavailable_or_ambiguous_async_callers_leave_the_existing_stop_usable(bool multiple)
    {
        using var program = new Debuggee("own-async-callers");
        var source = """
            program AsyncCallers;
            uses System as Sys, System.Console as Console, System.Threading.Tasks as Tasks,
              System.IO.File as File, System.Threading.Thread as Thread;
            async function Child(Gate: Tasks.Task<int>): Tasks.Task<int>;
            begin
              const Release: int := await Gate;
              var Value: int := Release + 19;
              return Value;
            end function;
            async function Parent(Pending: Tasks.Task<int>; Increment: int): Tasks.Task<int>;
            begin
              const Value: int := await Pending;
              return Value + Increment;
            end function;
            begin
              const Gate: Tasks.TaskCompletionSource<int> := new Tasks.TaskCompletionSource<int>(Tasks.TaskCreationOptions.RunContinuationsAsynchronously);
              const Pending: Tasks.Task<int> := Child(Gate.Task);
              // PARENTS
              Console.WriteLine('pending');
              const Releaser: Tasks.Task<int> := Tasks.Task.Run<int>(function(): int
              begin
                while not File.Exists('release.txt') do Thread.Sleep(10); end while;
                Gate.SetResult(22);
                return 0;
              end function);
              Console.WriteLine(Pending.GetAwaiter().GetResult());
              // RESULTS
              discard Releaser.GetAwaiter().GetResult();
            end;
            end program;
            """;
        if (multiple) source = source.Replace("// PARENTS", "const First: Tasks.Task<int> := Parent(Pending, 1); const Second: Tasks.Task<int> := Parent(Pending, 2);", StringComparison.Ordinal)
            .Replace("// RESULTS", "Console.WriteLine(First.GetAwaiter().GetResult()); Console.WriteLine(Second.GetAwaiter().GetResult());", StringComparison.Ordinal);
        await program.Compile(source);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, 6, deadline.Token);
        var before = host.Snapshot;
        await host.SetBreakpoints(new(program.Source, [], before.StopId), deadline.Token);
        await host.Resume(before.StopId!.Value, "over", deadline.Token);
        while (!string.Concat(host.Output().Items.Select(item => item.Text)).Contains("pending", StringComparison.Ordinal))
            await Task.Delay(10, deadline.Token);
        File.WriteAllText(program.FilePath("release.txt"), "go");
        var resumed = await host.WaitForStop(deadline.Token);
        var child = await Top(host, resumed, deadline.Token);
        Assert.Equal(7, child.Frame.Source!.Line);
        var error = await Assert.ThrowsAsync<DebugException>(() => host.Resume(resumed.StopId!.Value, "out", deadline.Token));
        Assert.Equal(multiple ? "ambiguous_caller" : "unavailable", error.Code);
        var retained = await host.Status(deadline.Token);
        Assert.Equal("stopped", retained.State); Assert.Equal(resumed.StopId, retained.StopId); Assert.Null(retained.AsyncStep);
        Assert.Equal(resumed.Revision, retained.Revision);
        Assert.Contains((await host.FrameBindings(new(retained.StopId!.Value, child.Frame.Id), deadline.Token)).Items, item => item.Name == "Release" && item.Scalar == "22");
        await host.Resume(retained.StopId.Value, "continue", deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        await host.Stop(deadline.Token);
        var output = string.Concat(host.Output().Items.Select(item => item.Text));
        Assert.Contains("41", output);
        if (multiple) { Assert.Contains("42", output); Assert.Contains("43", output); }
    }
}
