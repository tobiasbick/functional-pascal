using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class SourceControlTests
{
    [Fact]
    public async Task Pending_library_breakpoints_bind_before_the_library_routine_executes()
    {
        using var program = new Debuggee("own-pending-module");
        var source = program.FilePath("Plugin.fpas"); var library = program.FilePath("Plugin.dll");
        File.WriteAllText(source, """
            unit Delayed;
            public function Compute(Value: int): int;
            begin
              const Result: int := Value + 1;
              return Result;
            end function;
            end unit;
            """);
        await program.CompileFile(source, library, "--target", "library");
        await program.Compile("""
            program PendingModule;
            uses Delayed as L, System.Console as Console;
            function InvokeLater(): int;
            begin
              return L.Compute(41);
            end function;
            begin
              Console.WriteLine(InvokeLater());
            end;
            end program;
            """, "--reference", library);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, deadline.Token);
        var pending = Assert.Single(await host.SetBreakpoints(new(source, [new(5)], host.Snapshot.StopId), deadline.Token));
        Assert.Equal("pending", pending.State);
        await host.Resume(host.Snapshot.StopId!.Value, "continue", deadline.Token);
        var hit = await host.WaitForStop(deadline.Token);
        Assert.Equal("breakpoint", hit.Reason);
        Assert.Equal(new[] { pending.Id }, hit.HitBreakpoints);
        var bound = Assert.Single(await host.Breakpoints(deadline.Token));
        Assert.Equal(bound, Assert.Single(hit.Breakpoints!));
        Assert.Equal("bound", bound.State); Assert.Equal(1, bound.Hits);
        Assert.Equal(source, bound.Actual!.Path); Assert.Equal(5, bound.Actual.Line);
        Assert.Contains(await host.Bindings(hit.StopId!.Value, deadline.Token), item => item.Name == "Result" && item.Scalar == "42");
        await host.SetBreakpoints(new(source, [], hit.StopId), deadline.Token);
        await host.Resume(hit.StopId.Value, "continue", deadline.Token);
        Assert.Equal("exited", (await host.WaitForStop(deadline.Token)).State);
        await host.Stop(deadline.Token);
        Assert.Contains("42", string.Concat(host.Output().Items.Select(item => item.Text)));
    }

    [Fact]
    public async Task Running_breakpoint_updates_balance_their_private_pause_and_leave_user_pause_available()
    {
        using var program = new Debuggee("own-running-breakpoints");
        await program.Compile("""
            program RunningChanges;
            uses System.Threading.Thread as Thread, System.Console as Console;
            begin
              Thread.Sleep(60000);
              Console.WriteLine(42);
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, deadline.Token);
        await host.Resume(host.Snapshot.StopId!.Value, "continue", deadline.Token);
        for (var iteration = 0; iteration < 10; iteration++)
        {
            var changes = await host.SetBreakpoints(new(program.Source, iteration % 2 == 0 ? [new(5)] : []), deadline.Token);
            Assert.Equal(iteration % 2 == 0 ? 1 : 0, changes.Length);
            Assert.Equal("running", host.Snapshot.State);
        }
        await host.Pause(deadline.Token);
        Assert.Equal("pause", (await host.WaitForStop(deadline.Token)).Reason);
        await host.Stop(deadline.Token);
    }
}
