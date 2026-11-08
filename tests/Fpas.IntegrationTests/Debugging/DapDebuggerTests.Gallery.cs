using System.Text.Json;
using System.Text.Json.Nodes;
using Fpas.IntegrationTests.Tui;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class DapDebuggerTests
{
    [Fact]
    public async Task Gallery_overlay_and_editor_keep_state_in_the_private_terminal()
    {
        using var program = new Debuggee("gallery-tui");
        await TuiFixture.CompileExample(program, "gallery");

        await using var client = new DapProbe();
        await client.Success("initialize");
        await client.Success("launch", new() { ["program"] = program.Assembly, ["fpasTui"] = true });
        await client.Event("initialized");
        await client.Success("configurationDone");

        await WaitFor(new() { ["control"] = "pages/overlay" });
        await Send(new() { ["kind"] = "activate", ["control"] = "pages/tabs" });
        await Send(new() { ["kind"] = "focus", ["control"] = "settings-tabs" });
        await client.Success("fpasTuiTerminalInput",
            new() { ["bytes"] = Convert.ToBase64String("\u001b[C"u8) });
        await WaitFor(new() { ["control"] = "tab-colors" });
        await Send(new() { ["kind"] = "activate", ["control"] = "pages/overlay" });
        await Send(new() { ["kind"] = "focus", ["control"] = "overlay-open" });
        await Send(new() { ["kind"] = "activate", ["control"] = "overlay-open" });
        var modal = (await client.Success("fpasTuiSnapshot")).GetProperty("frame");
        Assert.True(Control(modal, "overlay-close").GetProperty("focused").GetBoolean());
        Assert.False(Control(modal, "overlay-open").GetProperty("enabled").GetBoolean());
        Assert.Equal("unchanged", await Send(new() { ["kind"] = "focus", ["control"] = "pages/buttons" }));
        Assert.Equal("unchanged", await Send(new() { ["kind"] = "activate", ["control"] = "pages/buttons" }));
        Assert.Equal("unchanged", await Send(new() { ["kind"] = "key", ["key"] = "Q", ["modifiers"] = 4 }));

        await client.Success("fpasTuiResize", new() { ["columns"] = 40, ["rows"] = 17 });
        await WaitFor(new() { ["columns"] = 40, ["rows"] = 17, ["control"] = "next-page" });
        var resized = (await client.Success("fpasTuiSnapshot")).GetProperty("frame");
        Assert.True(Control(resized, "overlay-close").GetProperty("focused").GetBoolean());
        Assert.True(Control(resized, "overlay-body").GetProperty("height").GetInt32() <
            Control(modal, "overlay-body").GetProperty("height").GetInt32());

        await client.Success("fpasTuiTerminalInput", new() { ["bytes"] = Convert.ToBase64String("\r"u8) });
        await WaitFor(new() { ["after"] = resized.GetProperty("id").GetInt64() });
        var closed = (await client.Success("fpasTuiSnapshot")).GetProperty("frame");
        Assert.True(Control(closed, "overlay-open").GetProperty("focused").GetBoolean());
        Assert.Contains("40 × 17", closed.GetProperty("lines")[1].GetString());
        Assert.DoesNotContain(closed.GetProperty("controls").EnumerateArray(),
            item => item.GetProperty("id").GetString() == "overlay-close");

        await client.Success("fpasTuiResize", new() { ["columns"] = 80, ["rows"] = 24 });
        await WaitFor(new() { ["columns"] = 80, ["rows"] = 24, ["control"] = "pages/multiline" });
        await Send(new() { ["kind"] = "activate", ["control"] = "pages/multiline" });
        await Send(new() { ["kind"] = "key", ["key"] = "End", ["modifiers"] = 4 });
        await WaitFor(new() { ["text"] = "End of notes" });
        await WaitFor(new() { ["text"] = "Unchanged" });
        var notes = (await client.Success("fpasTuiSnapshot")).GetProperty("frame");
        var editor = Control(notes, "document-editor");
        await Send(new()
        {
            ["kind"] = "scroll", ["delta"] = -100,
            ["column"] = editor.GetProperty("column").GetInt32(), ["row"] = editor.GetProperty("row").GetInt32()
        });
        await WaitFor(new() { ["text"] = "Field notes" });
        await Send(new() { ["kind"] = "activate", ["control"] = "document-draft" });
        await WaitFor(new() { ["text"] = "Draft idea" });
        await Send(new() { ["kind"] = "activate", ["control"] = "document-notes" });
        await WaitFor(new() { ["text"] = "Field notes" });
        var restored = (await client.Success("fpasTuiSnapshot")).GetProperty("frame");
        await client.Success("fpasTuiTerminalInput",
            new() { ["bytes"] = Convert.ToBase64String("\u001b[200~\r\nAdded\r\nRow\u001b[201~"u8) });
        await WaitFor(new() { ["text"] = "Row" });
        var edited = (await client.Success("fpasTuiSnapshot")).GetProperty("frame");
        Assert.Equal(restored.GetProperty("id").GetInt64() + 1, edited.GetProperty("id").GetInt64());
        Assert.EndsWith("End of notes\nAdded\nRow",
            Control(edited, "document-editor").GetProperty("value").GetString()!, StringComparison.Ordinal);
        await WaitFor(new() { ["text"] = "Edited" });

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
