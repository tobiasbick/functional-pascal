using System.Text.Json;
using System.Text.Json.Nodes;
using Fpas.IntegrationTests.Tui;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class DapDebuggerTests
{
    [Fact]
    public async Task Mandelbrot_responds_to_real_terminal_keys_mouse_and_resize()
    {
        using var program = new Debuggee("mandelbrot-tui");
        await TuiFixture.CompileExample(program, "mandelbrot");

        await using var client = new DapProbe();
        await client.Success("initialize");
        await client.Success("launch", new() { ["program"] = program.Assembly, ["fpasTui"] = true });
        await client.Event("initialized");
        await client.Success("configurationDone");

        await Wait(new() { ["text"] = "Deep Ocean  ready" });
        var first = await Frame();
        CheckImage(first, 78, 18);

        await Type("2"u8.ToArray());
        await Wait(new() { ["text"] = "Ember  ready" });
        Assert.Contains((await Frame()).GetProperty("cells").EnumerateArray(), cell =>
            cell.GetProperty("style").GetProperty("foreground").GetInt32() == 0xB03A00);

        await Type("+\u001b[<64;8;6M"u8.ToArray());
        await Wait(new() { ["text"] = "Zoom 1.39x" });
        await Type("\u001b[<0;8;6M\u001b[<0;8;6m"u8.ToArray());
        var centered = FormattableString.Invariant($"center=({-0.75 + 2.4 * 0.82 * 0.88 * 78 / 18 * 0.5 * (6.0 / 77 - 0.5):F4}, {2.4 * 0.82 * 0.88 * (0.5 - 4.0 / 17):F4})");
        await Wait(new() { ["text"] = centered });
        await Type("\u001b[<1;8;6M\u001b[<1;8;6m"u8.ToArray());
        await Wait(new() { ["text"] = "center=(-0.7500, 0.0000)  Deep Ocean  ready" });

        await Type("H"u8.ToArray());
        await Wait(new() { ["control"] = "close-help" });
        var help = await Frame();
        await Type("3\u001b[<64;8;6M\u001b"u8.ToArray());
        await Wait(new() { ["after"] = help.GetProperty("id").GetInt64() });
        var afterHelp = await Frame();
        Assert.DoesNotContain(afterHelp.GetProperty("controls").EnumerateArray(), control =>
            control.GetProperty("id").GetString() == "close-help");
        Assert.Contains(afterHelp.GetProperty("lines").EnumerateArray(), row =>
            row.GetString()!.Contains("Deep Ocean  ready", StringComparison.Ordinal));
        Assert.Contains(afterHelp.GetProperty("lines").EnumerateArray(), row =>
            row.GetString()!.Contains("Zoom 1.00x", StringComparison.Ordinal));

        await client.Success("fpasTuiResize", new() { ["columns"] = 53, ["rows"] = 17 });
        await Wait(new() { ["columns"] = 53, ["rows"] = 17, ["text"] = "Deep Ocean  ready" });
        CheckImage(await Frame(), 51, 11);

        await Type("\u001b"u8.ToArray());
        Assert.Equal(0, (await client.Event("exited")).GetProperty("exitCode").GetInt32());
        await client.Event("terminated");
        await client.Success("disconnect");
        await client.End();

        async Task Wait(JsonObject frame)
        {
            var response = await client.Success("fpasTuiWait", new() { ["frame"] = frame });
            if (response.GetProperty("status").GetString() != "matched")
            {
                var snapshot = (await client.Success("fpasTuiSnapshot")).GetProperty("frame");
                Assert.Fail($"{frame}\n{string.Join('\n', snapshot.GetProperty("lines").EnumerateArray().Select(row => row.GetString()))}");
            }
        }

        async Task<JsonElement> Frame() => (await client.Success("fpasTuiSnapshot", new() { ["cells"] = true })).GetProperty("frame");

        async Task Type(byte[] bytes) => await client.Success("fpasTuiTerminalInput",
            new() { ["bytes"] = Convert.ToBase64String(bytes) });
    }

    private static void CheckImage(JsonElement frame, int width, int height)
    {
        var canvas = Assert.Single(frame.GetProperty("controls").EnumerateArray(),
            control => control.GetProperty("id").GetString() == "fractal");
        Assert.Equal(width, canvas.GetProperty("width").GetInt32());
        Assert.Equal(height, canvas.GetProperty("height").GetInt32());
        var cells = frame.GetProperty("cells");
        var stride = frame.GetProperty("columns").GetInt32();
        var start = canvas.GetProperty("row").GetInt32() * stride + canvas.GetProperty("column").GetInt32();
        Assert.Equal(0, cells[start + height / 2 * stride + width / 2].GetProperty("style").GetProperty("foreground").GetInt32());
        Assert.InRange(cells[start].GetProperty("style").GetProperty("foreground").GetInt32(), 1, 0xFFFFFF);
    }
}
