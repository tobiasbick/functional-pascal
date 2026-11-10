using System.Text.Json;
using System.Text.Json.Nodes;
using Fpas.IntegrationTests.Tui;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class DapDebuggerTests
{
    [Fact]
    public async Task Tui_controller_keeps_committed_frame_at_breakpoint_and_observes_resized_frame_after_continue()
    {
        using var program = new Debuggee("dap-tui");
        var source = await TuiFixture.Compile(program, "example.fpas");
        var line = Array.FindIndex(source.Split('\n'), row => row.Contains("var Next: Model := State;", StringComparison.Ordinal)) + 1;
        Assert.True(line > 0);

        await using var client = new DapProbe();
        await client.Success("initialize");
        await client.Success("launch", new() { ["program"] = program.Assembly, ["fpasTui"] = true });
        await client.Event("initialized");
        var breakpointSource = new JsonObject { ["path"] = program.Source };
        await client.Success("setBreakpoints", new()
        {
            ["source"] = breakpointSource.DeepClone(),
            ["breakpoints"] = new JsonArray(new JsonObject { ["line"] = line })
        });
        await client.Success("configurationDone");
        Assert.Equal("matched", (await client.Success("fpasTuiWait", new()
        {
            ["frame"] = new JsonObject { ["columns"] = 80, ["rows"] = 24, ["control"] = "items/Amber" }
        })).GetProperty("status").GetString());
        var initial = (await client.Success("fpasTuiSnapshot")).GetProperty("frame");
        var input = await client.Success("fpasTuiSend", new()
        {
            ["input"] = new JsonObject { ["kind"] = "paste", ["text"] = "bir" }
        });
        Assert.Equal("queued", input.GetProperty("status").GetString());
        var stop = await client.Event("stopped");
        Assert.Equal("breakpoint", stop.GetProperty("reason").GetString());
        await client.Success("fpasTuiResize", new() { ["columns"] = 53, ["rows"] = 17 });
        var stopped = await client.Success("fpasTuiSnapshot");
        Assert.True(stopped.GetProperty("paused").GetBoolean());
        Assert.Equal(initial.GetProperty("id").GetInt64(), stopped.GetProperty("frame").GetProperty("id").GetInt64());
        Assert.Equal(80, stopped.GetProperty("frame").GetProperty("columns").GetInt32());
        var inputId = input.GetProperty("inputId").GetInt64();
        var paused = await client.Success("fpasTuiWait", new() { ["inputId"] = inputId, ["timeoutMs"] = 50 });
        Assert.Equal("paused", paused.GetProperty("status").GetString());
        await client.Success("setBreakpoints", new() { ["source"] = breakpointSource, ["breakpoints"] = new JsonArray() });
        await client.Success("continue", new() { ["threadId"] = stop.GetProperty("threadId").GetInt32() });
        Assert.Equal("matched", (await client.Success("fpasTuiWait", new()
        {
            ["frame"] = new JsonObject { ["columns"] = 53, ["rows"] = 17, ["text"] = "bir", ["control"] = "items/Birch" }
        })).GetProperty("status").GetString());
        var processed = await client.Success("fpasTuiWait", new() { ["inputId"] = inputId });
        Assert.Equal("presented", processed.GetProperty("status").GetString());
        Assert.Equal(inputId, processed.GetProperty("inputId").GetInt64());
        var resized = await client.Success("fpasTuiSnapshot");
        Assert.DoesNotContain(resized.GetProperty("frame").GetProperty("controls").EnumerateArray(), control => control.GetProperty("id").GetString() == "items/Amber");
        Assert.Equal(17, resized.GetProperty("terminalLines").GetArrayLength());
        using var terminalDeadline = new CancellationTokenSource(TimeSpan.FromSeconds(3));
        while (!ShowsStyledResults(resized))
        {
            await Task.Delay(20, terminalDeadline.Token);
            resized = await client.Success("fpasTuiSnapshot");
        }

        await client.Success("fpasTuiTerminalInput", new()
        {
            ["bytes"] = Convert.ToBase64String("\u0001\u001b[200~ce\u001b[201~"u8)
        });
        Assert.Equal("matched", (await client.Success("fpasTuiWait", new()
        {
            ["frame"] = new JsonObject { ["text"] = "ce", ["control"] = "items/Cedar" }
        })).GetProperty("status").GetString());

        await client.Success("fpasTuiSend", new() { ["input"] = new JsonObject { ["kind"] = "key", ["key"] = "O", ["modifiers"] = 4 } });
        Assert.Equal("matched", (await client.Success("fpasTuiWait", new()
        {
            ["frame"] = new JsonObject { ["control"] = "theme-blue" }
        })).GetProperty("status").GetString());
        await client.Success("fpasTuiSend", new()
        {
            ["input"] = new JsonObject { ["kind"] = "focus", ["control"] = "theme-blue" }
        });
        var theme = await client.Success("fpasTuiSend", new()
        {
            ["input"] = new JsonObject { ["kind"] = "activate", ["control"] = "theme-blue" }
        });
        Assert.Equal("presented", (await client.Success("fpasTuiWait", new()
        {
            ["inputId"] = theme.GetProperty("inputId").GetInt64()
        })).GetProperty("status").GetString());
        var themed = (await client.Success("fpasTuiSnapshot", new() { ["cells"] = true })).GetProperty("frame");
        Assert.Equal(0x121212, themed.GetProperty("cells")[0].GetProperty("colors").GetProperty("background").GetInt32());
        var selectedTheme = Assert.Single(themed.GetProperty("controls").EnumerateArray(), control =>
            control.GetProperty("id").GetString() == "theme-blue");
        Assert.True(selectedTheme.GetProperty("focused").GetBoolean());
        var themeCell = selectedTheme.GetProperty("row").GetInt32() * themed.GetProperty("columns").GetInt32()
            + selectedTheme.GetProperty("column").GetInt32();
        var themeColors = themed.GetProperty("cells")[themeCell].GetProperty("colors");
        Assert.Equal(0x142847, themeColors.GetProperty("foreground").GetInt32());
        Assert.Equal(0xF2F2F2, themeColors.GetProperty("background").GetInt32());
        await client.Success("fpasTuiSend", new() { ["input"] = new JsonObject { ["kind"] = "key", ["key"] = "Escape" } });
        await client.Success("fpasTuiSend", new() { ["input"] = new JsonObject { ["kind"] = "activate", ["control"] = "quit" } });
        Assert.Equal(0, (await client.Event("exited")).GetProperty("exitCode").GetInt32());
        await client.Event("terminated");
        await client.Success("disconnect");
        await client.End();
    }

    private static bool ShowsStyledResults(JsonElement snapshot)
    {
        var lines = snapshot.GetProperty("terminalLines").EnumerateArray()
            .Select(row => row.GetString()!).ToArray();
        return lines.Any(row => row.Contains("Birch", StringComparison.Ordinal))
            && lines.Any(row => row.Contains("╭ Results ─", StringComparison.Ordinal));
    }
}
