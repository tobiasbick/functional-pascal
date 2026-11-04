using System.Text.Json;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class McpDebugTests
{
    [NetCoreDbgFact]
    public async Task Real_async_continuation_exposes_hoisted_values_and_evaluation()
    {
        using var program = new Debuggee("mcp-async");
        await program.Compile("""
            async program Probe;
            uses System.Console as C, System.Threading.Tasks as T;
            async function Work(Input: int; Gate: T.Task<int>): T.Task<int>;
            begin
              var Value: int := Input + 1;
              const Release: int := await Gate;
              Value := Value + Release;
              return Value;
            end function;
            begin
              const Gate: T.TaskCompletionSource<int> := new T.TaskCompletionSource<int>(T.TaskCreationOptions.RunContinuationsAsynchronously);
              const Pending: T.Task<int> := Work(19, Gate.Task);
              Gate.SetResult(22);
              C.WriteLine(await Pending);
            end;
            end program;
            """);
        await using var probe = await McpProbe.Create();
        var entry = await probe.Call("debug_launch", new { program = program.Assembly, cwd = program.DirectoryPath });
        var id = entry.GetProperty("sessionId").GetString()!;
        await probe.Call("debug_set_breakpoints", new { sessionId = id, source = program.Source, lines = new[] { 7 } });
        await probe.Continue(id, entry);
        var stop = await probe.Stopped(id, entry.GetProperty("stopId").GetInt64());
        var locals = await probe.Locals(id, stop);
        AssertValue(locals, "Input", "19"); AssertValue(locals, "Value", "20"); AssertValue(locals, "Release", "22");
        var stopId = stop.GetProperty("stopId").GetInt64();
        var stack = await probe.Call("debug_stack", new { sessionId = id, stopId });
        var frameHandle = stack.GetProperty("data").GetProperty("frames")[0].GetProperty("handle").GetString();
        var evaluated = await probe.Call("debug_evaluate", new { sessionId = id, stopId, frameHandle, expression = "Value + Release" });
        Assert.Equal("42", evaluated.GetProperty("data").GetProperty("value").GetString());
        await probe.Call("debug_step", new { sessionId = id, stopId, kind = "over" });
        var next = await probe.Stopped(id, stopId);
        AssertValue(await probe.Locals(id, next), "Value", "42");
        await probe.Continue(id, next); await probe.State(id, "exited");
    }

    [NetCoreDbgFact]
    public async Task Captures_generic_values_and_step_into_out_use_current_source_storage()
    {
        using var program = new Debuggee("mcp-captures");
        await program.Compile("""
            program Probe;
            uses System.Console as C;
            type Box<T> = record Item: T; end record;
            function Make(Start: int): function(): int;
            begin
              var Count: int := Start;
              function Next(): int;
              begin
                Count := Count + 2;
                return Count;
              end function;
              return Next;
            end function;
            begin
              const Counter: function(): int := Make(38);
              const Wrapped: Box<string> := new Box<string>('marker');
              C.WriteLine(Counter());
              C.WriteLine(Counter());
            end;
            end program;
            """);
        await using var probe = await McpProbe.Create();
        var entry = await probe.Call("debug_launch", new { program = program.Assembly, cwd = program.DirectoryPath });
        var id = entry.GetProperty("sessionId").GetString()!;
        await probe.Call("debug_set_breakpoints", new { sessionId = id, source = program.Source, lines = new[] { 17 } });
        await probe.Continue(id, entry);
        var stop = await probe.Stopped(id, entry.GetProperty("stopId").GetInt64());
        var locals = await probe.Locals(id, stop);
        var fields = await probe.Expand(id, stop, locals.Single(value => value.GetProperty("name").GetString() == "Wrapped"));
        AssertValue(fields, "Item", "\"marker\"");
        await probe.Call("debug_step", new { sessionId = id, stopId = stop.GetProperty("stopId").GetInt64(), kind = "into" });
        var inside = await probe.Stopped(id, stop.GetProperty("stopId").GetInt64());
        var captured = await probe.Locals(id, inside);
        var receiver = captured.Single(value => value.GetProperty("name").GetString() == "this");
        fields = await probe.Expand(id, inside, receiver);
        var cell = fields.Single(value => value.GetProperty("name").GetString() == "Count");
        AssertValue(await probe.Expand(id, inside, cell), "Value", "38");
        await probe.Call("debug_step", new { sessionId = id, stopId = inside.GetProperty("stopId").GetInt64(), kind = "out" });
        var outside = await probe.Stopped(id, inside.GetProperty("stopId").GetInt64());
        await probe.Call("debug_set_breakpoints", new { sessionId = id, source = program.Source, lines = new[] { 9 } });
        await probe.Continue(id, outside);
        var second = await probe.Stopped(id, outside.GetProperty("stopId").GetInt64());
        fields = await probe.Expand(id, second, (await probe.Locals(id, second)).Single(value => value.GetProperty("name").GetString() == "this"));
        AssertValue(await probe.Expand(id, second, fields.Single(value => value.GetProperty("name").GetString() == "Count")), "Value", "40");
    }

    [NetCoreDbgFact]
    public async Task Exceptions_are_reported_with_backend_details()
    {
        using var program = new Debuggee("mcp-exception");
        await program.Compile("""
            program Fail;
            uses System as S;
            begin raise new S.InvalidOperationException('mcp failure'); end;
            end program;
            """);
        await using var probe = await McpProbe.Create();
        var entry = await probe.Call("debug_launch", new { program = program.Assembly, cwd = program.DirectoryPath });
        var id = entry.GetProperty("sessionId").GetString()!;
        await probe.Continue(id, entry);
        var stop = await probe.Stopped(id, entry.GetProperty("stopId").GetInt64());
        Assert.Equal("exception", stop.GetProperty("reason").GetString());
        Assert.Contains("InvalidOperationException", stop.GetProperty("exception").GetProperty("id").GetString());
    }

    private static void AssertValue(JsonElement[] values, string name, string expected)
        => Assert.Equal(expected, values.Single(value => value.GetProperty("name").GetString() == name).GetProperty("value").GetString());
}
