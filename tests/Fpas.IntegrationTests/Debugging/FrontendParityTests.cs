using System.Text.Json;
using System.Text.Json.Nodes;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Debugging;

public sealed class FrontendParityTests
{
    [Fact]
    public async Task Batch_and_DAP_preserve_exact_values_and_independent_errors_at_the_same_source_stop()
    {
        using var program = new Debuggee("frontend-parity");
        await program.Compile("""
            program FrontendParity;
            uses System.Console as Console;
            begin
              const Value: long := 9007199254740993L;
              const Numbers: int[] := [3, 7];
              Console.WriteLine(Value);
            end;
            end program;
            """);
        var batch = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "debug", program.Assembly,
            "--break", program.Source + ":6", "--print", "Value", "--print", "Numbers[1] + 1", "--print", "Missing"], timeLimit: TimeSpan.FromSeconds(40));
        Assert.True(batch.ExitCode == 3, batch.Output + batch.Error); Assert.Empty(batch.Error);
        using var report = JsonDocument.Parse(batch.Output);
        Assert.Equal(6, report.RootElement.GetProperty("stop").GetProperty("source").GetProperty("line").GetInt32());
        var expressions = report.RootElement.GetProperty("expressions").EnumerateArray().ToArray();
        Assert.Equal("9007199254740993", expressions[0].GetProperty("value").GetProperty("scalar").GetString());
        Assert.Equal("8", expressions[1].GetProperty("value").GetProperty("scalar").GetString());
        Assert.True(expressions[2].TryGetProperty("error", out _));

        await using var dap = new DapProbe();
        await dap.Success("initialize"); await dap.Success("launch", new() { ["program"] = program.Assembly });
        await dap.Event("initialized");
        await dap.Success("setBreakpoints", new() { ["source"] = new JsonObject { ["path"] = program.Source },
            ["breakpoints"] = new JsonArray(new JsonObject { ["line"] = 6 }) });
        await dap.Success("configurationDone");
        var stopped = await dap.Event("stopped");
        var stack = await dap.Success("stackTrace", new() { ["threadId"] = stopped.GetProperty("threadId").GetInt32(), ["levels"] = 1 });
        Assert.Equal(6, stack.GetProperty("stackFrames")[0].GetProperty("line").GetInt32());
        foreach (var (expression, expected) in new[] { ("Value", "9007199254740993"), ("Numbers[1] + 1", "8") })
            Assert.Equal(expected, (await dap.Success("evaluate", new() { ["expression"] = expression })).GetProperty("result").GetString());
        Assert.False((await dap.Request("evaluate", new() { ["expression"] = "Missing" })).GetProperty("success").GetBoolean());
        Assert.Equal("9007199254740993", (await dap.Success("evaluate", new() { ["expression"] = "Value" })).GetProperty("result").GetString());
        await dap.Success("terminate"); await dap.End();
    }
}
