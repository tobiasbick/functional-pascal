namespace Fpas.IntegrationTests.Debugging;

public sealed partial class NetCoreDbgTests
{
    [NetCoreDbgFact]
    public async Task Escaped_generic_async_routine_retains_shared_captures_across_two_real_awaits()
    {
        using var program = new Debuggee("async-captures");
        await program.Compile("""
            program AsyncCaptures;
            uses System as Sys, System.Console as Console, System.Threading.Tasks as Tasks;
            type Job = record Run: function(): Tasks.Task<int>; Read: function(): int; end record;
            function Make<T>(Label: T; First: Tasks.Task<int>; Second: Tasks.Task<int>): Job;
            begin
              var Count: int := 0;
              function Read(): int;
              begin
                return Count;
              end function;
              async function Run<U>(Delta: U): Tasks.Task<int>;
              where U: struct;
              begin
                const A: int := await First;
                Count := Count + A;
                discard Label;
                discard Delta;
                const B: int := await Second;
                Count := Count + B;
                return Read();
              end function;
              function Start(): Tasks.Task<int>;
              begin return Run<int>(7); end function;
              return new Job(Start, Read);
            end function;
            begin
              const First: Tasks.TaskCompletionSource<int> := new Tasks.TaskCompletionSource<int>();
              const Second: Tasks.TaskCompletionSource<int> := new Tasks.TaskCompletionSource<int>();
              const Work: Job := Make<string>('marker', First.Task, Second.Task);
              const Pending: Tasks.Task<int> := Work.Run();
              if Pending.IsCompleted then raise new Sys.InvalidOperationException('First await must suspend'); end if;
              First.SetResult(20);
              if Pending.IsCompleted or Work.Read() <> 20 then
                raise new Sys.InvalidOperationException('Second await must suspend after the first continuation');
              end if;
              Second.SetResult(22);
              Console.WriteLine(Pending.GetAwaiter().GetResult());
              Console.WriteLine(Work.Read());
            end;
            end program;
            """);
        Assert.Equal("42\n42\n", await program.Run());
        var source = File.ReadAllLines(program.Source);
        var firstLine = Array.FindIndex(source, text => text.Contains("Count := Count + A", StringComparison.Ordinal)) + 1;
        var secondLine = Array.FindIndex(source, text => text.Contains("Count := Count + B", StringComparison.Ordinal)) + 1;
        var readLine = Array.FindIndex(source, text => text.Trim() == "return Count;") + 1;
        await using var dap = new NetCoreDbgSession(program);
        var entry = await dap.Launch(program, firstLine, secondLine);
        await dap.Continue(entry.Thread);
        var stop = await dap.Stop();
        Assert.Equal(firstLine, stop.Frame.GetProperty("line").GetInt32());
        var locals = await dap.Locals(stop.Frame);
        Assert.DoesNotContain(locals, local => local.GetProperty("name").GetString()!.StartsWith('<'));
        Value(locals, "A", "20"); Value(locals, "Delta", "7");
        await CheckAsyncCapture(dap, locals, "0");
        await dap.Request("next", new { threadId = stop.Thread });
        stop = await dap.Stop("step");
        Assert.Equal(firstLine + 1, stop.Frame.GetProperty("line").GetInt32());
        await CheckAsyncCapture(dap, await dap.Locals(stop.Frame), "20");
        await dap.Continue(stop.Thread);
        stop = await dap.Stop();
        Assert.Equal(secondLine, stop.Frame.GetProperty("line").GetInt32());
        locals = await dap.Locals(stop.Frame);
        Assert.DoesNotContain(locals, local => local.GetProperty("name").GetString()!.StartsWith('<'));
        Value(locals, "A", "20"); Value(locals, "B", "22"); Value(locals, "Delta", "7");
        await CheckAsyncCapture(dap, locals, "20");
        await dap.Request("next", new { threadId = stop.Thread });
        stop = await dap.Stop("step");
        Assert.Equal(secondLine + 1, stop.Frame.GetProperty("line").GetInt32());
        await CheckAsyncCapture(dap, await dap.Locals(stop.Frame), "42");
        await dap.Request("stepIn", new { threadId = stop.Thread });
        stop = await dap.Stop("step");
        Assert.Equal(readLine, stop.Frame.GetProperty("line").GetInt32());
        Assert.Contains("Read", stop.Frame.GetProperty("name").GetString());
        var receiver = Value(await dap.Locals(stop.Frame), "this");
        var fields = await dap.Variables(receiver.GetProperty("variablesReference").GetInt32());
        Value(await dap.Variables(Value(fields, "Count").GetProperty("variablesReference").GetInt32()), "Value", "42");
        Assert.Equal("42\n42\n", await dap.Finish(stop.Thread));
    }

    private static async Task CheckAsyncCapture(NetCoreDbgSession dap, System.Text.Json.JsonElement[] locals, string count)
    {
        var receiver = Value(locals, "this");
        var fields = await dap.Variables(receiver.GetProperty("variablesReference").GetInt32());
        Value(fields, "Label", "\"marker\"");
        Value(await dap.Variables(Value(fields, "Count").GetProperty("variablesReference").GetInt32()), "Value", count);
    }
}
