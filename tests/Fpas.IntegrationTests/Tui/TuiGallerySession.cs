// [Documentation](../../coverage.md#terminal-ui)
using System.Buffers.Binary;
using System.Text.Json;
using System.Text.Json.Nodes;

namespace Fpas.IntegrationTests.Tui;

/// <summary>Reads framed gallery replies and inspects the public snapshots.</summary>
internal sealed class TuiGallerySession(Stream pipe, CancellationToken token)
{
    private int sequence;

    internal Task<string?> Key(string key) => Turn(new() { ["kind"] = "key", ["key"] = key });

    internal async Task Click(string label)
    {
        var control = Controls(await Request("snapshot")).Single(c => c.GetProperty("label").GetString() == label);
        await Turn(new() { ["kind"] = "click", ["button"] = "left",
            ["column"] = control.GetProperty("column").GetInt32() + 1, ["row"] = control.GetProperty("row").GetInt32() });
    }

    internal async Task Resize(int columns, int rows)
    {
        Assert.Equal("queued", Status(await Request("resize", new() { ["columns"] = columns, ["rows"] = rows })));
        Assert.Equal("presented", Status(await Request("step")));
    }

    internal async Task<string?> Turn(JsonObject input)
    {
        Assert.Equal("queued", Status(await Request("send", new() { ["input"] = input.DeepClone() })));
        var step = await Request("step");
        Assert.True(Status(step) != "failed", step.ToString());
        return Status(step);
    }

    internal static string? Status(JsonElement result) => result.GetProperty("status").GetString();

    internal async Task<JsonElement> Request(string operation, JsonObject? arguments = null)
    {
        var request = arguments?.DeepClone().AsObject() ?? new JsonObject();
        request["operation"] = operation;
        request["id"] = ++sequence;
        var bytes = JsonSerializer.SerializeToUtf8Bytes(request);
        var header = new byte[4];
        BinaryPrimitives.WriteInt32BigEndian(header, bytes.Length);
        await pipe.WriteAsync(header, token);
        await pipe.WriteAsync(bytes, token);
        while (true)
        {
            await pipe.ReadExactlyAsync(header, token);
            var length = BinaryPrimitives.ReadInt32BigEndian(header);
            Assert.InRange(length, 1, 16_777_216);
            var body = new byte[length];
            await pipe.ReadExactlyAsync(body, token);
            using var message = JsonDocument.Parse(body);
            if (message.RootElement.GetProperty("kind").GetString() != "reply") continue;
            Assert.Equal(sequence, message.RootElement.GetProperty("id").GetInt32());
            return message.RootElement.GetProperty("result").Clone();
        }
    }

    internal static JsonElement.ArrayEnumerator Controls(JsonElement frame) => frame.GetProperty("controls").EnumerateArray();

    internal static JsonElement Control(JsonElement frame, string id) =>
        Controls(frame).Single(c => c.GetProperty("id").GetString() == id);

    internal static void AssertCenteredDialog(JsonElement frame)
    {
        var rows = frame.GetProperty("lines").EnumerateArray().Select(row => row.GetString()!).ToArray();
        var width = frame.GetProperty("columns").GetInt32();
        var top = Array.FindIndex(rows, row => row.Contains("Delete this project?", StringComparison.Ordinal));
        Assert.InRange(top, 0, rows.Length - 1);
        var left = rows[top].LastIndexOf('╭', rows[top].IndexOf("Delete this project?", StringComparison.Ordinal));
        var right = rows[top].IndexOf('╮', left + 1);
        Assert.Equal((width - Math.Min(width - 2, 50)) / 2, left);
        Assert.Equal(Math.Min(width - 2, 50), right - left + 1);
        var bottom = Array.FindIndex(rows, top + 1, row => row[left] == '╰' && row[right] == '╯');
        Assert.InRange(rows.Length - 1 - bottom - top, 0, 1);
        Assert.DoesNotContain(Controls(frame), c => c.GetProperty("id").GetString() == "sheet-open"
            && c.GetProperty("enabled").GetBoolean());
    }

    internal static string? Focused(JsonElement frame) =>
        Controls(frame).Single(c => c.GetProperty("focused").GetBoolean()).GetProperty("label").GetString();

    internal static string? Selected(JsonElement frame) =>
        Controls(frame).Single(c => c.GetProperty("role").GetString() == "navigation" && c.GetProperty("value").GetString() == "true").GetProperty("label").GetString();

    internal static string Preview(JsonElement frame, int leftWidth) => string.Join("\n",
        frame.GetProperty("lines").EnumerateArray().Select(line => line.GetString()![(leftWidth + 2)..^1]));

    internal static (int Border, int Surface) Colors(JsonElement frame)
    {
        var colors = frame.GetProperty("cells")[0].GetProperty("colors");
        return (colors.GetProperty("foreground").GetInt32(), colors.GetProperty("background").GetInt32());
    }

    internal static void AssertPanels(JsonElement frame, int columns, int rows, int leftWidth)
    {
        Assert.Equal(columns, frame.GetProperty("columns").GetInt32());
        Assert.Equal(rows, frame.GetProperty("rows").GetInt32());
        Assert.NotEmpty(Controls(frame));
        var lines = frame.GetProperty("lines").EnumerateArray().Select(line => line.GetString()).ToArray();
        Assert.Equal(rows, lines.Length);
        int rightWidth = columns - leftWidth - 1;
        string Row(char start, char fill, char end) =>
            start + new string(fill, leftWidth - 2) + end + " " + start + new string(fill, rightWidth - 2) + end;
        Assert.Equal(Row('╭', '─', '╮'), lines[0]);
        Assert.All(lines[1..^1], line =>
        {
            Assert.Equal('│', line![0]);
            Assert.Equal("│ │", line.Substring(leftWidth - 1, 3));
            Assert.Equal('│', line[^1]);
        });
        Assert.Equal(Row('╰', '─', '╯'), lines[^1]);
    }
}
