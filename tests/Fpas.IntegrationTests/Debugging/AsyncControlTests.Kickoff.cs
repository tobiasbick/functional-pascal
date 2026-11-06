namespace Fpas.IntegrationTests.Debugging;

public sealed partial class AsyncControlTests
{
    [Fact]
    public async Task Step_into_a_nested_generic_async_call_enters_the_child_source_body()
    {
        using var program = new Debuggee("own-nested-async-entry");
        await program.Compile("""
            program NestedEntry;
            uses System as Sys, System.Console as Console, System.Threading.Tasks as Tasks;
            async function Child<T>(Input: T; Gate: Tasks.Task<int>): Tasks.Task<T>;
            begin
              const Release: int := await Gate;
              return Input;
            end function;
            async function Parent(Gate: Tasks.Task<int>): Tasks.Task<int>;
            begin
              const Value: int := await Child<int>(42, Gate);
              return Value;
            end function;
            begin
              const Gate: Tasks.TaskCompletionSource<int> := new Tasks.TaskCompletionSource<int>(Tasks.TaskCreationOptions.RunContinuationsAsynchronously);
              const Pending: Tasks.Task<int> := Parent(Gate.Task);
              if Pending.IsCompleted then raise new Sys.InvalidOperationException('Must suspend'); end if;
              Gate.SetResult(1);
              Console.WriteLine(Pending.GetAwaiter().GetResult());
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, 10, deadline.Token);
        var before = host.Snapshot; var parent = await Top(host, before, deadline.Token);
        Assert.EndsWith(".Parent", parent.Frame.Name);
        await host.SetBreakpoints(new(program.Source, [], before.StopId), deadline.Token);
        await host.Resume(before.StopId!.Value, "into", deadline.Token);
        var entered = await host.WaitForStop(deadline.Token);
        var child = await Top(host, entered, deadline.Token);
        Assert.Equal("step", entered.Reason); Assert.EndsWith(".Child", child.Frame.Name);
        Assert.Equal(5, child.Frame.Source!.Line);
        Assert.NotEqual(parent.Frame.Async!.Id, child.Frame.Async!.Id);
        Assert.Contains(child.Values, item => item.Name == "Input" && item.Scalar == "42");
        await host.Resume(entered.StopId!.Value, "continue", deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        await host.Stop(deadline.Token);
        Assert.Contains("42", string.Concat(host.Output().Items.Select(item => item.Text)));
    }

    [Fact]
    public async Task Step_into_an_async_call_enters_its_user_body_instead_of_skipping_the_kickoff()
    {
        using var program = new Debuggee("own-async-kickoff");
        await program.Compile("""
            program AsyncKickoff;
            uses System as Sys, System.Console as Console, System.Threading.Tasks as Tasks;
            async function Work(Input: int; Gate: Tasks.Task<int>): Tasks.Task<int>;
            begin
              var Value: int := Input + 1;
              const Release: int := await Gate;
              return Value + Release;
            end function;
            begin
              const Gate: Tasks.TaskCompletionSource<int> := new Tasks.TaskCompletionSource<int>();
              const Pending: Tasks.Task<int> := Work(19, Gate.Task);
              if Pending.IsCompleted then raise new Sys.InvalidOperationException('Must suspend'); end if;
              Gate.SetResult(22);
              Console.WriteLine(Pending.GetAwaiter().GetResult());
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, 11, deadline.Token);
        var before = host.Snapshot;
        await host.SetBreakpoints(new(program.Source, [], before.StopId), deadline.Token);
        await host.Resume(before.StopId!.Value, "into", deadline.Token);
        var entered = await host.WaitForStop(deadline.Token);
        Assert.Equal("step", entered.Reason);
        var body = await Top(host, entered, deadline.Token);
        Assert.EndsWith(".Work", body.Frame.Name);
        Assert.NotNull(body.Frame.Async);
        Assert.Equal(5, body.Frame.Source!.Line);
        Assert.Contains(body.Values, item => item.Name == "Input" && item.Scalar == "19");
        await host.Resume(entered.StopId!.Value, "continue", deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        await host.Stop(deadline.Token);
        Assert.Contains("42", string.Concat(host.Output().Items.Select(item => item.Text)));
    }
}
