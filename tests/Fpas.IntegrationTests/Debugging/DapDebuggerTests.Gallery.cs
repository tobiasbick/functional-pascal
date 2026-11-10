using System.Text.Json;
using System.Text.Json.Nodes;
using Fpas.IntegrationTests.Tui;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class DapDebuggerTests
{
    [Fact]
    public async Task Gallery_sheet_keeps_focus_in_the_private_terminal()
    {
        using var program = new Debuggee("gallery-tui");
        await TuiFixture.CompileExample(program, "gallery");

        await using var client = new DapProbe();
        await client.Success("initialize");
        await client.Success("launch", new() { ["program"] = program.Assembly, ["fpasTui"] = true });
        await client.Event("initialized");
        await client.Success("configurationDone");

        await client.Success("fpasTuiResize", new() { ["columns"] = 134, ["rows"] = 38 });
        await WaitFor(new() { ["columns"] = 134, ["rows"] = 38, ["control"] = "pages/buttons" });
        await Send(new() { ["kind"] = "activate", ["control"] = "pages/buttons" });
        Assert.Equal("presented", await Send(new() { ["kind"] = "focus", ["control"] = "copy-install" }));
        var focused = (await client.Success("fpasTuiSnapshot")).GetProperty("frame");
        await client.Success("fpasTuiTerminalInput", new() { ["bytes"] = Convert.ToBase64String("\r"u8) });
        await WaitFor(new() { ["after"] = focused.GetProperty("id").GetInt64(), ["text"] = "✓" });
        var copied = (await client.Success("fpasTuiSnapshot")).GetProperty("frame");
        Assert.Equal("copied", Control(copied, "copy-install").GetProperty("value").GetString());
        Assert.True(Control(copied, "copy-install").GetProperty("focused").GetBoolean());

        await Send(new() { ["kind"] = "activate", ["control"] = "pages/overlays" });
        await Send(new() { ["kind"] = "focus", ["control"] = "sheet-open" });
        await Send(new() { ["kind"] = "activate", ["control"] = "sheet-open" });
        var sheet = (await client.Success("fpasTuiSnapshot")).GetProperty("frame");
        Assert.True(Control(sheet, "sheet/close").GetProperty("focused").GetBoolean());
        Assert.False(Control(sheet, "sheet-open").GetProperty("enabled").GetBoolean());
        Assert.Equal("unchanged", await Send(new() { ["kind"] = "activate", ["control"] = "pages/buttons" }));

        await client.Success("fpasTuiResize", new() { ["columns"] = 40, ["rows"] = 17 });
        await WaitFor(new() { ["columns"] = 40, ["rows"] = 17, ["control"] = "drawer-open" });
        var resized = (await client.Success("fpasTuiSnapshot")).GetProperty("frame");
        Assert.True(Control(resized, "sheet/close").GetProperty("focused").GetBoolean());

        await client.Success("fpasTuiTerminalInput", new() { ["bytes"] = Convert.ToBase64String("\r"u8) });
        await WaitFor(new() { ["after"] = resized.GetProperty("id").GetInt64() });
        var closed = (await client.Success("fpasTuiSnapshot")).GetProperty("frame");
        Assert.True(Control(closed, "sheet-open").GetProperty("focused").GetBoolean());
        Assert.DoesNotContain(closed.GetProperty("controls").EnumerateArray(),
            item => item.GetProperty("id").GetString() == "sheet/close");

        await client.Success("fpasTuiTerminalInput",
            new() { ["bytes"] = Convert.ToBase64String(new byte[] { 17 }) });
        Assert.Equal(0, (await client.Event("exited")).GetProperty("exitCode").GetInt32());
        await client.Event("terminated");
        await client.Success("disconnect");
        await client.End();

        async Task WaitFor(JsonObject condition) =>
            Assert.Equal("matched", (await client.Success("fpasTuiWait", new()
            {
                ["frame"] = condition
            })).GetProperty("status").GetString());

        async Task<string?> Send(JsonObject input)
        {
            var queued = await client.Success("fpasTuiSend", new() { ["input"] = input });
            Assert.Equal("queued", queued.GetProperty("status").GetString());
            var result = await client.Success("fpasTuiWait", new() { ["inputId"] = queued.GetProperty("inputId").GetInt64() });
            return result.GetProperty("status").GetString();
        }

        static JsonElement Control(JsonElement frame, string id) =>
            Assert.Single(frame.GetProperty("controls").EnumerateArray(), item => item.GetProperty("id").GetString() == id);
    }
}
