using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;
using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Debugging;

public sealed class CaptureInspectionTests
{
    [Fact]
    public async Task Declaring_locals_and_async_closure_receivers_share_the_same_marked_cell()
    {
        using var program = new Debuggee("own-async-closure-captures");
        var source = """
            program AsyncClosure;
            uses System.Console as Console, System.Threading.Tasks as Tasks, System.Runtime.CompilerServices as Runtime;
            function Factory(Seed: int): function(): Tasks.Task<int>;
            begin
              var Value: int := Seed;
              const Box: Runtime.StrongBox<int> := new Runtime.StrongBox<int>(70);
              Console.WriteLine('declaring');
              return async function(): Tasks.Task<int> begin
                await Tasks.Task.Delay(20);
                Value := Value + 2;
                Console.WriteLine('closure');
                return Value + Box.Value;
              end function;
            end function;
            begin
              const Read: function(): Tasks.Task<int> := Factory(40);
              Console.WriteLine(Read().GetAwaiter().GetResult());
            end;
            end program;
            """;
        await program.Compile(source);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, 7, deadline.Token);
        var declaring = host.Snapshot; var frame = await Frame(host, declaring, deadline.Token);
        var locals = await host.FrameBindings(new(declaring.StopId!.Value, frame.Id), deadline.Token);
        Assert.Equal("40", Assert.Single(locals.Items, item => item.Name == "Value").Scalar);
        Assert.Equal("object", Assert.Single(locals.Items, item => item.Name == "Box").Kind);
        var raw = await host.FrameBindings(new(declaring.StopId.Value, frame.Id, View: "raw"), deadline.Token);
        Assert.Equal("object", Assert.Single(raw.Items, item => item.Name == "Value").Kind);
        await host.SetBreakpoints(new(program.Source, [new(11)], declaring.StopId), deadline.Token);
        await host.Resume(declaring.StopId.Value, "continue", deadline.Token); var resumed = await host.WaitForStop(deadline.Token);
        var captured = await host.FrameBindings(new(resumed.StopId!.Value, (await Frame(host, resumed, deadline.Token)).Id), deadline.Token);
        Assert.Equal("42", Assert.Single(captured.Items, item => item.Name == "Value").Scalar);
        var box = Assert.Single(captured.Items, item => item.Name == "Box"); Assert.Equal("object", box.Kind);
        Assert.Equal("70", Assert.Single((await host.ValueChildren(new(resumed.StopId.Value, box.Reference!.Value), deadline.Token)).Items).Scalar);
        await host.Resume(resumed.StopId.Value, "continue", deadline.Token);
        Assert.Equal(0, (await host.WaitForStop(deadline.Token)).ExitCode); await host.Stop(deadline.Token);
        Assert.Contains("112", string.Concat(host.Output().Items.Select(item => item.Text)));
    }

    [Fact]
    public async Task Captured_source_bindings_use_symbols_for_cells_and_preserve_raw_values_and_shadowing()
    {
        using var program = new Debuggee("own-capture-values");
        var source = """
            program Captures;
            uses System.Console as Console, System.Runtime.CompilerServices as Runtime;
            function Factory(Seed: int): function(int): int;
            begin
              var Value: int := Seed;
              const Box: Runtime.StrongBox<int> := new Runtime.StrongBox<int>(70);
              return function(Input: int): int begin
                Value := Value + Input;
                Console.WriteLine('captured');
                begin
                  Console.WriteLine('before shadow');
                  const Value: int := 99;
                  Console.WriteLine('shadowed');
                end;
                Console.WriteLine('restored');
                return Value + Box.Value;
              end function;
            end function;
            begin
              const Read: function(int): int := Factory(40);
              Console.WriteLine(Read(2));
            end;
            end program;
            """;
        await program.Compile(source);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, 9, deadline.Token);
        var stop = host.Snapshot; var frame = await Frame(host, stop, deadline.Token);
        var sourceValues = await host.FrameBindings(new(stop.StopId!.Value, frame.Id), deadline.Token);
        Assert.Equal("42", Assert.Single(sourceValues.Items, item => item.Name == "Value").Scalar);
        Assert.Equal("2", Assert.Single(sourceValues.Items, item => item.Name == "Input").Scalar);
        var box = Assert.Single(sourceValues.Items, item => item.Name == "Box"); Assert.Equal("object", box.Kind);
        Assert.Equal("70", Assert.Single((await host.ValueChildren(new(stop.StopId.Value, box.Reference!.Value), deadline.Token)).Items).Scalar);
        var raw = await host.FrameBindings(new(stop.StopId.Value, frame.Id, View: "raw"), deadline.Token);
        Assert.DoesNotContain(raw.Items, item => item.Name == "Value");
        var receiver = Assert.Single(raw.Items, item => item.Name == "this");
        var closure = await host.ValueChildren(new(stop.StopId.Value, receiver.Reference!.Value), deadline.Token);
        Assert.Equal("object", Assert.Single(closure.Items, item => item.Name == "Value").Kind);
        await host.SetBreakpoints(new(program.Source, [new(11), new(13), new(15)], stop.StopId), deadline.Token);
        await host.Resume(stop.StopId.Value, "continue", deadline.Token); var before = await host.WaitForStop(deadline.Token);
        var beforeShadow = await host.FrameBindings(new(before.StopId!.Value, (await Frame(host, before, deadline.Token)).Id), deadline.Token);
        Assert.Equal("42", Assert.Single(beforeShadow.Items, item => item.Name == "Value").Scalar);
        await host.Resume(before.StopId.Value, "continue", deadline.Token); var shadow = await host.WaitForStop(deadline.Token);
        var shadowed = await host.FrameBindings(new(shadow.StopId!.Value, (await Frame(host, shadow, deadline.Token)).Id), deadline.Token);
        Assert.Equal("99", Assert.Single(shadowed.Items, item => item.Name == "Value").Scalar);
        await host.Resume(shadow.StopId.Value, "continue", deadline.Token); var restored = await host.WaitForStop(deadline.Token);
        var outer = await host.FrameBindings(new(restored.StopId!.Value, (await Frame(host, restored, deadline.Token)).Id), deadline.Token);
        Assert.Equal("42", Assert.Single(outer.Items, item => item.Name == "Value").Scalar);
        await host.Stop(deadline.Token);
    }

    [Fact]
    public async Task Active_declaring_frames_and_async_hoisted_scopes_decode_only_compiler_marked_cells()
    {
        using var program = new Debuggee("own-async-capture-values");
        var source = """
            program AsyncCaptures;
            uses System.Console as Console, System.Threading.Tasks as Tasks, System.Runtime.CompilerServices as Runtime;
            async function Observe(): Tasks.Task;
            begin
              var Value: int := 40;
              const Box: Runtime.StrongBox<int> := new Runtime.StrongBox<int>(70);
              const Read: function(): int := function(): int => Value + Box.Value;
              Console.WriteLine('before');
              await Tasks.Task.Delay(20);
              Value := Value + 2;
              begin
                const Value: int := 99;
                Console.WriteLine('inner');
              end;
              Console.WriteLine('outer');
              Console.WriteLine(Read());
            end function;
            begin
              Observe().GetAwaiter().GetResult();
            end;
            end program;
            """;
        await program.Compile(source);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await Launch(program, 8, deadline.Token);
        var first = host.Snapshot; var firstFrame = await Frame(host, first, deadline.Token);
        var initial = await host.FrameBindings(new(first.StopId!.Value, firstFrame.Id), deadline.Token);
        Assert.Equal("40", Assert.Single(initial.Items, item => item.Name == "Value").Scalar);
        Assert.Equal("object", Assert.Single(initial.Items, item => item.Name == "Box").Kind);
        var raw = await host.FrameBindings(new(first.StopId.Value, firstFrame.Id, View: "raw"), deadline.Token);
        Assert.Equal("object", Assert.Single(raw.Items, item => item.Name == "Value").Kind);
        await host.SetBreakpoints(new(program.Source, [new(13), new(15)], first.StopId), deadline.Token);
        await host.Resume(first.StopId.Value, "continue", deadline.Token); var inner = await host.WaitForStop(deadline.Token);
        var local = await host.FrameBindings(new(inner.StopId!.Value, (await Frame(host, inner, deadline.Token)).Id), deadline.Token);
        Assert.Equal("99", Assert.Single(local.Items, item => item.Name == "Value").Scalar);
        await host.Resume(inner.StopId.Value, "continue", deadline.Token); var outer = await host.WaitForStop(deadline.Token);
        var values = await host.FrameBindings(new(outer.StopId!.Value, (await Frame(host, outer, deadline.Token)).Id), deadline.Token);
        Assert.Equal("42", Assert.Single(values.Items, item => item.Name == "Value").Scalar);
        await host.Stop(deadline.Token);
    }

    private static async Task<DebugSessionHost> Launch(Debuggee program, int line, CancellationToken token)
    {
        var host = await DebugSessionHost.Start(WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost()), program.DirectoryPath, token);
        try
        {
            await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath, new(program.Source, line)), token);
            await host.Initialize(token); Assert.Equal("breakpoint", (await host.WaitForStop(token)).Reason); return host;
        }
        catch { await host.DisposeAsync(); throw; }
    }

    private static async Task<DebugFrame> Frame(DebugSessionHost host, DebugSnapshot stop, CancellationToken token)
    {
        var thread = Assert.Single(await host.Threads(stop.StopId!.Value, token), item => item.IsStopThread);
        return (await host.Stack(new(stop.StopId.Value, thread.Id), token)).Items[0];
    }
}
