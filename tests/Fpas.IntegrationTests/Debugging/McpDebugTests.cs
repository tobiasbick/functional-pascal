using Fpas.Tests.Support;
using ModelContextProtocol.Protocol;
using System.Text.Json;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class McpDebugTests
{
    [Fact]
    public async Task Missing_backend_and_help_do_not_start_a_protocol_session()
    {
        var help = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "debug", "--help"]);
        Assert.Equal(0, help.ExitCode); Assert.Contains("--netcoredbg", help.Output);
        var failed = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "debug", "--netcoredbg", "missing"]);
        Assert.NotEqual(0, failed.ExitCode); Assert.Equal("", failed.Output); Assert.Contains("absolute path", failed.Error);
    }

    [NetCoreDbgFact]
    public async Task Official_MCP_client_discovers_tools_and_debugs_source_with_live_values()
    {
        using var program = new Debuggee("mcp-sync");
        await program.Compile("""
            program McpProbe;
            uses System.Console as Console;
            function Double(Input: int): int;
            begin
              var Value: int := Input;
              Value := Value * 2;
              return Value;
            end function;
            begin Console.WriteLine(Double(21)); end;
            end program;
            """);
        await using var probe = await McpProbe.Create();
        var tools = await probe.Client.ListToolsAsync(cancellationToken: probe.Token);
        Assert.Equal(12, tools.Count);
        Assert.Contains(tools, tool => tool.Name == "debug_launch");
        var schema = tools.Single(tool => tool.Name == "debug_launch").ProtocolTool.OutputSchema;
        Assert.True(schema.HasValue);
        Assert.True(schema.Value.GetProperty("anyOf")[0].GetProperty("properties").TryGetProperty("sessionId", out _));
        var entry = await probe.Call("debug_launch", new { program = program.Assembly, cwd = program.DirectoryPath });
        Assert.Equal("stopped", entry.GetProperty("state").GetString());
        var id = entry.GetProperty("sessionId").GetString()!;
        var textOnly = await probe.Client.CallToolAsync("debug_status", new Dictionary<string, object?> { ["sessionId"] = id }, cancellationToken: probe.Token);
        using var textResult = JsonDocument.Parse(Assert.Single(textOnly.Content.OfType<TextContentBlock>()).Text);
        Assert.True(JsonElement.DeepEquals(textResult.RootElement, textOnly.StructuredContent!.Value));
        await probe.Call("debug_set_breakpoints", new { sessionId = id, source = program.Source, lines = new[] { 6 } });
        await probe.Call("debug_continue", new { sessionId = id, stopId = entry.GetProperty("stopId").GetInt64() });
        var stop = await probe.Stopped(id, entry.GetProperty("stopId").GetInt64());
        var variables = await probe.Locals(id, stop);
        Assert.Equal("21", variables.Single(value => value.GetProperty("name").GetString() == "Value").GetProperty("value").GetString());
        var firstStack = await probe.Call("debug_stack", new { sessionId = id, stopId = stop.GetProperty("stopId").GetInt64() });
        var oldFrame = firstStack.GetProperty("data").GetProperty("frames")[0].GetProperty("handle").GetString();
        var firstScopes = await probe.Call("debug_variables", new { sessionId = id, stopId = stop.GetProperty("stopId").GetInt64(), frameHandle = oldFrame });
        var oldScope = firstScopes.GetProperty("data").GetProperty("scopes")[0].GetProperty("handle").GetString();
        await probe.Call("debug_step", new { sessionId = id, stopId = stop.GetProperty("stopId").GetInt64(), kind = "over" });
        var next = await probe.Stopped(id, stop.GetProperty("stopId").GetInt64());
        variables = await probe.Locals(id, next);
        Assert.Equal("42", variables.Single(value => value.GetProperty("name").GetString() == "Value").GetProperty("value").GetString());
        await probe.Call("debug_evaluate", new { sessionId = id, stopId = next.GetProperty("stopId").GetInt64(), frameHandle = oldFrame, expression = "Value" }, "stale_reference");
        await probe.Call("debug_variables", new { sessionId = id, stopId = next.GetProperty("stopId").GetInt64(), variablesHandle = oldScope }, "stale_reference");
        await probe.Call("debug_stack", new { sessionId = id, stopId = stop.GetProperty("stopId").GetInt64() }, "stale_reference");
        await probe.Call("debug_continue", new { sessionId = id, stopId = next.GetProperty("stopId").GetInt64() });
        var ended = await probe.State(id, "exited");
        Assert.Equal(0, ended.GetProperty("exitCode").GetInt32());
        var output = await probe.Call("debug_output", new { sessionId = id });
        Assert.Contains("42", string.Concat(output.GetProperty("data").GetProperty("items").EnumerateArray().Select(item => item.GetProperty("text").GetString())));
        await probe.Call("debug_stop", new { sessionId = id });
        await probe.Call("debug_stop", new { sessionId = id });
    }
}
