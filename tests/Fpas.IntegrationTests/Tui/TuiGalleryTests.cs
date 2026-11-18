using System.Buffers.Binary;
using System.Diagnostics;
using System.IO.Pipes;
using System.Text.Json;
using System.Text.Json.Nodes;
using Fpas.IntegrationTests.Debugging;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Tui;

/// <summary>Verifies the public terminal gallery's panel layout and themes.</summary>
public sealed class TuiGalleryTests
{
    [Fact]
    public async Task Gallery_navigates_scrolls_and_previews_progress_and_text_across_resizes()
    {
        using var program = new Debuggee("tui-gallery");
        await TuiFixture.CompileExample(program, "gallery");
        var name = "fpas-tui-" + Guid.NewGuid().ToString("N");
        await using var pipe = new NamedPipeServerStream(name, PipeDirection.InOut, 1, PipeTransmissionMode.Byte,
            PipeOptions.Asynchronous | PipeOptions.CurrentUserOnly);
        var start = new ProcessStartInfo(BuildProcess.DotnetHost()) { WorkingDirectory = program.DirectoryPath };
        start.ArgumentList.Add(program.Assembly);
        start.ArgumentList.Add("--headless-control");
        start.Environment["FPAS_TUI_CONTROL_PIPE"] = name;
        using var process = Process.Start(start)!;
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(20));
        var token = deadline.Token;
        var sequence = 0;
        try
        {
            await pipe.WaitForConnectionAsync(token);
            var initial = await Request("snapshot");
            AssertPanels(initial, 80, 24, 26);
            Assert.Equal("Overview", Selected(initial));
            AssertProgress(initial, 26, 60);
            var dark = Colors(initial);
            await Key("Down");
            var moved = await Request("snapshot");
            Assert.Equal("Buttons", Focused(moved));
            Assert.Equal("Overview", Selected(moved));
            await Key("Enter");
            var buttons = await Request("snapshot");
            Assert.Equal("Buttons", Selected(buttons));
            Assert.Contains("Actions triggered", Preview(buttons, 26));
            await Resize(100, 28);
            AssertPanels(await Request("snapshot"), 100, 28, 33);
            await Resize(40, 17);
            var narrow = await Request("snapshot");
            AssertPanels(narrow, 40, 17, 13);
            Assert.Equal(dark, Colors(narrow));
            await Key("End");
            var end = await Request("snapshot");
            Assert.Equal("App chrome", Focused(end));
            await Key("Enter");
            Assert.Equal("App chrome", Selected(await Request("snapshot")));
            await Key("PageUp");
            var previous = Focused(await Request("snapshot"));
            Assert.NotEqual("App chrome", previous);
            await Key("PageDown");
            Assert.Equal("App chrome", Focused(await Request("snapshot")));
            await Key("Home");
            Assert.Equal("Overview", Focused(await Request("snapshot")));
            await Click("Text");
            var text = await Request("snapshot");
            Assert.Equal("Text", Selected(text));
            Assert.Contains("Field recording", Preview(text, 13));
            await Click("Progress bar");
            await Key("Right");
            AssertProgress(await Request("snapshot"), 13, 70);
            for (var i = 0; i < 8; i++) await Key("Left");
            AssertProgress(await Request("snapshot"), 13, 0);
            for (var i = 0; i < 11; i++) await Key("Right");
            AssertProgress(await Request("snapshot"), 13, 100);
            await Turn(new() { ["kind"] = "scroll", ["column"] = 3, ["row"] = 4, ["delta"] = 40 });
            Assert.Contains(Controls(await Request("snapshot")), c => c.GetProperty("label").GetString() == "App chrome");
            await Turn(new() { ["kind"] = "scroll", ["column"] = 3, ["row"] = 4, ["delta"] = -40 });

            Assert.Equal("presented", await Turn(new() { ["kind"] = "key", ["key"] = "T", ["modifiers"] = 4 }));
            var light = await Request("snapshot");
            AssertPanels(light, 40, 17, 13);
            var lightColors = Colors(light);
            Assert.NotEqual(dark.Border, lightColors.Border);
            Assert.NotEqual(dark.Surface, lightColors.Surface);
            Assert.Equal("presented", await Turn(new() { ["kind"] = "key", ["key"] = "T", ["modifiers"] = 4 }));
            await Resize(80, 24);
            var restored = await Request("snapshot");
            AssertPanels(restored, 80, 24, 26);
            Assert.Equal(dark, Colors(restored));
            Assert.Equal("Progress bar", Selected(restored));
            AssertProgress(restored, 26, 100);

            await Click("Feedback");
            var feedback = Preview(await Request("snapshot"), 26);
            Assert.Matches(@"Uploading assets +─+ +68%", feedback);
            Assert.Matches(@"Exporting clip\.mp4 +─+ +42%", feedback);
            await Key("Right");
            Assert.Matches(@"Uploading assets +─+ +78%", Preview(await Request("snapshot"), 26));

            await Click("Cell grid");
            Assert.Equal("processed", Status(await Request("step")));
            var gridFrame = await Request("snapshot");
            var grid = Controls(gridFrame).Single(c => c.GetProperty("role").GetString() == "cell-grid");
            int x = grid.GetProperty("column").GetInt32(), y = grid.GetProperty("row").GetInt32();
            int width = grid.GetProperty("width").GetInt32(), height = grid.GetProperty("height").GetInt32();
            Assert.True(width > 20 && height > 4);
            Assert.Contains($"{width} × {height}", Preview(gridFrame, 26));
            await Turn(new() { ["kind"] = "click", ["button"] = "left", ["column"] = x + 2, ["row"] = y + 1 });
            var centered = await Request("snapshot");
            Assert.Equal('+', centered.GetProperty("lines")[y + 1].GetString()![x + 2]);
            await Turn(new() { ["kind"] = "scroll", ["column"] = x + 3, ["row"] = y, ["delta"] = -3 });
            Assert.Contains("Wheel (3, 0) -3", Preview(await Request("snapshot"), 26));
            await Resize(100, 30);
            Assert.Equal("processed", Status(await Request("step")));
            var resized = await Request("snapshot");
            var bigger = Controls(resized).Single(c => c.GetProperty("role").GetString() == "cell-grid");
            Assert.True(bigger.GetProperty("width").GetInt32() > width && bigger.GetProperty("height").GetInt32() > height);
            Assert.Equal("idle", Status(await Request("step")));

            await Click("Cell grid");
            await Key("End");
            await Key("Enter");
            var chrome = await Request("snapshot");
            var encoding = Controls(chrome).Single(c => c.GetProperty("role").GetString() == "status-item");
            var footerRow = encoding.GetProperty("row").GetInt32();
            Assert.InRange(footerRow, 26, 28);
            var footer = chrome.GetProperty("lines")[footerRow].GetString()!;
            Assert.Matches(@"main +Saved +Indexing 62% +─{8} +Ln 48, Col 17 +UTF-8", footer);
            Assert.Contains("────────", chrome.GetProperty("lines")[footerRow - 1].GetString());
            await Click("UTF-8");
            Assert.Equal("UTF-16", Focused(await Request("snapshot")));
            await Key("Enter");
            Assert.Equal("UTF-8", Focused(await Request("snapshot")));

            Assert.Equal("unchanged", await Turn(new() { ["kind"] = "key", ["key"] = "Q", ["modifiers"] = 4 }));
            await process.WaitForExitAsync(token);
            Assert.Equal(0, process.ExitCode);
        }
        finally
        {
            if (!process.HasExited)
            {
                process.Kill();
                await process.WaitForExitAsync();
            }
        }

