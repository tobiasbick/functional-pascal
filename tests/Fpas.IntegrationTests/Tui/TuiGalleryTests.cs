// [Documentation](../../coverage.md#terminal-ui)
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
    public async Task Gallery_uses_GUI_sections_and_sample_data_and_keeps_terminal_navigation_working()
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
            Assert.Contains("Components for FPas", Preview(initial, 26));
            Assert.Equal("Maya Okafor", Control(initial, "workspace-name").GetProperty("value").GetString());
            var dark = Colors(initial);

            // Traverse the sidebar without changing the active page. The terminal extension follows both GUI groups.
            string[] destinations = ["Overview", "Buttons", "Inputs", "Selection", "Navigation", "Overlays", "Feedback",
                "Data display", "Charts", "Actions", "Smart inputs", "Pickers", "Capture", "Disclosure", "Reader",
                "Data grid", "Code", "Social and media", "Planning", "More charts", "Motion", "AI", "Audio", "Files",
                "Developer tools", "More developer tools", "Instruments", "Creative controls", "Canvas and bars",
                "Voice", "Layout", "Text and display", "Editing tools", "App chrome", "Cell grid"];
            foreach (var destination in destinations.Skip(1))
            {
                await Key("Down");
                Assert.Equal(destination, Focused(await Request("snapshot")));
            }
            Assert.Contains("Components for FPas", Preview(await Request("snapshot"), 26));
            await Key("Home");
            Assert.Equal("Overview", Selected(await Request("snapshot")));
            await Resize(144, 36);
            var workspace = await Request("snapshot");
            Assert.Equal("false", Control(workspace, "workspace-summary").GetProperty("value").GetString());
            await Click("Email me a weekly summary");
            await Click("Browse components");
            var buttons = await Request("snapshot");
            Assert.Equal("Buttons", Selected(buttons));
            foreach (var id in new[] { "primary", "secondary", "outline", "ghost", "danger", "link", "small", "medium",
                "large", "new-project", "download", "continue", "save", "finish-save", "disabled", "selected" })
                Assert.Equal("button", Control(buttons, id).GetProperty("role").GetString());
            Assert.False(Control(buttons, "disabled").GetProperty("enabled").GetBoolean());
            await Click("Save changes");
            Assert.Equal("Saving", Control(await Request("snapshot"), "save").GetProperty("label").GetString());
            await Click("Reset");

            await Turn(new() { ["kind"] = "scroll", ["column"] = 68, ["row"] = 4, ["delta"] = 30 });
            await Click("Options");
            var menu = await Request("snapshot");
            Assert.Equal("Rename", Focused(menu));
            Assert.Equal("menu-item", Control(menu, "options/rename").GetProperty("role").GetString());
            await Key("Down");
            await Key("Enter");
            var chosen = await Request("snapshot");
            Assert.Equal("Options", Focused(chosen));
            Assert.Contains("Selected Duplicate", Preview(chosen, 48));
            await Key("Down");
            await Key("End");
            await Key("Enter");
            Assert.Equal("Cancel", Focused(await Request("snapshot")));
            await Key("Escape");
            Assert.Equal("Options", Focused(await Request("snapshot")));

            await Click("Inputs");
            await Key("Right");
            Assert.Equal("Full name", Focused(await Request("snapshot")));
            await Key("End");
            await Turn(new() { ["kind"] = "paste", ["text"] = "!" });
            var inputs = await Request("snapshot");
            Assert.Contains("We never share your email.", Preview(inputs, 48));
            Assert.Equal("Maya Okafor!", Control(inputs, "input-name").GetProperty("value").GetString());
            var nameField = Control(inputs, "input-name");
            var emailField = Control(inputs, "input-email");
            Assert.Equal("maya@example.com", emailField.GetProperty("value").GetString());
            Assert.Equal(nameField.GetProperty("row").GetInt32(), emailField.GetProperty("row").GetInt32());
            Assert.True(nameField.GetProperty("column").GetInt32() < emailField.GetProperty("column").GetInt32());
            await Resize(80, 24);
            var stacked = await Request("snapshot");
            Assert.True(Control(stacked, "input-name").GetProperty("row").GetInt32()
                < Control(stacked, "input-email").GetProperty("row").GetInt32());
            Assert.True(Control(stacked, "input-about").GetProperty("row").GetInt32()
                > Control(stacked, "input-email").GetProperty("row").GetInt32());
            await Turn(new() { ["kind"] = "scroll", ["column"] = 40, ["row"] = 3, ["delta"] = 30 });
            var stackedPickers = await Request("snapshot");
            Assert.True(Control(stackedPickers, "input-country").GetProperty("row").GetInt32()
                < Control(stackedPickers, "input-language").GetProperty("row").GetInt32());
            await Resize(144, 36);

            var about = Control(inputs, "input-about");
            Assert.Equal("text-area", about.GetProperty("role").GetString());
            Assert.StartsWith("Mathematician and writer.", about.GetProperty("value").GetString());
            Assert.Equal(6, about.GetProperty("height").GetInt32());
            await Click("About");
            await Turn(new() { ["kind"] = "key", ["key"] = "End", ["modifiers"] = 4 });
            await Key("Enter");
            await Turn(new() { ["kind"] = "paste", ["text"] = "A second line." });
            Assert.EndsWith("\nA second line.", Control(await Request("snapshot"), "input-about").GetProperty("value").GetString());

            Assert.Equal("Germany", Control(inputs, "input-country").GetProperty("value").GetString());
            Assert.Equal("Pick a language", Control(inputs, "input-language").GetProperty("value").GetString());
            Assert.Equal(Control(inputs, "input-country").GetProperty("row").GetInt32(),
                Control(inputs, "input-language").GetProperty("row").GetInt32());
            await Click("Country");
            Assert.Equal("Germany", Focused(await Request("snapshot")));
            await Key("Down");
            await Key("Escape");
            Assert.Equal("Country", Focused(await Request("snapshot")));
            await Click("Language");
            await Turn(new() { ["kind"] = "paste", ["text"] = "RU" });
            Assert.Equal("Rust", Focused(await Request("snapshot")));
            await Key("Enter");
            Assert.Equal("Rust", Control(await Request("snapshot"), "input-language").GetProperty("value").GetString());

            await Click("Disclosure");
            var disclosure = await Request("snapshot");
            Assert.Equal("false", Control(disclosure, "security").GetProperty("value").GetString());
            Assert.Contains("Two-factor authentication is on", Preview(disclosure, 48));
            await Key("Right");
            await Key("Enter");
            disclosure = await Request("snapshot");
            Assert.Equal("Security", Focused(disclosure));
            Assert.Equal("true", Control(disclosure, "security").GetProperty("value").GetString());
            Assert.Contains("Recovery codes were generated 3 months ago.", Preview(disclosure, 48));
            await Click("Security");
            disclosure = await Request("snapshot");
            Assert.Equal("Security", Focused(disclosure));
            Assert.DoesNotContain("Recovery codes were generated", Preview(disclosure, 48));

            await Click("Navigation");
            await Key("Right");
            Assert.Equal("Overview", Focused(await Request("snapshot")));
            await Key("Right");
            var navigation = await Request("snapshot");
            Assert.Equal("Analytics", Focused(navigation));
            Assert.Equal("true", Control(navigation, "tabs/1").GetProperty("value").GetString());
            await Key("Tab");
            Assert.Equal("Account", Focused(await Request("snapshot")));
            await Click("Billing");
            Assert.Equal("true", Control(await Request("snapshot"), "pill-tabs/2").GetProperty("value").GetString());
            await Resize(44, 22);
            await Key("Escape");
            await Key("Right");
            await Key("End");
            navigation = await Request("snapshot");
            Assert.Equal("Settings", Focused(navigation));
            Assert.Equal("true", Control(navigation, "tabs/3").GetProperty("value").GetString());
            await Resize(144, 36);

            await Click("Selection");
            await Key("Right");
            var selection = await Request("snapshot");
            Assert.Equal("Email notifications", Focused(selection));
            var newsletter = Control(selection, "newsletter");
            var wifi = Control(selection, "wifi");
            Assert.Equal("true", newsletter.GetProperty("value").GetString());
            Assert.Equal(newsletter.GetProperty("row").GetInt32(), wifi.GetProperty("row").GetInt32());
            Assert.True(newsletter.GetProperty("column").GetInt32() < wifi.GetProperty("column").GetInt32());
            Assert.False(Control(selection, "disabled-option").GetProperty("enabled").GetBoolean());
            await Turn(new() { ["kind"] = "character", ["text"] = " " });
            await Key("Tab");
            Assert.Equal("Marketing", Focused(await Request("snapshot")));
            await Key("Enter");
            await Key("Tab");
            Assert.Equal("Wi-Fi", Focused(await Request("snapshot")));
            await Resize(80, 24);
            var narrowSelection = await Request("snapshot");
            Assert.True(Control(narrowSelection, "disabled-option").GetProperty("row").GetInt32()
                < Control(narrowSelection, "wifi").GetProperty("row").GetInt32());
            await Resize(144, 36);
            Assert.Equal("true", Control(selection, "wifi").GetProperty("value").GetString());
            Assert.Equal("false", Control(selection, "bluetooth").GetProperty("value").GetString());
            Assert.Contains("Turns off all radios.", Preview(selection, 48));
            await Turn(new() { ["kind"] = "character", ["text"] = " " });
            await Key("Tab");
            Assert.Equal("Bluetooth", Focused(await Request("snapshot")));
            await Key("Enter");

            await Key("Tab");
            Assert.Equal("Airplane mode", Focused(await Request("snapshot")));
            await Key("Tab");
            Assert.Equal("Every 5 minutes", Focused(await Request("snapshot")));
            await Key("Right");
            var frequency = await Request("snapshot");
            Assert.Equal("Only when I save", Focused(frequency));
            Assert.Equal("true", Control(frequency, "save-frequency/2").GetProperty("value").GetString());

            await Click("Overlays");
            await Click("Edit profile");
            var profile = await Request("snapshot");
            Assert.Equal("Name", Focused(profile));
            Assert.Equal("Maya Okafor!", Control(profile, "edit-profile-name").GetProperty("value").GetString());
            await Key("End");
            await Key("Backspace");
            Assert.Equal("false", Control(profile, "edit-profile-public").GetProperty("value").GetString());
            await Click("Public profile");
            await Click("Save changes");
            await Click("Open sheet");
            var sheet = await Request("snapshot");
            Assert.Contains("Jonas commented on Release 1.0", Preview(sheet, 48));
            Assert.Contains("Priya invited you to Engines", Preview(sheet, 48));
            Assert.False(Control(sheet, "navigation/Overview").GetProperty("enabled").GetBoolean());
            Assert.Equal("true", Control(sheet, "notifications-mentions").GetProperty("value").GetString());
            Assert.Equal("true", Control(sheet, "notifications-weekly").GetProperty("value").GetString());
            Assert.Equal("false", Control(sheet, "notifications-comments").GetProperty("value").GetString());
            await Click("Weekly digest");
            await Click("Comments");
            await Key("Escape");
            await Click("Selection");
            var changed = await Request("snapshot");
            Assert.Equal("true", Control(changed, "save-frequency/2").GetProperty("value").GetString());
            Assert.Equal("true", Control(changed, "newsletter").GetProperty("value").GetString());
            Assert.Equal("true", Control(changed, "marketing").GetProperty("value").GetString());
            await Click("Overview");
            Assert.Equal("true", Control(await Request("snapshot"), "workspace-summary").GetProperty("value").GetString());
            Assert.Equal("true", Control(changed, "wifi").GetProperty("value").GetString());
            Assert.Equal("false", Control(changed, "bluetooth").GetProperty("value").GetString());
            await Click("Overlays");
            await Click("Open sheet");
            Assert.Equal("true", Control(await Request("snapshot"), "notifications-comments").GetProperty("value").GetString());
            await Key("Escape");
            Assert.Equal("Open sheet", Focused(await Request("snapshot")));
            await Click("Delete project");
            var dialog = await Request("snapshot");
            Assert.Equal("Cancel", Focused(dialog));
            AssertCenteredDialog(dialog);
            await Resize(40, 12);
            AssertCenteredDialog(await Request("snapshot"));
            await Resize(144, 36);
            await Turn(new() { ["kind"] = "click", ["button"] = "left", ["column"] = 1, ["row"] = 0 });
            Assert.Equal("Delete project", Focused(await Request("snapshot")));
            await Key("Enter");
            await Key("Tab");
            await Key("Enter");
            await Key("Escape");
            await Resize(100, 28);
            AssertPanels(await Request("snapshot"), 100, 28, 33);
            await Resize(40, 17);
            var narrow = await Request("snapshot");
            AssertPanels(narrow, 40, 17, 13);
            Assert.Equal(dark, Colors(narrow));
            await Key("End");
            Assert.Equal("Cell grid", Focused(await Request("snapshot")));
            await Key("PageUp");
            Assert.NotEqual("Cell grid", Focused(await Request("snapshot")));
            await Key("End");
            await Key("Up");
            await Key("Up");
            await Key("Up");
            await Key("Enter");
            Assert.Equal("Text and display", Selected(await Request("snapshot")));
            await Resize(100, 28);
            Assert.Contains("Components are functions.", Preview(await Request("snapshot"), 33));
            await Resize(40, 17);

            await Key("Home");
            await Click("Feedback");
            await Turn(new() { ["kind"] = "scroll", ["column"] = 3, ["row"] = 4, ["delta"] = 40 });
            Assert.Contains(Controls(await Request("snapshot")), c => c.GetProperty("label").GetString() == "App chrome");
            await Turn(new() { ["kind"] = "scroll", ["column"] = 3, ["row"] = 4, ["delta"] = -40 });
            Assert.Equal("presented", await Turn(new() { ["kind"] = "key", ["key"] = "T", ["modifiers"] = 4 }));
            Assert.NotEqual(dark, Colors(await Request("snapshot")));
            await Turn(new() { ["kind"] = "key", ["key"] = "T", ["modifiers"] = 4 });
            await Resize(80, 24);
            var feedback = await Request("snapshot");
            AssertPanels(feedback, 80, 24, 26);
            Assert.Equal(dark, Colors(feedback));
            Assert.Matches(@"Uploading assets +─+ +68%", Preview(feedback, 26));
            Assert.Matches(@"Exporting clip\.mp4 +─+ +42%", Preview(feedback, 26));

            await Resize(120, 24);
            await Click("Data display");
            var people = await Request("snapshot");
            Assert.Contains("aiko@example.com", Preview(people, 40));
            Assert.Equal("True", Control(people, "person-1").GetProperty("value").GetString());
            await Key("Right");
            Assert.Equal("Priya Nair", Focused(await Request("snapshot")));
            await Key("Down");
            await Key("Enter");
            Assert.Equal("True", Control(await Request("snapshot"), "person-2").GetProperty("value").GetString());
            await Click("Name");
            people = await Request("snapshot");
            Assert.Contains("Name ↓", Preview(people, 40));
            Assert.True(Control(people, "person-2").GetProperty("row").GetInt32()
                < Control(people, "person-4").GetProperty("row").GetInt32());
            await Click("Aiko Tanaka");
            Assert.Equal("True", Control(await Request("snapshot"), "person-4").GetProperty("value").GetString());
            await Resize(120, 10);
            people = await Request("snapshot");
            var headerRow = Control(people, "people/header/0").GetProperty("row").GetInt32();
            var firstRow = Control(people, "person-2");
            await Turn(new() { ["kind"] = "scroll", ["column"] = firstRow.GetProperty("column").GetInt32(),
                ["row"] = firstRow.GetProperty("row").GetInt32(), ["delta"] = 10 });
            people = await Request("snapshot");
            Assert.Equal(headerRow, Control(people, "people/header/0").GetProperty("row").GetInt32());
            Assert.DoesNotContain(Controls(people), c => c.GetProperty("id").GetString() == "person-2");
            await Click("Aiko Tanaka");
            await Key("Escape");
            Assert.Equal("Data display", Focused(await Request("snapshot")));
            await Turn(new() { ["kind"] = "scroll", ["column"] = firstRow.GetProperty("column").GetInt32(),
                ["row"] = firstRow.GetProperty("row").GetInt32(), ["delta"] = -10 });
            await Key("Right");
            Assert.Equal("Aiko Tanaka", Focused(await Request("snapshot")));
            await Key("Home");
            Assert.Equal("Tomas Rivera", Focused(await Request("snapshot")));
            await Resize(120, 24);
            var listFrame = await Request("snapshot");
            var list = Control(listFrame, "sample-list");
            Assert.Equal("listbox", list.GetProperty("role").GetString());
            Assert.Equal("release", list.GetProperty("value").GetString());
            Assert.Contains("Priya Nair · Ship it", Preview(listFrame, 40));
            await Turn(new() { ["kind"] = "click", ["column"] = list.GetProperty("column").GetInt32() + 2,
                ["row"] = list.GetProperty("row").GetInt32() });
            var review = await Request("snapshot");
            Assert.Equal("design-review", Control(review, "sample-list").GetProperty("value").GetString());
            Assert.Contains("Jonas Berg · Tomorrow at 10:00", Preview(review, 40));
            Assert.DoesNotContain("Opened ", Preview(review, 40));
            await Key("Enter");
            await Key("End");
            var planning = await Request("snapshot");
            Assert.Equal("planning", Control(planning, "sample-list").GetProperty("value").GetString());
            Assert.Contains("Tomas Rivera · Q1 roadmap", Preview(planning, 40));
            Assert.Contains("Opened Design review", Preview(planning, 40));
            await Resize(80, 24);

            await Key("Escape");
            await Key("End");
            await Key("Enter");
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

            await Key("Escape");
            await Key("End");
            await Key("Up");
            await Key("Enter");
            Assert.Equal("App chrome", Selected(await Request("snapshot")));
            await Resize(156, 30);
            var chrome = await Request("snapshot");
            var encoding = Control(chrome, "desktop-status/encoding");
            var footerRow = encoding.GetProperty("row").GetInt32();
            Assert.InRange(footerRow, 26, 28);
            var footer = chrome.GetProperty("lines")[footerRow].GetString()!;
            Assert.Matches(@"main +Saved +Indexing 62% +─{8} +Ln 48, Col 17 +Spaces: 4 +UTF-8 +Markdown +Notifications", footer);
            Assert.Contains("────────", chrome.GetProperty("lines")[footerRow - 1].GetString());
            await Click("Spaces: 4");
            Assert.Equal("Spaces: 8", Focused(await Request("snapshot")));
            await Click("Markdown");
            Assert.Equal("C", Focused(await Request("snapshot")));
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
            var step = await Request("step");
            Assert.True(Status(step) != "failed", step.ToString());
            return Status(step);
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

    private static JsonElement Control(JsonElement frame, string id) =>
        Controls(frame).Single(c => c.GetProperty("id").GetString() == id);

    private static void AssertCenteredDialog(JsonElement frame)
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

    private static string? Focused(JsonElement frame) =>
        Controls(frame).Single(c => c.GetProperty("focused").GetBoolean()).GetProperty("label").GetString();

    private static string? Selected(JsonElement frame) =>
        Controls(frame).Single(c => c.GetProperty("role").GetString() == "navigation" && c.GetProperty("value").GetString() == "true").GetProperty("label").GetString();

    private static string Preview(JsonElement frame, int leftWidth) => string.Join("\n",
        frame.GetProperty("lines").EnumerateArray().Select(line => line.GetString()![(leftWidth + 2)..^1]));

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
