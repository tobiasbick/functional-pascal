using Fpas.Debugging.Core;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class AsyncControlTests
{
    [Theory]
    [InlineData("Gate", false, false, false)]
    [InlineData("Gate.ConfigureAwait(false)", true, false, false)]
    [InlineData("new Tasks.ValueTask<int>(Gate)", false, false, false)]
    [InlineData("new Tasks.ValueTask<int>(Gate).ConfigureAwait(false)", true, false, false)]
    [InlineData("Gate", true, true, false)]
    [InlineData("Gate", false, true, false)]
    [InlineData("Gate", false, false, true)]
    public async Task Step_out_from_a_resumed_child_follows_the_awaiting_parent(string awaited, bool generic, bool optimized, bool loadTwice)
    {
        using var program = new Debuggee("own-async-return");
        var source = """
            program AsyncReturn;
            uses System as Sys, System.Console as Console, System.Threading.Tasks as Tasks,
              System.IO.File as File, System.Threading.Thread as Thread;
            async function Child(Input: int; Gate: Tasks.Task<int>): Tasks.Task<int>;
            begin
              const Release: int := await Gate;
              var Value: int := Input + Release;
              return Value;
            end function;
            async function Parent(Input: int; Gate: Tasks.Task<int>): Tasks.Task<int>;
            begin
              const ChildValue: int := await Child(Input, Gate);
              var Result: int := ChildValue + 1;
              return Result;
            end function;
            begin
              const Gate: Tasks.TaskCompletionSource<int> := new Tasks.TaskCompletionSource<int>(Tasks.TaskCreationOptions.RunContinuationsAsynchronously);
              const Pending: Tasks.Task<int> := Parent(19, Gate.Task);
              if Pending.IsCompleted then raise new Sys.InvalidOperationException('Must suspend'); end if;
              Console.WriteLine('pending');
              const Releaser: Tasks.Task<int> := Tasks.Task.Run<int>(function(): int
              begin
                while not File.Exists('release.txt') do Thread.Sleep(10); end while;
                Gate.SetResult(22);
                return 0;
              end function);
              Console.WriteLine(Pending.GetAwaiter().GetResult());
              discard Releaser.GetAwaiter().GetResult();
            end;
            end program;
            """;
        source = source.Replace("await Gate;", $"await {awaited};", StringComparison.Ordinal);
        if (loadTwice) source = source.Replace("System.IO.File as File, System.Threading.Thread as Thread;",
            "System.IO.File as File, System.Threading.Thread as Thread, System.Runtime.Loader.AssemblyLoadContext as LoadContext, System.IO.Path as Path;", StringComparison.Ordinal)
            .Replace("Console.WriteLine('pending');", "Console.WriteLine('pending'); const Duplicate: LoadContext := new LoadContext('duplicate', false); discard Duplicate.LoadFromAssemblyPath(Path.GetFullPath('Program.dll'));", StringComparison.Ordinal);
        if (generic) source = source.Replace("Child(", "Child<T>(", StringComparison.Ordinal)
            .Replace("await Child<T>(Input, Gate)", "await Child<T>(Input, Gate).ConfigureAwait(false)", StringComparison.Ordinal)
            .Replace("Parent(", "Parent<T>(", StringComparison.Ordinal).Replace("Parent<T>(19,", "Parent<int>(19,", StringComparison.Ordinal);
        await program.Compile(source, optimized ? ["--optimize"] : []);
        if (optimized) program.UseFullyOptimizedJit();
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, 6, deadline.Token);
        var before = host.Snapshot;
        var child = await Top(host, before, deadline.Token);
        Assert.EndsWith(".Child", child.Frame.Name);
        Assert.Equal(6, child.Frame.Source?.Line);
        await host.SetBreakpoints(new(program.Source, [], before.StopId), deadline.Token);
        await host.Resume(before.StopId!.Value, "over", deadline.Token);
        while (!string.Concat(host.Output().Items.Select(item => item.Text)).Contains("pending", StringComparison.Ordinal))
            await Task.Delay(10, deadline.Token);
        File.WriteAllText(program.FilePath("release.txt"), "go");
        var resumed = await host.WaitForStop(deadline.Token);
        Assert.Equal("step", resumed.Reason);
        var returning = await Top(host, resumed, deadline.Token);
        Assert.Equal(optimized ? 8 : 7, returning.Frame.Source!.Line);
        if (optimized) Assert.True(child.Frame.Approximate);
        Assert.Equal(child.Frame.Async!.Id, returning.Frame.Async!.Id);
        Assert.NotEqual(child.Frame.ThreadId, returning.Frame.ThreadId);
        await host.Resume(resumed.StopId!.Value, "out", deadline.Token);
        var parentStop = await host.WaitForStop(deadline.Token);
        Assert.Equal("step", parentStop.Reason);
        var parent = await Top(host, parentStop, deadline.Token);
        Assert.EndsWith(".Parent", parent.Frame.Name);
        Assert.Equal(optimized ? 14 : 13, parent.Frame.Source!.Line);
        Assert.NotEqual(child.Frame.Async.Id, parent.Frame.Async!.Id);
        if (parent.Frame.Async.Id is null) Assert.Equal("optimized_out", parent.Frame.Async.IdentityStatus);
        CheckValue("Input", "19"); CheckValue("ChildValue", "41");
        if (parent.Frame.Async.IdentityStatus == "optimized_out")
        {
            var unavailable = await Assert.ThrowsAsync<DebugException>(() => host.Resume(parentStop.StopId!.Value, "out", deadline.Token));
            Assert.Equal("unavailable", unavailable.Code);
            var retained = await host.Status(deadline.Token);
            Assert.Equal(parentStop.Revision, retained.Revision); Assert.Equal(parentStop.StopId, retained.StopId);
        }
        await host.Resume(parentStop.StopId!.Value, "continue", deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        await host.Stop(deadline.Token);
        Assert.Contains("42", string.Concat(host.Output().Items.Select(item => item.Text)));

        void CheckValue(string name, string expected)
        {
            var value = Assert.Single(parent.Values, item => item.Name == name);
            if (optimized && value.Status == "optimized_out") Assert.Null(value.Scalar);
            else { Assert.Equal("available", value.Status); Assert.Equal(expected, value.Scalar); }
        }
    }
}