        Task<string?> Key(string key) => Turn(new() { ["kind"] = "key", ["key"] = key });

        async Task Click(string label)
        {
            var control = Controls(await Request("snapshot")).Single(c => c.GetProperty("label").GetString() == label);
            await Turn(new() { ["kind"] = "click", ["button"] = "left",
                ["column"] = control.GetProperty("column").GetInt32() + 1, ["row"] = control.GetProperty("row").GetInt32() });
        }

        async Task Resize(int columns, int rows)
        {
            Assert.Equal("queued", Status(await Request("resize", new() { ["columns"] = columns, ["rows"] = rows })));
            Assert.Equal("presented", Status(await Request("step")));
        }

        async Task<string?> Turn(JsonObject input)
        {
            Assert.Equal("queued", Status(await Request("send", new() { ["input"] = input.DeepClone() })));
            return Status(await Request("step"));
        }

        async Task<JsonElement> Request(string operation, JsonObject? arguments = null)
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

        static string? Status(JsonElement result) => result.GetProperty("status").GetString();
    }

    private static JsonElement.ArrayEnumerator Controls(JsonElement frame) => frame.GetProperty("controls").EnumerateArray();

    private static string? Focused(JsonElement frame) =>
        Controls(frame).Single(c => c.GetProperty("focused").GetBoolean()).GetProperty("label").GetString();

    private static string? Selected(JsonElement frame) =>
        Controls(frame).Single(c => c.GetProperty("value").GetString() == "true").GetProperty("label").GetString();

    private static string Preview(JsonElement frame, int leftWidth) => string.Join("\n",
        frame.GetProperty("lines").EnumerateArray().Select(line => line.GetString()![(leftWidth + 2)..^1]));

    private static void AssertProgress(JsonElement frame, int leftWidth, int percent)
    {
        var preview = Preview(frame, leftWidth);
        var line = Assert.Single(preview.Split('\n'), row => row.TrimStart().StartsWith("Loading", StringComparison.Ordinal));
        var parts = line.Split("  ", StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
        Assert.Equal(3, parts.Length);
        Assert.StartsWith(parts[0], "Loading tracks");
        Assert.All(parts[1], cell => Assert.Equal('─', cell));
        Assert.Equal($"{percent}%", parts[2]);
    }

    private static (int Border, int Surface) Colors(JsonElement frame)
    {
        var colors = frame.GetProperty("cells")[0].GetProperty("colors");
        return (colors.GetProperty("foreground").GetInt32(), colors.GetProperty("background").GetInt32());
    }

    private static void AssertPanels(JsonElement frame, int columns, int rows, int leftWidth)
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
