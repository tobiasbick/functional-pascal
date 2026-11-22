// [Documentation](../../coverage.md#terminal-ui)
using System.Diagnostics;
using System.IO.Pipes;
using System.Text.Json;
using System.Text.Json.Nodes;
using Fpas.IntegrationTests.Debugging;
using Fpas.Workspaces.Processes;
using static Fpas.IntegrationTests.Tui.TuiGallerySession;

namespace Fpas.IntegrationTests.Tui;

/// <summary>Verifies the public terminal gallery's panel layout and themes.</summary>
public sealed partial class TuiGalleryTests
{
    [Fact]
    public async Task Gallery_shows_live_examples_and_keeps_terminal_navigation_working()
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
        var session = new TuiGallerySession(pipe, token);
        try
        {
            await pipe.WaitForConnectionAsync(token);
            var initial = await Request("snapshot");
            AssertPanels(initial, 80, 25, 19);
            Assert.Equal("Overview", Selected(initial));
            Assert.Contains("Components for FPas", Preview(initial));
            Assert.Equal("Maya Okafor", Control(initial, "workspace-name").GetProperty("value").GetString());
            var dark = Colors(initial);

            await CheckKeyboardPreview(session);

            // Traverse the sidebar without changing the active page.
            string[] destinations = ["Overview", "Buttons", "Inputs", "Selection", "Navigation", "Overlays", "Feedback",
                "Data display", "Charts", "Pickers", "Disclosure", "Data grid", "Code", "Files",
                "Text and display", "App chrome"];
            foreach (var destination in destinations.Skip(1))
            {
                await Key("Down");
                Assert.Equal(destination, Focused(await Request("snapshot")));
            }
            Assert.Contains("Components for FPas", Preview(await Request("snapshot")));
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
            Assert.Contains("Selected Duplicate", Preview(chosen));
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
            Assert.Contains("We never share your email.", Preview(inputs));
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

            Assert.Contains("Search projects", Preview(inputs));
            await Click("Search");
            await Turn(new() { ["kind"] = "paste", ["text"] = "roadmap" });
            Assert.Equal("roadmap", Control(await Request("snapshot"), "input-search").GetProperty("value").GetString());
            await Key("Tab");
            Assert.Equal("Clear search", Focused(await Request("snapshot")));
            await Key("Enter");
            var cleared = await Request("snapshot");
            Assert.Equal("Search", Focused(cleared));
            Assert.Equal("", Control(cleared, "input-search").GetProperty("value").GetString());
            Assert.DoesNotContain(Controls(cleared), c => c.GetProperty("id").GetString() == "input-search/clear");

            await Click("Seats");
            Assert.Equal("3", Seats(await Request("snapshot")));
            await Turn(new() { ["kind"] = "key", ["key"] = "Up", ["modifiers"] = 1 });
            Assert.Equal("13", Seats(await Request("snapshot")));
            await Click("Decrease");
            Assert.Equal("12", Seats(await Request("snapshot")));
            await Click("Seats");
            await Turn(new() { ["kind"] = "key", ["key"] = "A", ["modifiers"] = 4 });
            await Turn(new() { ["kind"] = "paste", ["text"] = "-" });
            Assert.Equal("-", Seats(await Request("snapshot")));
            await Key("Tab");
            Assert.Equal("12", Seats(await Request("snapshot")));
            await Click("Seats");
            await Turn(new() { ["kind"] = "key", ["key"] = "A", ["modifiers"] = 4 });
            await Turn(new() { ["kind"] = "paste", ["text"] = "125" });
            await Key("Enter");
            var bounded = await Request("snapshot");
            Assert.Equal("99", Seats(bounded));
            Assert.False(Control(bounded, "input-seats/increase").GetProperty("enabled").GetBoolean());
            await Click("Decrease");
            await Click("Increase");
            Assert.Equal("Seats", Focused(await Request("snapshot")));
            await Click("Disclosure");
            var disclosure = await Request("snapshot");
            Assert.Equal("false", Control(disclosure, "security").GetProperty("value").GetString());
            Assert.Contains("Two-factor authentication is on", Preview(disclosure));
            await Key("Right");
            await Key("Enter");
            disclosure = await Request("snapshot");
            Assert.Equal("Security", Focused(disclosure));
            Assert.Equal("true", Control(disclosure, "security").GetProperty("value").GetString());
            Assert.Contains("Recovery codes were generated 3 months ago.", Preview(disclosure));
            await Click("Security");
            disclosure = await Request("snapshot");
            Assert.Equal("Security", Focused(disclosure));
            Assert.DoesNotContain("Recovery codes were generated", Preview(disclosure));

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
            Assert.Contains("Turns off all radios.", Preview(selection));
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
            Assert.Contains("Jonas commented on Release 1.0", Preview(sheet));
            Assert.Contains("Priya invited you to Engines", Preview(sheet));
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
            AssertPanels(await Request("snapshot"), 100, 28, 24);
            await Resize(40, 17);
            var narrow = await Request("snapshot");
            AssertPanels(narrow, 40, 17, 9);
            Assert.Equal(dark, Colors(narrow));
            await Key("End");
            Assert.Equal("App chrome", Focused(await Request("snapshot")));
            await Key("PageUp");
            Assert.NotEqual("App chrome", Focused(await Request("snapshot")));
            await Key("End");
            await Key("Up");
            await Key("Enter");
            Assert.Equal("Text and display", Selected(await Request("snapshot")));
            await Resize(100, 28);
            Assert.Contains("Components are functions.", Preview(await Request("snapshot")));
            await Resize(40, 17);

            await Key("Home");
            await Click("Feedback");
            await Turn(new() { ["kind"] = "scroll", ["column"] = 3, ["row"] = 4, ["delta"] = 40 });
            Assert.Contains(Controls(await Request("snapshot")), c => c.GetProperty("label").GetString() == "App chrome");
            await Turn(new() { ["kind"] = "scroll", ["column"] = 3, ["row"] = 4, ["delta"] = -40 });
            Assert.Equal("presented", await Turn(new() { ["kind"] = "key", ["key"] = "T", ["modifiers"] = 4 }));
            Assert.NotEqual(dark, Colors(await Request("snapshot")));
            await Turn(new() { ["kind"] = "key", ["key"] = "T", ["modifiers"] = 4 });
            await Resize(80, 25);
            var feedback = await Request("snapshot");
            AssertPanels(feedback, 80, 25, 19);
            Assert.Equal(dark, Colors(feedback));
            Assert.Matches(@"Uploading assets +─+ +68%", Preview(feedback));
            Assert.Matches(@"Exporting clip\.mp4 +─+ +42%", Preview(feedback));
            Assert.Contains("Syncing 24 files", Preview(feedback));
            foreach (string title in new[] { "A new version is available", "Payment received", "Storage almost full", "Build failed" })
                Assert.Contains(title, Preview(feedback));
            await Click("Dismiss Storage almost full");
            await Resize(120, 24);
            var dismissed = await Request("snapshot");
            Assert.DoesNotContain("Storage almost full", Preview(dismissed));
            Assert.Contains("Build failed", Preview(dismissed));
            await Click("Inputs");
            await Click("Feedback");
            Assert.Contains("Storage almost full", Preview(await Request("snapshot")));

            await Click("Data display");
            var people = await Request("snapshot");
            Assert.Contains("priya@example.com", Preview(people));
            Assert.Contains("Status", Preview(people));
            Assert.Equal("True", Control(people, "person-1").GetProperty("value").GetString());
            await Key("Right");
            Assert.Equal("Priya Nair", Focused(await Request("snapshot")));
            await Key("Down");
            await Key("Enter");
            Assert.Equal("True", Control(await Request("snapshot"), "person-2").GetProperty("value").GetString());
            await Click("Name");
            people = await Request("snapshot");
            Assert.Contains("Name ↓", Preview(people));
            Assert.True(Control(people, "person-2").GetProperty("row").GetInt32()
                < Control(people, "person-4").GetProperty("row").GetInt32());
            await Click("Aiko Tanaka");
            var aiko = await Request("snapshot");
            Assert.Equal("True", Control(aiko, "person-4").GetProperty("value").GetString());
            Assert.Contains("aiko@example.com", Preview(aiko));
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
            Assert.Contains("Priya Nair · Ship it", Preview(listFrame));
            await Turn(new() { ["kind"] = "click", ["column"] = list.GetProperty("column").GetInt32() + 2,
                ["row"] = list.GetProperty("row").GetInt32() });
            var review = await Request("snapshot");
            Assert.Equal("design-review", Control(review, "sample-list").GetProperty("value").GetString());
            Assert.Contains("Jonas Berg · Tomorrow at 10:00", Preview(review));
            Assert.DoesNotContain("Opened ", Preview(review));
            await Key("Enter");
            await Key("End");
            var planning = await Request("snapshot");
            Assert.Equal("planning", Control(planning, "sample-list").GetProperty("value").GetString());
            Assert.Contains("Tomas Rivera · Q1 roadmap", Preview(planning));
            Assert.Contains("Opened Design review", Preview(planning));
            await Resize(80, 25);

            await Key("Escape");
            await Key("End");
            await Key("Enter");
            Assert.Equal("App chrome", Selected(await Request("snapshot")));
            var chrome = await Request("snapshot");
            var encoding = Control(chrome, "desktop-status/encoding");
            var footerRow = encoding.GetProperty("row").GetInt32();
            var footer = chrome.GetProperty("lines")[footerRow].GetString()!;
            Assert.Matches(@"Saved +UTF-8 +Spaces: 4 +Markdown", footer);
            Assert.DoesNotContain(Controls(chrome), c => c.GetProperty("id").GetString() == "desktop-status/branch");
            Assert.Contains("────────", chrome.GetProperty("lines")[footerRow - 1].GetString());
            await Key("Right");
            Assert.Equal("UTF-8", Focused(await Request("snapshot")));
            await Key("Tab");
            await Key("Enter");
            Assert.Equal("Spaces: 8", Focused(await Request("snapshot")));
            await Click("Markdown");
            Assert.Equal("C", Focused(await Request("snapshot")));
            await Click("UTF-8");
            Assert.Equal("UTF-16", Focused(await Request("snapshot")));
            await Key("Enter");
            Assert.Equal("UTF-8", Focused(await Request("snapshot")));

            await Key("Escape");
            await Key("Up");
            await Key("Up");
            await Key("Enter");
            Assert.Equal("Files", Selected(await Request("snapshot")));
            await Key("Right");
            Assert.True(Control(await Request("snapshot"), "files-open").GetProperty("focused").GetBoolean());
            await Key("Enter");
            var filePicker = await Request("snapshot");
            Assert.True(Control(filePicker, "files-open-picker/list").GetProperty("focused").GetBoolean());
            foreach (var _ in Enumerable.Range(0, 4))
                await Key("Down");
            Assert.Equal("notes.txt", Control(await Request("snapshot"), "files-open-picker/list").GetProperty("value").GetString());
            await Key("Enter");
            Assert.Contains("notes.txt", Preview(await Request("snapshot")));
            await Key("Tab");
            await Key("Enter");
            var savePicker = await Request("snapshot");
            Assert.Equal("untitled.txt", Control(savePicker, "files-save-picker/name").GetProperty("value").GetString());
            await Key("Escape");
            Assert.True(Control(await Request("snapshot"), "files-save").GetProperty("focused").GetBoolean());

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

        Task<string?> Key(string key) => session.Key(key);
        Task Click(string label) => session.Click(label);
        Task Resize(int columns, int rows) => session.Resize(columns, rows);
        Task<string?> Turn(JsonObject input) => session.Turn(input);
        Task<JsonElement> Request(string operation, JsonObject? arguments = null) =>
            session.Request(operation, arguments);

        static string? Seats(JsonElement frame) => Control(frame, "input-seats").GetProperty("value").GetString();

    }
}
