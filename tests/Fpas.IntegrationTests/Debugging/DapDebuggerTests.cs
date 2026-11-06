using System.Text.Json.Nodes;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class DapDebuggerTests
{
    [Fact]
    public async Task Independent_client_configures_inspects_steps_pauses_and_terminates_through_real_stdio()
    {
        using var program = new Debuggee("dap-λ space");
        await program.Compile("""
            program DapTarget;
            uses System.Console as Console, System.Threading.Thread as Thread;
            procedure Observe();
            begin
              const Value: int := 42;
              const Numbers: int[] := [3, 7];
              Console.WriteLine('capture');
              Console.WriteLine(Value);
            end procedure;
            begin
              Observe();
              while true do Thread.Sleep(100); end while;
            end;
            end program;
            """);
        await using var client = new DapProbe();
        var caps = await client.Success("initialize", new() { ["linesStartAt1"] = false, ["columnsStartAt1"] = false, ["pathFormat"] = "uri", ["supportsVariablePaging"] = true }, split: true);
        Assert.True(caps.GetProperty("supportsConfigurationDoneRequest").GetBoolean());
        Assert.False(caps.TryGetProperty("supportsSetVariable", out _));
        await client.Success("launch", new() { ["program"] = new Uri(program.Assembly).AbsoluteUri, ["cwd"] = new Uri(program.DirectoryPath).AbsoluteUri });
        await client.Event("initialized");
        var points = await client.Success("setBreakpoints", new() { ["source"] = new JsonObject { ["path"] = new Uri(program.Source).AbsoluteUri },
            ["breakpoints"] = new JsonArray(new JsonObject { ["line"] = 6 }) });
        Assert.True(points.GetProperty("breakpoints")[0].GetProperty("verified").GetBoolean());
        await client.Success("setExceptionBreakpoints", new() { ["filters"] = new JsonArray("unhandled") });
        await client.Success("configurationDone");
        var stopped = await client.Event("stopped"); Assert.Equal("breakpoint", stopped.GetProperty("reason").GetString());
        var thread = stopped.GetProperty("threadId").GetInt32();
        Assert.NotEmpty((await client.Success("threads")).GetProperty("threads").EnumerateArray());
        var stack = await client.Success("stackTrace", new() { ["threadId"] = thread, ["levels"] = 4 });
        var frame = stack.GetProperty("stackFrames")[0]; Assert.Equal(6, frame.GetProperty("line").GetInt32());
        Assert.Equal(new Uri(program.Source).AbsoluteUri, frame.GetProperty("source").GetProperty("path").GetString());
        var id = frame.GetProperty("id").GetInt32();
        var scopes = await client.Success("scopes", new() { ["frameId"] = id });
        var variables = await client.Success("variables", new() { ["variablesReference"] = scopes.GetProperty("scopes")[0].GetProperty("variablesReference").GetInt32(), ["count"] = 64 });
        var value = Assert.Single(variables.GetProperty("variables").EnumerateArray(), item => item.GetProperty("name").GetString() == "Value"); Assert.Equal("42", value.GetProperty("value").GetString());
        var array = Assert.Single(variables.GetProperty("variables").EnumerateArray(), item => item.GetProperty("name").GetString() == "Numbers");
        var elements = await client.Success("variables", new() { ["variablesReference"] = array.GetProperty("variablesReference").GetInt32(), ["start"] = 1, ["count"] = 1 });
        Assert.Equal("7", elements.GetProperty("variables")[0].GetProperty("value").GetString());
        var evaluated = await client.Success("evaluate", new() { ["frameId"] = id, ["expression"] = "Numbers[1] + Value", ["context"] = "watch" });
        Assert.Equal("49", evaluated.GetProperty("result").GetString());
        var refused = await client.Request("evaluate", new() { ["frameId"] = id, ["expression"] = "Console.WriteLine('bad')" }); Assert.False(refused.GetProperty("success").GetBoolean());
        await client.Success("next", new() { ["threadId"] = thread });
        Assert.Equal(thread, (await client.Event("continued")).GetProperty("threadId").GetInt32()); await client.Event("stopped");
        Assert.False((await client.Request("scopes", new() { ["frameId"] = id })).GetProperty("success").GetBoolean());
        await client.Success("stepOut", new() { ["threadId"] = thread }); await client.Event("stopped");
        await client.Success("continue", new() { ["threadId"] = thread });
        await client.Success("pause", new() { ["threadId"] = thread }); await client.Event("stopped");
        await client.Success("cancel", new() { ["requestId"] = 999999 });
        await client.Success("terminate"); await client.Event("terminated"); await client.End();
    }

    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task Entry_configuration_and_natural_exit_emit_independent_terminal_facts(bool withoutHostSearch)
    {
        using var program = new Debuggee("dap-entry-exit");
        await program.Compile("program Entry; uses System.Console as Console; begin Console.WriteLine('done'); end; end program;");
        await using var client = new DapProbe(withoutHostSearch: withoutHostSearch);
        await client.Success("initialize"); await client.Success("launch", new() { ["program"] = program.Assembly, ["stopAtEntry"] = true });
        await client.Event("initialized"); await client.Success("configurationDone");
        var entry = await client.Event("stopped"); Assert.Equal("entry", entry.GetProperty("reason").GetString());
        await client.Success("continue", new() { ["threadId"] = entry.GetProperty("threadId").GetInt32() });
        Assert.Equal(0, (await client.Event("exited")).GetProperty("exitCode").GetInt32()); await client.Event("terminated");
        await client.Success("disconnect"); await client.End();
    }
}
