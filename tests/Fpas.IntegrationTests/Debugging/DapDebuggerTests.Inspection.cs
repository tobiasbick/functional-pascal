using System.Text.Json.Nodes;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class DapDebuggerTests
{
    [Fact]
    public async Task Explicit_pages_that_exceed_the_byte_budget_fail_and_remain_readable_in_smaller_pages()
    {
        using var program = new Debuggee("dap-page-bytes");
        await program.Compile("""
            program PageBytes;
            uses System as Sys, System.Console as Console;
            begin
              const Items: array of string? := new string?[128];
              const Text: string := new Sys.String(#120, 4096);
              for Index: int := 0 to 127 do Items[Index] := Text; end for;
              Console.WriteLine('observe');
            end;
            end program;
            """);
        await using var client = new DapProbe(); await client.Success("initialize", new() { ["supportsVariablePaging"] = true });
        await client.Success("launch", new() { ["program"] = program.Assembly }); await client.Event("initialized");
        await client.Success("setBreakpoints", new() { ["source"] = new JsonObject { ["path"] = program.Source }, ["breakpoints"] = new JsonArray(new JsonObject { ["line"] = 7 }) });
        await client.Success("configurationDone"); await client.Event("stopped");
        var value = await client.Success("evaluate", new() { ["expression"] = "Items" }); var reference = value.GetProperty("variablesReference").GetInt32();
        var tooLarge = await client.Request("variables", new() { ["variablesReference"] = reference, ["start"] = 0, ["count"] = 128 });
        Assert.False(tooLarge.GetProperty("success").GetBoolean()); Assert.Equal("limit_exceeded", tooLarge.GetProperty("message").GetString());
        var last = await client.Success("variables", new() { ["variablesReference"] = reference, ["start"] = 127, ["count"] = 1 });
        Assert.Equal(new string('x', 4096), last.GetProperty("variables")[0].GetProperty("value").GetString());
        await client.Success("disconnect"); await client.End();
    }

    [Fact]
    public async Task Null_is_explicit_and_unpaged_large_values_fail_instead_of_silently_losing_children()
    {
        using var program = new Debuggee("dap-paging-null");
        await program.Compile("""
            program Paging;
            uses System.Console as Console;
            begin
              const Missing: string? := null;
              const Numbers: array of int := new int[129];
              Numbers[128] := 73;
              Console.WriteLine('observe');
            end;
            end program;
            """);
        await using var client = new DapProbe();
        await client.Success("initialize", new() { ["supportsVariablePaging"] = true });
        await client.Success("launch", new() { ["program"] = program.Assembly }); await client.Event("initialized");
        await client.Success("setBreakpoints", new() { ["source"] = new JsonObject { ["path"] = program.Source }, ["breakpoints"] = new JsonArray(new JsonObject { ["line"] = 7 }) });
        await client.Success("configurationDone"); var stop = await client.Event("stopped");
        var stack = await client.Success("stackTrace", new() { ["threadId"] = stop.GetProperty("threadId").GetInt32() });
        var id = stack.GetProperty("stackFrames")[0].GetProperty("id").GetInt32();
        var missing = await client.Success("evaluate", new() { ["frameId"] = id, ["expression"] = "Missing" }); Assert.Equal("null", missing.GetProperty("result").GetString());
        var numbers = await client.Success("evaluate", new() { ["frameId"] = id, ["expression"] = "Numbers" });
        Assert.Equal(129, numbers.GetProperty("indexedVariables").GetInt32());
        var reference = numbers.GetProperty("variablesReference").GetInt32();
        Assert.False((await client.Request("variables", new() { ["variablesReference"] = reference })).GetProperty("success").GetBoolean());
        var last = await client.Success("variables", new() { ["variablesReference"] = reference, ["start"] = 128, ["count"] = 1 });
        Assert.Equal("73", last.GetProperty("variables")[0].GetProperty("value").GetString());
        var beyond = await client.Success("variables", new() { ["variablesReference"] = reference, ["start"] = 129, ["count"] = 1 }); Assert.Empty(beyond.GetProperty("variables").EnumerateArray());
        await client.Success("disconnect"); await client.End();
    }

    [Fact]
    public async Task Exception_info_and_unsupported_requests_preserve_the_live_exception_stop()
    {
        using var program = new Debuggee("dap-exception");
        await program.Compile("program Failure; uses System.InvalidOperationException as Failure; begin raise new Failure('DAP λ'); end; end program;");
        await using var client = new DapProbe();
        await client.Success("initialize"); await client.Success("launch", new() { ["program"] = program.Assembly }); await client.Event("initialized");
        await client.Success("configurationDone"); var stop = await client.Event("stopped");
        Assert.Equal("exception", stop.GetProperty("reason").GetString());
        Assert.False((await client.Request("setVariable")).GetProperty("success").GetBoolean());
        Assert.False((await client.Request("disconnect", new() { ["terminateDebuggee"] = false })).GetProperty("success").GetBoolean());
        var info = await client.Success("exceptionInfo", new() { ["threadId"] = stop.GetProperty("threadId").GetInt32() });
        Assert.Equal("System.InvalidOperationException", info.GetProperty("exceptionId").GetString()); Assert.Equal("DAP λ", info.GetProperty("description").GetString());
        Assert.Equal("unhandled", info.GetProperty("breakMode").GetString());
        await client.Success("disconnect"); await client.End();
    }
}
