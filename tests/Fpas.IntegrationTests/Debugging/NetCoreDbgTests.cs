using System.Text.Json;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class NetCoreDbgTests
{
    [NetCoreDbgFact]
    public async Task Source_breakpoint_parameters_locals_next_and_normal_completion()
    {
        using var program = new Debuggee("sync");
        await program.Compile("""
            program Probe;
            uses System.Console as Console;
            function Calculate(Input: int): int;
            begin
              var Value: int := Input + 1;
              Value := Value * 2;
              if Value > 0 then
                Value := Value + 2;
              end if;
              return Value;
            end function;
            begin
              Console.WriteLine(Calculate(19));
            end;
            end program;
            """);
        Assert.Equal("42\n", await program.Run());
        await using var dap = new NetCoreDbgSession(program);
        var entry = await dap.Launch(program, 6);
        Assert.False(entry.Pending.GetProperty("breakpoints")[0].GetProperty("verified").GetBoolean());
        var bound = await dap.Breakpoints(program.Source, 6);
        Assert.True(bound.GetProperty("breakpoints")[0].GetProperty("verified").GetBoolean());
        Assert.Equal(6, bound.GetProperty("breakpoints")[0].GetProperty("line").GetInt32());
        var threads = await dap.Request("threads");
        Assert.Contains(threads.GetProperty("threads").EnumerateArray(), item => item.GetProperty("id").GetInt32() == entry.Thread);
        await dap.Continue(entry.Thread);
        var stop = await dap.Stop();
        Assert.Equal(6, stop.Frame.GetProperty("line").GetInt32());
        Assert.Equal(program.Source, stop.Frame.GetProperty("source").GetProperty("path").GetString());
        var locals = await dap.Locals(stop.Frame);
        Value(locals, "Input", "19"); Value(locals, "Value", "20");
        await dap.Request("next", new { threadId = stop.Thread });
        stop = await dap.Stop("step");
        Assert.Equal(7, stop.Frame.GetProperty("line").GetInt32());
        Value(await dap.Locals(stop.Frame), "Value", "40");
        Assert.Equal("42\n", await dap.Finish(stop.Thread));
    }

    [NetCoreDbgFact]
    public async Task Disconnect_terminates_the_owned_target_at_entry()
    {
        using var program = new Debuggee("disconnect");
        await program.Compile("program Stop; uses System.Console as C; begin C.WriteLine('not reached'); end; end program;");
        await using var dap = new NetCoreDbgSession(program);
        await dap.Launch(program);
        await dap.Disconnect();
    }

    private static JsonElement Value(JsonElement[] values, string name, string? expected = null)
    {
        var value = Assert.Single(values, item => item.GetProperty("name").GetString() == name);
        if (expected is not null) Assert.Equal(expected, value.GetProperty("value").GetString());
        return value;
    }
}
