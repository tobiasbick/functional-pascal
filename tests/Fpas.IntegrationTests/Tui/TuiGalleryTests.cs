using System.Buffers.Binary;
using System.Diagnostics;
using System.IO.Pipes;
using System.Text.Json;
using System.Text.Json.Nodes;
using Fpas.IntegrationTests.Debugging;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Tui;

/// <summary>Exercises gallery widgets, layout and themes over the terminal control pipe.</summary>
public sealed class TuiGalleryTests
{
    /// <summary>Navigates the gallery, edits controls, resizes views and switches themes.</summary>
    [Fact]
    public async Task Gallery_can_be_inspected_and_stepped_over_the_control_pipe()
    {
        using var program = new Debuggee("headless-control");
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
            var first = await Request("snapshot");
            var initialId = first.GetProperty("id").GetInt64();
            Assert.Equal(80, first.GetProperty("columns").GetInt32());
            Assert.Contains("80 × 24", Screen(first));
            Assert.EndsWith("Ctrl+Q Quit  Tab Focus  PageUp Previous page  PageDown Next page",
                first.GetProperty("lines")[22].GetString()!.TrimEnd(), StringComparison.Ordinal);
            Assert.False(Control(first, "button-disabled").GetProperty("enabled").GetBoolean());
            var queued = await Activate("button-press");
            Assert.Equal("queued", Status(queued));
            var wait = new JsonObject { ["inputId"] = queued.GetProperty("inputId").GetInt64(), ["timeoutMs"] = 0 };
            Assert.Equal("timeout", Status(await Request("wait", wait)));
            Assert.Equal(initialId, (await Request("snapshot")).GetProperty("id").GetInt64());
            Assert.Equal("presented", Status(await Request("step")));
            Assert.Equal("presented", Status(await Request("wait", wait)));
            var pressed = await Request("snapshot");
            Assert.Contains("Presses: 1", Screen(pressed));
            Assert.True(Control(pressed, "pages/buttons").GetProperty("focused").GetBoolean());
            Assert.Equal("presented", await Focus("button-press"));
            Assert.Contains("Presses: 1", Screen(await Request("snapshot")));
            Assert.Equal("presented", await Key("Enter"));
            Assert.Contains("Presses: 2", Screen(await Request("snapshot")));
            Assert.Equal("queued", Status(await Activate("button-disabled")));
            Assert.Equal("unchanged", Status(await Request("step")));
            Assert.Equal("unchanged", await Focus("button-disabled"));
            Assert.True(Control(await Request("snapshot"), "button-press").GetProperty("focused").GetBoolean());

            await CheckResponsiveView();
            await CheckButtonVariants();

            await Choose("pages/checkbox");
            Assert.Equal("presented", await Focus("check-toggle"));
            Assert.Equal("false", Control(await Request("snapshot"), "check-toggle").GetProperty("value").GetString());
            Assert.Equal("presented", await Key("Enter"));
            Assert.Equal("true", Control(await Request("snapshot"), "check-toggle").GetProperty("value").GetString());
            Assert.Equal("presented", await Focus("pages/checkbox"));
            await Choose("check-toggle");
            var toggled = await Request("snapshot");
            Assert.Equal("false", Control(toggled, "check-toggle").GetProperty("value").GetString());
            Assert.True(Control(toggled, "pages/checkbox").GetProperty("focused").GetBoolean());

            await CheckTextInput();

            await Choose("pages/lists");
            await Choose("list-empty");
            Assert.Contains("No items.", Screen(await Request("snapshot")));
            await Choose("list-empty");
            Assert.Equal("birch", Control(await Request("snapshot"), "sample-list").GetProperty("value").GetString());
            Assert.Equal("presented", await Focus("list-empty"));
            Assert.Equal("presented", await Key("Tab"));
            Assert.True(Control(await Request("snapshot"), "sample-list").GetProperty("focused").GetBoolean());

            Assert.Equal("presented", await Key("Down"));
            var disabled = await Request("snapshot");
            Assert.Equal("cedar", Control(disabled, "sample-list").GetProperty("value").GetString());
            await Key("Enter");
            Assert.Contains("Opened: -", Screen(await Request("snapshot")));
            await Turn(new() { ["kind"] = "activate", ["control"] = "sample-list" });
            Assert.Contains("Opened: -", Screen(await Request("snapshot")));

            Assert.Equal("presented", await Key("End"));
            var end = await Request("snapshot");
            Assert.Contains("▸ Juniper", Screen(end));
            Assert.Equal("presented", await Key("Enter"));
            Assert.Contains("Opened: Juniper", Screen(await Request("snapshot")));

            var list = Control(end, "sample-list");
            var column = list.GetProperty("column").GetInt32();
            var row = list.GetProperty("row").GetInt32();
            var scroll = new JsonObject { ["kind"] = "scroll", ["column"] = column, ["row"] = row, ["delta"] = -9 };
            Assert.Equal("presented", await Turn(scroll));
            var scrolled = await Request("snapshot");
            Assert.Equal("juniper", Control(scrolled, "sample-list").GetProperty("value").GetString());
            Assert.DoesNotContain("▸ Juniper", Screen(scrolled));
            Assert.Equal("presented", await Key("End"));
            Assert.Contains("▸ Juniper", Screen(await Request("snapshot")));
            Assert.Equal("presented", await Turn(scroll));
            Assert.Equal("presented", await Turn(new() { ["kind"] = "click", ["column"] = column, ["row"] = row }));
            Assert.Contains("Opened: Juniper", Screen(await Request("snapshot")));
            Assert.Equal("presented", await Key("Tab"));
            Assert.True(Control(await Request("snapshot"), "theme-light").GetProperty("focused").GetBoolean());
            await Choose("sample-list");
            var opened = await Request("snapshot");
            Assert.Contains("Opened: Amber", Screen(opened));
            Assert.True(Control(opened, "theme-light").GetProperty("focused").GetBoolean());

            Assert.Equal("presented", await Focus("theme-blue"));
            await Choose("theme-blue");
            var themed = await Request("snapshot");
            Assert.Equal(0x121212, themed.GetProperty("cells")[0].GetProperty("colors").GetProperty("background").GetInt32());
            Assert.Equal("amber", Control(themed, "sample-list").GetProperty("value").GetString());
            Assert.Contains("▸ Amber", Screen(themed));
            Assert.True(Control(themed, "theme-blue").GetProperty("focused").GetBoolean());

            await CheckLog();
            await CheckTabs();
            await CheckTable();
            await CheckOverlay();

            await Choose("pages/canvas");
            var canvas = Control(await Request("snapshot"), "sample-canvas");
            Assert.Equal("queued", Status(await Request("resize", new() { ["columns"] = 53, ["rows"] = 17 })));
            Assert.Equal("presented", Status(await Request("step")));
            var resized = await Request("snapshot");
            Assert.Equal(53, resized.GetProperty("columns").GetInt32());
            Assert.Equal(17, resized.GetProperty("rows").GetInt32());
            var resizedCanvas = Control(resized, "sample-canvas");
            Assert.True(resizedCanvas.GetProperty("width").GetInt32() < canvas.GetProperty("width").GetInt32());
            Assert.Contains($"Canvas: {resizedCanvas.GetProperty("width").GetInt32()} × {resizedCanvas.GetProperty("height").GetInt32()}",
                Screen(resized));
            Assert.Equal("queued", Status(await Request("send", new()
            {
                ["input"] = new JsonObject { ["kind"] = "key", ["key"] = "Q", ["modifiers"] = 4 }
            })));
            Assert.Equal("unchanged", Status(await Request("step")));
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

        Task<JsonElement> Activate(string id) =>
            Request("send", new() { ["input"] = new JsonObject { ["kind"] = "activate", ["control"] = id } });

        async Task Choose(string id)
        {
            Assert.Equal("queued", Status(await Activate(id)));
            Assert.Equal("presented", Status(await Request("step")));
        }

        async Task CheckResponsiveView()
        {
            Assert.Equal("queued", Status(await Request("resize", new() { ["columns"] = 40, ["rows"] = 17 })));
            Assert.Contains("80 × 24", Screen(await Request("snapshot")));
            Assert.Equal("presented", Status(await Request("step")));
            var compact = await Request("snapshot");
            Assert.Contains("40 × 17", Screen(compact));
            Assert.DoesNotContain(compact.GetProperty("controls").EnumerateArray(),
                item => item.GetProperty("id").GetString() == "pages/buttons");
            Assert.True(Control(compact, "button-press").GetProperty("focused").GetBoolean());
            await Choose("next-page");
            Assert.True(Control(await Request("snapshot"), "check-toggle").GetProperty("enabled").GetBoolean());
            await Choose("previous-page");

            await Resize(32, 17);
            await Choose("theme-amber");
            var amber = await Request("snapshot");
            Assert.Contains(" Amber ", Screen(amber));
            Assert.Equal(0x121212, CellStyle(amber, 0, 0).GetProperty("background").GetInt32());
            await Choose("theme-dark");

            await Resize(40, 11);
            Assert.Contains("Resize to 32 × 12", Screen(await Request("snapshot")));
            await Resize(80, 24);
            var restored = await Request("snapshot");
            Assert.Contains("80 × 24", Screen(restored));
            Assert.Contains("Presses: 2", Screen(restored));
            Assert.True(Control(restored, "pages/buttons").GetProperty("enabled").GetBoolean());
        }

        async Task CheckButtonVariants()
        {
            var normal = await Request("snapshot");
            Assert.Equal(0xE8E8E8, ControlStyle(normal, "button-press").GetProperty("background").GetInt32());
            Assert.Equal(0x8C8C8C, ControlStyle(normal, "button-disabled").GetProperty("foreground").GetInt32());
            Assert.Equal(0x2D2D2D, ControlStyle(normal, "theme-dark").GetProperty("background").GetInt32());
            Assert.Equal(0xF2716C, ControlStyle(normal, "button-reset").GetProperty("background").GetInt32());
            var reset = Control(normal, "button-reset");
            Assert.Equal("presented", await Turn(new()
            {
                ["kind"] = "move", ["column"] = reset.GetProperty("column").GetInt32(),
                ["row"] = reset.GetProperty("row").GetInt32()
            }));
            Assert.Equal(0x232323, ControlStyle(await Request("snapshot"), "button-reset").GetProperty("background").GetInt32());
            Assert.Equal("presented", await Focus("button-reset"));
            var focused = ControlStyle(await Request("snapshot"), "button-reset");
            Assert.Equal(0xF2716C, focused.GetProperty("foreground").GetInt32());
            Assert.Equal(0x121212, focused.GetProperty("background").GetInt32());
            Assert.Equal("presented", await Key("Enter"));
            Assert.Contains("Presses: 0", Screen(await Request("snapshot")));
        }

        async Task CheckTextInput()
        {
            await Choose("pages/textinput");
            var empty = await Request("snapshot");
            var field = Control(empty, "text-entry");
            Assert.True(field.GetProperty("focused").GetBoolean());
            Assert.Equal("", field.GetProperty("value").GetString());
            Assert.Contains("Type something…", Screen(empty));
            var column = field.GetProperty("column").GetInt32();
            var row = field.GetProperty("row").GetInt32();
            Assert.Equal(0xBEBEBE, CellStyle(empty, column + 2, row).GetProperty("background").GetInt32());
            Assert.Equal(0x8C8C8C, CellStyle(empty, column + 3, row).GetProperty("foreground").GetInt32());

            await Choose("theme-blue");
            var themed = await Request("snapshot");
            Assert.True(Control(themed, "text-entry").GetProperty("focused").GetBoolean());
            Assert.Equal(0x142847, ControlStyle(themed, "theme-blue").GetProperty("background").GetInt32());
            Assert.Equal(0x8C8C8C, CellStyle(themed, column + 3, row).GetProperty("foreground").GetInt32());
            Assert.Equal(0x181818, CellStyle(themed, column + 3, row).GetProperty("background").GetInt32());

            Assert.Equal("presented", await Focus("pages/textinput"));
            Assert.Equal("unchanged", await Turn(new() { ["kind"] = "activate", ["control"] = "text-entry" }));
            var unfocused = await Request("snapshot");
            Assert.True(Control(unfocused, "pages/textinput").GetProperty("focused").GetBoolean());
            Assert.Contains("Type something…", Screen(unfocused));
            Assert.Equal(0x8C8C8C, CellStyle(unfocused, column + 2, row).GetProperty("foreground").GetInt32());

            Assert.Equal("presented", await Click(column + 8, row));
            Assert.Equal("presented", await Turn(new() { ["kind"] = "paste", ["text"] = "one\r\ntwo" }));
            var filled = await Request("snapshot");
            Assert.Equal("one two", Control(filled, "text-entry").GetProperty("value").GetString());
            Assert.DoesNotContain("Type something…", Screen(filled));

            Assert.Equal("presented", await Turn(new() { ["kind"] = "key", ["key"] = "A", ["modifiers"] = 4 }));
            Assert.Equal("presented", await Key("Backspace"));
            var cleared = await Request("snapshot");
            Assert.Equal("", Control(cleared, "text-entry").GetProperty("value").GetString());
            Assert.Contains("Type something…", Screen(cleared));
            await Choose("theme-dark");
        }

        async Task CheckLog()
        {
            await Choose("pages/scrolling");
            await Choose("log-append");
            var following = await Request("snapshot");
            Assert.Contains("Following newest", Screen(following));
            Assert.Contains("Event 10", Screen(following));
            var viewport = Control(following, "sample-scroll");
            var column = viewport.GetProperty("column").GetInt32();
            var row = viewport.GetProperty("row").GetInt32();
            var bar = column + viewport.GetProperty("width").GetInt32() - 1;
            var lastRow = row + viewport.GetProperty("height").GetInt32() - 1;
            Assert.Equal("█", Cell(following, bar, lastRow));

            Assert.Equal("presented", await Focus("sample-scroll"));
            Assert.Equal("presented", await Key("Home"));
            await Choose("log-append");
            var paused = await Request("snapshot");
            Assert.Contains("Paused", Screen(paused));
            Assert.Contains("Event 01", Screen(paused));
            Assert.DoesNotContain("Event 11", Screen(paused));
            Assert.Equal("0", Control(paused, "sample-scroll").GetProperty("value").GetString());
            Assert.Equal("█", Cell(paused, bar, row));
            Assert.True(Control(paused, "sample-scroll").GetProperty("focused").GetBoolean());

            Assert.Equal("presented", await Key("PageDown"));
            Assert.DoesNotContain("Event 01", Screen(await Request("snapshot")));
            Assert.Equal("presented", await Turn(new() { ["kind"] = "scroll", ["column"] = column, ["row"] = row, ["delta"] = -20 }));
            Assert.Contains("Event 01", Screen(await Request("snapshot")));
            Assert.Equal("presented", await Key("End"));
            Assert.Contains("Event 11", Screen(await Request("snapshot")));
            await Choose("log-append");
            Assert.Contains("Event 12", Screen(await Request("snapshot")));
        }

        async Task CheckTabs()
        {
            await Choose("pages/tabs");
            Assert.Equal("presented", await Focus("settings-tabs"));
            var general = await Request("snapshot");
            var bar = Control(general, "settings-tabs");
            Assert.Equal("general", bar.GetProperty("value").GetString());
            var row = bar.GetProperty("row").GetInt32();
            var disabledColumn = general.GetProperty("lines")[row].GetString()!
                .IndexOf("Updates unavailable", StringComparison.Ordinal);
            Assert.True(disabledColumn >= 0);
            await Click(disabledColumn, row);
            Assert.Equal("general", Control(await Request("snapshot"), "settings-tabs").GetProperty("value").GetString());

            Assert.Equal("presented", await Key("Right"));
            var appearance = await Request("snapshot");
            Assert.Equal("appearance", Control(appearance, "settings-tabs").GetProperty("value").GetString());
            Assert.DoesNotContain(appearance.GetProperty("controls").EnumerateArray(),
                item => item.GetProperty("id").GetString() == "tab-name");
            Assert.Equal("presented", await Key("Tab"));
            Assert.True(Control(await Request("snapshot"), "tab-colors").GetProperty("focused").GetBoolean());
            Assert.Equal("presented", await Key("Enter"));

            Assert.Equal("presented", await Turn(new() { ["kind"] = "key", ["key"] = "T", ["modifiers"] = 4 }));
            var switched = Control(await Request("snapshot"), "settings-tabs");
            Assert.Equal("general", switched.GetProperty("value").GetString());
            Assert.True(switched.GetProperty("focused").GetBoolean());
            Assert.Equal("presented", await Key("Left"));
            Assert.Equal("shortcuts", Control(await Request("snapshot"), "settings-tabs").GetProperty("value").GetString());
            Assert.Equal("presented", await Key("Home"));
            Assert.Equal("general", Control(await Request("snapshot"), "settings-tabs").GetProperty("value").GetString());

            await Resize(44, 17);
            var narrowFrame = await Request("snapshot");
            var narrowStatus = narrowFrame.GetProperty("lines")[15].GetString()!.Trim();
            Assert.StartsWith("Tabs", narrowStatus, StringComparison.Ordinal);
            Assert.EndsWith("Ctrl+Q Quit  Tab Focus", narrowStatus, StringComparison.Ordinal);
            Assert.DoesNotContain("PageUp", narrowStatus);
            var narrow = Control(narrowFrame, "settings-tabs");
            Assert.True(narrow.GetProperty("width").GetInt32() < bar.GetProperty("width").GetInt32());
            var left = narrow.GetProperty("column").GetInt32();
            var right = left + narrow.GetProperty("width").GetInt32() - 1;
            row = narrow.GetProperty("row").GetInt32();
            Assert.Equal("presented", await Click(right, row));
            var scrolled = await Request("snapshot");
            Assert.Equal("appearance", Control(scrolled, "settings-tabs").GetProperty("value").GetString());
            Assert.Equal("false", Control(scrolled, "tab-colors").GetProperty("value").GetString());
            Assert.Equal("presented", await Click(left, row));
            Assert.Equal("general", Control(await Request("snapshot"), "settings-tabs").GetProperty("value").GetString());
            await Resize(80, 24);
        }

        Task<string?> Click(int column, int row) =>
            Turn(new() { ["kind"] = "click", ["column"] = column, ["row"] = row });

        async Task CheckTable()
        {
            await Choose("pages/table");
            Assert.Equal("presented", await Focus("world-table"));
            var first = await Request("snapshot");
            var table = Control(first, "world-table");
            Assert.Equal("table", table.GetProperty("role").GetString());
            var row = table.GetProperty("row").GetInt32();
            var column = table.GetProperty("column").GetInt32();
            var header = first.GetProperty("lines")[row - 1].GetString()!;
            var status = header.IndexOf("Status", StringComparison.Ordinal);
            var players = header.IndexOf("Players", StringComparison.Ordinal);
            Assert.True(status > column && players > status);

            Assert.Equal("presented", await Key("Down"));
            await Key("Enter");
            var disabled = await Request("snapshot");
            Assert.Equal("birch", Control(disabled, "world-table").GetProperty("value").GetString());
            Assert.Contains("Opened: -", Screen(disabled));
            Assert.Equal("presented", await Key("End"));
            Assert.Equal("presented", await Key("Enter"));
            Assert.Contains("Opened: juniper", Screen(await Request("snapshot")));

            Assert.Equal("presented", await Turn(new() { ["kind"] = "scroll", ["column"] = column, ["row"] = row, ["delta"] = -20 }));
            var scrolled = await Request("snapshot");
            Assert.Equal(header, scrolled.GetProperty("lines")[row - 1].GetString());
            Assert.Equal("juniper", Control(scrolled, "world-table").GetProperty("value").GetString());
            await Click(column + 4, row - 1);
            Assert.Equal("juniper", Control(await Request("snapshot"), "world-table").GetProperty("value").GetString());
            Assert.Equal("presented", await Click(column + 4, row));
            var selected = await Request("snapshot");
            Assert.Equal("alder", Control(selected, "world-table").GetProperty("value").GetString());
            Assert.Contains("Opened: juniper", Screen(selected));

            await Resize(53, 17);
            var resized = await Request("snapshot");
            var narrower = Control(resized, "world-table");
            var narrowHeader = resized.GetProperty("lines")[narrower.GetProperty("row").GetInt32() - 1].GetString()!;
            var narrowStatus = narrowHeader.IndexOf("Status", StringComparison.Ordinal);
            var narrowPlayers = narrowHeader.IndexOf("Players", StringComparison.Ordinal);
            Assert.InRange(narrowStatus, column + 1, status - 1);
            Assert.Equal(players - status, narrowPlayers - narrowStatus);
            Assert.Equal("alder", narrower.GetProperty("value").GetString());
            await Resize(80, 24);
        }

        async Task Resize(int columns, int rows)
        {
            Assert.Equal("queued", Status(await Request("resize", new() { ["columns"] = columns, ["rows"] = rows })));
            Assert.Equal("presented", Status(await Request("step")));
        }

        async Task CheckOverlay()
        {
            await Choose("pages/overlay");
            Assert.Equal("presented", await Focus("overlay-open"));
            await Choose("overlay-open");
            var first = await Request("snapshot");
            var status = first.GetProperty("lines")[22].GetString()!;
            Assert.Contains("Overlay open", status);
            Assert.Contains("Escape Close", status);
            Assert.DoesNotContain("Ctrl+Q", status);
            Assert.Contains("Scroll from here.", Screen(first));
            Assert.Contains("mouse wheel.", Screen(first));
            Assert.DoesNotContain("End of overlay notes.", Screen(first));
            var body = Control(first, "overlay-body");
            var footer = Control(first, "overlay-close");
            Assert.Equal(60, footer.GetProperty("width").GetInt32());
            Assert.Equal(9, body.GetProperty("height").GetInt32());
            Assert.True(footer.GetProperty("focused").GetBoolean());

            Assert.Equal("presented", await Focus("overlay-body"));
            Assert.Equal("presented", await Key("End"));
            var end = await Request("snapshot");
            Assert.Contains("End of overlay notes.", Screen(end));
            Assert.Equal(footer.GetProperty("row").GetInt32(),
                Control(end, "overlay-close").GetProperty("row").GetInt32());
            Assert.Equal("presented", await Turn(new()
            {
                ["kind"] = "scroll", ["delta"] = -100,
                ["column"] = body.GetProperty("column").GetInt32(), ["row"] = body.GetProperty("row").GetInt32()
            }));
            Assert.Contains("Scroll from here.", Screen(await Request("snapshot")));

            await Resize(40, 3);
            var tiny = await Request("snapshot");
            Assert.DoesNotContain(tiny.GetProperty("controls").EnumerateArray(),
                item => item.GetProperty("id").GetString() == "overlay-body");
            Assert.True(Control(tiny, "overlay-close").GetProperty("focused").GetBoolean());
            Assert.Contains("Esc Close", Screen(tiny));

            await Resize(80, 24);
            Assert.Equal("presented", await Key("Enter"));
            var closed = await Request("snapshot");
            Assert.True(Control(closed, "overlay-open").GetProperty("focused").GetBoolean());
            Assert.DoesNotContain(closed.GetProperty("controls").EnumerateArray(),
                item => item.GetProperty("id").GetString() == "overlay-close");
        }

        Task<string?> Key(string key) => Turn(new() { ["kind"] = "key", ["key"] = key });
        Task<string?> Focus(string id) => Turn(new() { ["kind"] = "focus", ["control"] = id });

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
        static string Screen(JsonElement frame) =>
            string.Join('\n', frame.GetProperty("lines").EnumerateArray().Select(row => row.GetString()));
        static JsonElement Control(JsonElement frame, string id) =>
            Assert.Single(frame.GetProperty("controls").EnumerateArray(), item => item.GetProperty("id").GetString() == id);
        static string? Cell(JsonElement frame, int column, int row) =>
            frame.GetProperty("cells")[row * frame.GetProperty("columns").GetInt32() + column].GetProperty("text").GetString();
        static JsonElement CellStyle(JsonElement frame, int column, int row) =>
            frame.GetProperty("cells")[row * frame.GetProperty("columns").GetInt32() + column].GetProperty("colors");
        static JsonElement ControlStyle(JsonElement frame, string id)
        {
            var control = Control(frame, id);
            return CellStyle(frame, control.GetProperty("column").GetInt32(), control.GetProperty("row").GetInt32());
        }
    }
}
