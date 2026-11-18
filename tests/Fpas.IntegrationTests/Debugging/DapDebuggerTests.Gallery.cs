using System.Text.Json.Nodes;
using Fpas.IntegrationTests.Tui;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class DapDebuggerTests
{
    [Fact]
    public async Task Gallery_resizes_and_switches_theme_in_the_private_terminal()
    {
        using var program = new Debuggee("gallery-tui");
        await TuiFixture.CompileExample(program, "gallery");

        await using var client = new DapProbe();
        await client.Success("initialize");
        await client.Success("launch", new() { ["program"] = program.Assembly, ["fpasTui"] = true });
        await client.Event("initialized");
        await client.Success("configurationDone");

        await client.Success("fpasTuiResize", new() { ["columns"] = 134, ["rows"] = 38 });
        await WaitFor(new() { ["columns"] = 134, ["rows"] = 38, ["text"] = "╭─" });
        await client.Success("fpasTuiResize", new() { ["columns"] = 40, ["rows"] = 17 });
        await WaitFor(new() { ["columns"] = 40, ["rows"] = 17 });
        var dark = (await client.Success("fpasTuiSnapshot", new() { ["cells"] = true })).GetProperty("frame");
        await client.Success("fpasTuiTerminalInput", new() { ["bytes"] = Convert.ToBase64String(new byte[] { 20 }) });
        await WaitFor(new() { ["after"] = dark.GetProperty("id").GetInt64() });
        var light = (await client.Success("fpasTuiSnapshot", new() { ["cells"] = true })).GetProperty("frame");
        Assert.Equal(dark.GetProperty("lines")[0].GetString(), light.GetProperty("lines")[0].GetString());
        Assert.NotEqual(dark.GetProperty("cells")[0].GetProperty("colors").GetProperty("foreground").GetInt32(),
            light.GetProperty("cells")[0].GetProperty("colors").GetProperty("foreground").GetInt32());
        Assert.Contains(light.GetProperty("controls").EnumerateArray(), control =>
            control.GetProperty("id").GetString() == "navigation/Overview" && control.GetProperty("value").GetString() == "true");

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
    }
}
