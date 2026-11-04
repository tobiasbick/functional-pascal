using System.Text.Json;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class McpDebugTests
{
    [NetCoreDbgFact]
    public async Task Paged_values_and_invalid_references_preserve_a_usable_session()
    {
        using var program = new Debuggee("mcp-pages");
        await program.Compile("""
            program Page;
            uses System.Console as C;
            begin
              const Values: int[] := [10, 20, 30];
              C.WriteLine(Values[0]);
            end;
            end program;
            """);
        await using var probe = await McpProbe.Create();
        var entry = await probe.Call("debug_launch", new { program = program.Assembly, cwd = program.DirectoryPath });
        var id = entry.GetProperty("sessionId").GetString()!;
        await probe.Call("debug_set_breakpoints", new { sessionId = id, source = program.Source, lines = new[] { 5 } });
        await probe.Continue(id, entry);
        var stop = await probe.Stopped(id, entry.GetProperty("stopId").GetInt64());
        var stopId = stop.GetProperty("stopId").GetInt64();
        var array = (await probe.Locals(id, stop)).Single(value => value.GetProperty("name").GetString() == "Values");
        var variablesHandle = array.GetProperty("handle").GetString();
        var page = await probe.Call("debug_variables", new { sessionId = id, stopId, variablesHandle, start = 1, count = 1 });
        Assert.True(page.GetProperty("data").GetProperty("truncated").GetBoolean());
        var value = Assert.Single(page.GetProperty("data").GetProperty("variables").EnumerateArray());
        Assert.Equal("[1]", value.GetProperty("name").GetString()); Assert.Equal("20", value.GetProperty("value").GetString());
        await probe.Call("debug_variables", new { sessionId = id, stopId, variablesHandle = "v:999999" }, "stale_reference");
        await probe.Call("debug_stack", new { sessionId = id, stopId, count = 129 }, "invalid_argument");
        await probe.Call("debug_stack", new { sessionId = id, stopId, threadId = 0 }, "invalid_argument");
        await probe.Call("debug_variables", new { sessionId = id, stopId }, "invalid_argument");
        await probe.Call("debug_step", new { sessionId = id, stopId, kind = "invalid" }, "invalid_argument");
        await probe.Call("debug_output", new { sessionId = id, afterSequence = long.MaxValue }, "invalid_argument");
        var stack = await probe.Call("debug_stack", new { sessionId = id, stopId });
        var frameHandle = stack.GetProperty("data").GetProperty("frames")[0].GetProperty("handle").GetString();
        await probe.Call("debug_evaluate", new { sessionId = id, stopId, frameHandle, expression = "NoSuchValue" }, "backend_error");
        await probe.Continue(id, stop); await probe.State(id, "exited");
    }

    [NetCoreDbgFact]
    public async Task Failed_launch_cleans_up_and_allows_a_fresh_launch()
    {
        using var program = new Debuggee("mcp-failed-launch");
        await program.Compile("program Probe; begin end; end program;");
        var runtime = Path.ChangeExtension(program.Assembly, ".runtimeconfig.json");
        var original = File.ReadAllText(runtime);
        File.WriteAllText(runtime, "invalid runtime configuration");
        await using var probe = await McpProbe.Create();
        await probe.Call("debug_launch", new { program = program.Assembly, cwd = program.DirectoryPath }, "backend_error");
        File.WriteAllText(runtime, original);
        var entry = await probe.Call("debug_launch", new { program = program.Assembly, cwd = program.DirectoryPath });
        Assert.Equal("stopped", entry.GetProperty("state").GetString());
    }
}
