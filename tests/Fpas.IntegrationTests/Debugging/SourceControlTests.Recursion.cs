using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class SourceControlTests
{
    [Fact]
    public async Task Recursive_into_out_and_over_preserve_invocation_depth_and_values()
    {
        using var program = new Debuggee("own-recursive-step");
        await program.Compile("""
            program RecursiveStep;
            uses System.Console as Console;
            function Sum(Depth: int): int;
            begin
              if Depth = 0 then
                return 10;
              end if;
              const Nested: int := Sum(Depth - 1);
              return Nested + Depth;
            end function;
            begin
              const First: int := Sum(2);
              const Second: int := Sum(2);
              Console.WriteLine(First + Second);
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, deadline.Token);
        await host.SetBreakpoints(new(program.Source, [new(8)], host.Snapshot.StopId), deadline.Token);
        await host.Resume(host.Snapshot.StopId!.Value, "continue", deadline.Token);
        var first = await host.WaitForStop(deadline.Token);
        await host.SetBreakpoints(new(program.Source, [], first.StopId), deadline.Token);
        var child = await Step(host, first, "into", 5, deadline.Token);
        await RecursiveFrames(host, child.Stop, 2, deadline.Token);
        Assert.Contains(child.Values, item => item.Name == "Depth" && item.Scalar == "1");
        var call = await Step(host, child.Stop, "over", 8, deadline.Token);
        var leaf = await Step(host, call.Stop, "into", 5, deadline.Token);
        await RecursiveFrames(host, leaf.Stop, 3, deadline.Token);
        Assert.Contains(leaf.Values, item => item.Name == "Depth" && item.Scalar == "0");
        var returning = await Step(host, leaf.Stop, "over", 6, deadline.Token);
        var parent = await Step(host, returning.Stop, "out", 9, deadline.Token);
        await RecursiveFrames(host, parent.Stop, 2, deadline.Token);
        Assert.Contains(parent.Values, item => item.Name == "Nested" && item.Scalar == "10");
        var root = await Step(host, parent.Stop, "out", 9, deadline.Token);
        await RecursiveFrames(host, root.Stop, 1, deadline.Token);
        Assert.Contains(root.Values, item => item.Name == "Nested" && item.Scalar == "11");
        await host.SetBreakpoints(new(program.Source, [new(13)], root.Stop.StopId), deadline.Token);
        await host.Resume(root.Stop.StopId!.Value, "continue", deadline.Token);
        var secondCall = await host.WaitForStop(deadline.Token);
        Assert.Equal("breakpoint", secondCall.Reason);
        await host.SetBreakpoints(new(program.Source, [], secondCall.StopId), deadline.Token);
        var after = await Step(host, secondCall, "over", 14, deadline.Token);
        await RecursiveFrames(host, after.Stop, 0, deadline.Token);
        Assert.Contains(after.Values, item => item.Name == "First" && item.Scalar == "13");
        Assert.Contains(after.Values, item => item.Name == "Second" && item.Scalar == "13");
        await host.Resume(after.Stop.StopId!.Value, "continue", deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        await host.Stop(deadline.Token);
        Assert.Contains("26", string.Concat(host.Output().Items.Select(item => item.Text)));
    }

    private static async Task RecursiveFrames(DebugSessionHost host, DebugSnapshot stop, int count, CancellationToken token)
    {
        var thread = Assert.Single(await host.Threads(stop.StopId!.Value, token), item => item.IsStopThread);
        var stack = await host.Stack(new(stop.StopId.Value, thread.Id), token);
        Assert.Equal(count, stack.Items.Count(frame => frame.Name.EndsWith(".Sum", StringComparison.Ordinal)));
    }

    [Fact]
    public async Task An_unhandled_exception_interrupts_a_step_with_an_exception_stop()
    {
        using var program = new Debuggee("own-step-exception");
        await program.Compile("""
            program StepFailure;
            uses System as Sys, System.Console as Console;
            procedure Fail();
            begin
              raise new Sys.InvalidOperationException('step failure');
            end procedure;
            begin
              Fail();
              Console.WriteLine('unreachable');
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, deadline.Token);
        await host.SetBreakpoints(new(program.Source, [new(8)], host.Snapshot.StopId), deadline.Token);
        await host.Resume(host.Snapshot.StopId!.Value, "continue", deadline.Token);
        var call = await host.WaitForStop(deadline.Token);
        await host.SetBreakpoints(new(program.Source, [], call.StopId), deadline.Token);
        var throwing = await Step(host, call, "into", 5, deadline.Token);
        await host.Resume(throwing.Stop.StopId!.Value, "over", deadline.Token);
        var interrupted = await host.WaitForStop(deadline.Token);
        Assert.Equal("stopped", interrupted.State);
        Assert.Equal("exception", interrupted.Reason);
        Assert.NotEqual(throwing.Stop.StopId, interrupted.StopId);
        await host.Stop(deadline.Token);
        Assert.DoesNotContain("unreachable", string.Concat(host.Output().Items.Select(item => item.Text)), StringComparison.Ordinal);
    }
}
