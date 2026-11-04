namespace Fpas.IntegrationTests.Debugging;

public sealed partial class NetCoreDbgTests
{
    [NetCoreDbgFact]
    public async Task Async_block_locals_have_distinct_names_and_end_at_their_lexical_scope()
    {
        using var program = new Debuggee("async-scopes");
        await program.Compile("""
            program AsyncScopes;
            uses System as Sys, System.Console as Console, System.Threading.Tasks as Tasks;
            async function Work(First: Tasks.Task<int>; Second: Tasks.Task<int>): Tasks.Task<int>;
            begin
              const Value: int := 40;
              const Value_1: int := 99;
              begin
                const Value: int := 2;
                const Inner: int := await First;
                Console.WriteLine(Value + Inner);
              end;
              const Final: int := await Second;
              return Value + Final + Value_1 - 99;
            end function;
            begin
              const First: Tasks.TaskCompletionSource<int> := new Tasks.TaskCompletionSource<int>();
              const Second: Tasks.TaskCompletionSource<int> := new Tasks.TaskCompletionSource<int>();
              const Pending: Tasks.Task<int> := Work(First.Task, Second.Task);
              if Pending.IsCompleted then raise new Sys.InvalidOperationException('First await must suspend'); end if;
              First.SetResult(20);
              if Pending.IsCompleted then raise new Sys.InvalidOperationException('Second await must suspend'); end if;
              Second.SetResult(2);
              Console.WriteLine(Pending.GetAwaiter().GetResult());
            end;
            end program;
            """);
        Assert.Equal("22\n42\n", await program.Run());
        await using var dap = new NetCoreDbgSession(program);
        var entry = await dap.Launch(program, 10, 13);
        await dap.Continue(entry.Thread);
        var stop = await dap.Stop();
        Assert.Equal(10, stop.Frame.GetProperty("line").GetInt32());
        var locals = await dap.Locals(stop.Frame);
        Value(locals, "Value", "40"); Value(locals, "Value_1", "99"); Value(locals, "Value_2", "2"); Value(locals, "Inner", "20");
        await dap.Continue(stop.Thread);
        stop = await dap.Stop();
        Assert.Equal(13, stop.Frame.GetProperty("line").GetInt32());
        locals = await dap.Locals(stop.Frame);
        Value(locals, "Value", "40"); Value(locals, "Value_1", "99"); Value(locals, "Final", "2");
        Assert.DoesNotContain(locals, local => local.GetProperty("name").GetString() is "Value_2" or "Inner");
        Assert.Equal("22\n42\n", await dap.Finish(stop.Thread));
    }
}
