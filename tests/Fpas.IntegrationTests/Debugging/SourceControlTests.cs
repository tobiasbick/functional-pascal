using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;
using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class SourceControlTests
{
    private static WorkerLauncher Launcher => WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost());

    private static async Task<DebugSessionHost> Launch(Debuggee program, CancellationToken token)
    {
        var host = await DebugSessionHost.Start(Launcher, program.DirectoryPath, token);
        try
        {
            await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath), token);
            await host.Initialize(token);
            var entry = await host.WaitForStop(token);
            Assert.Equal("entry", entry.Reason);
            return host;
        }
        catch { await host.DisposeAsync(); throw; }
    }

    [Fact]
    public async Task Entry_configuration_merges_actual_locations_replaces_and_removes_breakpoints()
    {
        using var program = new Debuggee("own-source-breakpoints");
        await program.Compile("""
            program MultiBreak;
            uses System.Console as Console, System.IO.File as File;
            begin
              File.WriteAllText('started.txt', 'yes');
              var Value: int := 10;
              // Before the first update.
              Value := Value + 1;
              Console.WriteLine(Value);
              Value := Value + 2;
              Console.WriteLine(Value);
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, deadline.Token);
        Assert.False(File.Exists(program.FilePath("started.txt")));
        Assert.Empty(host.Output().Items);
        var configured = await host.SetBreakpoints(new(program.Source, [new(6), new(7), new(8), new(7), new(1000)], host.Snapshot.StopId), deadline.Token);
        Assert.Equal(4, configured.Length);
        Assert.All(configured.Take(3), breakpoint => Assert.Equal("bound", breakpoint.State));
        Assert.Equal("rejected", configured[3].State);
        Assert.Equal("non_executable", configured[3].Error!.Code);
        Assert.Null(configured[3].Actual);
        Assert.Equal(7, configured[0].Actual!.Line); Assert.Equal(7, configured[1].Actual!.Line);
        await host.Resume(host.Snapshot.StopId!.Value, "continue", deadline.Token);
        var first = await host.WaitForStop(deadline.Token);
        Assert.Equal("breakpoint", first.Reason);
        Assert.Equal(configured.Take(2).Select(item => item.Id), first.HitBreakpoints);
        Assert.Contains(await host.Bindings(first.StopId!.Value, deadline.Token), item => item.Name == "Value" && item.Scalar == "10");
        Assert.True(File.Exists(program.FilePath("started.txt")));
        var replaced = Assert.Single(await host.SetBreakpoints(new(program.Source, [new(9)], first.StopId), deadline.Token));
        await host.Resume(first.StopId.Value, "continue", deadline.Token);
        var second = await host.WaitForStop(deadline.Token);
        Assert.Equal(new[] { replaced.Id }, second.HitBreakpoints);
        Assert.Contains(await host.Bindings(second.StopId!.Value, deadline.Token), item => item.Name == "Value" && item.Scalar == "11");
        Assert.Empty(await host.SetBreakpoints(new(program.Source, [], second.StopId), deadline.Token));
        await host.Resume(second.StopId.Value, "continue", deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        await host.Stop(deadline.Token);
        Assert.Equal("11\n13\n", string.Concat(host.Output().Items.Select(item => item.Text)).Replace("\r\n", "\n", StringComparison.Ordinal));
    }

    [Fact]
    public async Task Source_into_over_and_out_stop_at_user_statements_with_current_frame_values()
    {
        using var program = new Debuggee("own-source-step");
        await program.Compile("""
            program SourceStep;
            uses System.Console as Console;
            function AddTwo(Value: int): int;
            begin
              const Result: int := Value + 2;
              return Result;
            end function;
            function Work(): int;
            begin
              var Value: int := 20;
              Value := AddTwo(Value);
              Value := Value + 1;
              return Value;
            end function;
            begin
              Console.WriteLine(Work());
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, deadline.Token);
        await host.SetBreakpoints(new(program.Source, [new(11)], host.Snapshot.StopId), deadline.Token);
        await host.Resume(host.Snapshot.StopId!.Value, "continue", deadline.Token);
        var call = await host.WaitForStop(deadline.Token);
        Assert.Contains(await host.Bindings(call.StopId!.Value, deadline.Token), item => item.Name == "Value" && item.Scalar == "20");
        await host.SetBreakpoints(new(program.Source, [], call.StopId), deadline.Token);
        var into = await Step(host, call, "into", 5, deadline.Token);
        Assert.EndsWith(".AddTwo", into.Frame.Name);
        Assert.Contains(into.Values, item => item.Name == "Value" && item.Scalar == "20");
        var over = await Step(host, into.Stop, "over", 6, deadline.Token);
        Assert.Contains(over.Values, item => item.Name == "Result" && item.Scalar == "22");
        var exited = await Step(host, over.Stop, "out", 12, deadline.Token);
        Assert.EndsWith(".Work", exited.Frame.Name);
        Assert.Contains(exited.Values, item => item.Name == "Value" && item.Scalar == "22");
        var final = await Step(host, exited.Stop, "over", 13, deadline.Token);
        Assert.Contains(final.Values, item => item.Name == "Value" && item.Scalar == "23");
        await host.Resume(final.Stop.StopId!.Value, "continue", deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        await host.Stop(deadline.Token);
        Assert.Contains("23", string.Concat(host.Output().Items.Select(item => item.Text)));
    }

    private static async Task<(DebugSnapshot Stop, DebugFrame Frame, DebugBinding[] Values)> Step(DebugSessionHost host, DebugSnapshot previous,
        string kind, int line, CancellationToken token)
    {
        await host.Resume(previous.StopId!.Value, kind, token);
        var stop = await host.WaitForStop(token);
        Assert.Equal("step", stop.Reason);
        var thread = Assert.Single(await host.Threads(stop.StopId!.Value, token), item => item.IsStopThread);
        var frame = (await host.Stack(new(stop.StopId.Value, thread.Id), token)).Items[0];
        Assert.Equal(line, frame.Source!.Line);
        return (stop, frame, (await host.FrameBindings(new(stop.StopId.Value, frame.Id), token)).Items);
    }
}
