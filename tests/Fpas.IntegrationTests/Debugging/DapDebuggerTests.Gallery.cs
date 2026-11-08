using System.Text.Json;
using System.Text.Json.Nodes;
using Fpas.IntegrationTests.Tui;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class DapDebuggerTests
{
    [Fact]
    public async Task Gallery_overlay_keeps_focus_through_terminal_resize()
    {
        using var program = new Debuggee("gallery-tui");
        await TuiFixture.CompileExample(program, "gallery");

        await using var client = new DapProbe();
        await client.Success("initialize");
        await client.Success("launch", new() { ["program"] = program.Assembly, ["fpasTui"] = true });
        await client.Event("initialized");
        await client.Success("configurationDone");

        await WaitFor(new() { ["control"] = "pages/overlay" });
        await Send(new() { ["kind"] = "activate", ["control"] = "pages/overlay" });
        await Send(new() { ["kind"] = "activate", ["control"] = "overlay-open" });
        var modal = (await client.Success("fpasTuiSnapshot")).GetProperty("frame");
        Assert.True(Control(modal, "overlay-close").GetProperty("focused").GetBoolean());
        Assert.False(Control(modal, "overlay-open").GetProperty("enabled").GetBoolean());
        Assert.Equal("unchanged", await Send(new() { ["kind"] = "key", ["key"] = "Q", ["modifiers"] = 4 }));

        await client.Success("fpasTuiResize", new() { ["columns"] = 53, ["rows"] = 17 });
        await WaitFor(new() { ["columns"] = 53, ["rows"] = 17, ["control"] = "overlay-close" });
        var resized = (await client.Success("fpasTuiSnapshot")).GetProperty("frame");
        Assert.True(Control(resized, "overlay-close").GetProperty("focused").GetBoolean());

        await Send(new() { ["kind"] = "key", ["key"] = "Escape" });
        var closed = (await client.Success("fpasTuiSnapshot")).GetProperty("frame");
        Assert.True(Control(closed, "overlay-open").GetProperty("focused").GetBoolean());
        Assert.DoesNotContain(closed.GetProperty("controls").EnumerateArray(),
            item => item.GetProperty("id").GetString() == "overlay-close");

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
