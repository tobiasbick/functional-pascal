namespace Fpas.IntegrationTests.Debugging;

public sealed partial class NetCoreDbgTests
{
    [NetCoreDbgFact]
    public async Task Real_async_suspension_retains_parameter_hoisted_values_and_source_stepping()
    {
        using var program = new Debuggee("async");
        await program.Compile("""
            async program AsyncProbe;
            uses System as Sys, System.Console as Console,
              System.Threading.Tasks as Tasks;
            async function Work(Input: int; Gate: Tasks.Task<int>): Tasks.Task<int>;
            begin
              var Value: int := Input + 1;
              const Release: int := await Gate;
              Value := Value + Release;
              return Value;
            end function;
            begin
              const Gate: Tasks.TaskCompletionSource<int> :=
                new Tasks.TaskCompletionSource<int>(Tasks.TaskCreationOptions.RunContinuationsAsynchronously);
              const Pending: Tasks.Task<int> := Work(19, Gate.Task);
              if Pending.IsCompleted then raise new Sys.InvalidOperationException('Must suspend'); end if;
              Gate.SetResult(22);
              Console.WriteLine(await Pending);
            end;
            end program;
            """);
        Assert.Equal("42\n", await program.Run());
        await using var dap = new NetCoreDbgSession(program);
        var entry = await dap.Launch(program, 8);
        await dap.Continue(entry.Thread);
        var stop = await dap.Stop();
        Assert.Equal(8, stop.Frame.GetProperty("line").GetInt32());
        var locals = await dap.Locals(stop.Frame);
        Assert.DoesNotContain(locals, local => local.GetProperty("name").GetString()!.StartsWith('<'));
        Value(locals, "Input", "19"); Value(locals, "Value", "20"); Value(locals, "Release", "22");
        await dap.Request("next", new { threadId = stop.Thread });
        stop = await dap.Stop("step");
        Assert.Equal(9, stop.Frame.GetProperty("line").GetInt32());
        locals = await dap.Locals(stop.Frame);
        Assert.DoesNotContain(locals, local => local.GetProperty("name").GetString()!.StartsWith('<'));
        Value(locals, "Value", "42");
        Assert.Equal("42\n", await dap.Finish(stop.Thread));
    }
}
