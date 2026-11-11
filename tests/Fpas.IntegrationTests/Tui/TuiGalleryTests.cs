using System.Buffers.Binary;
using System.Diagnostics;
using System.IO.Pipes;
using System.Text.Json;
using System.Text.Json.Nodes;
using Fpas.IntegrationTests.Debugging;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Tui;

/// <summary>Verifies the public terminal gallery's supported pages, layout and interactions.</summary>
public sealed class TuiGalleryTests
{
    [Fact]
    public async Task Gallery_presents_the_confirmed_widgets_at_desktop_and_drawer_sizes()
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
            await Resize(134, 38);
            var wide = await Request("snapshot");
            Assert.Contains("FPas CUI", Screen(wide));
            Assert.Contains("Components for FPAS", Screen(wide));
            var pages = wide.GetProperty("controls").EnumerateArray()
                .Where(item => item.GetProperty("role").GetString() == "navigation").ToArray();
            Assert.Equal(new[] { "pages/overview", "pages/buttons", "pages/overlays", "pages/actions" },
                pages.Select(item => item.GetProperty("id").GetString()));
            Assert.InRange(pages[0].GetProperty("row").GetInt32(), 0, 5);
            Assert.Equal("true", pages[0].GetProperty("value").GetString());
            Assert.Equal("true", Control(wide, "theme/dark").GetProperty("value").GetString());
            Assert.Equal(pages[0].GetProperty("row").GetInt32() + 2, pages[2].GetProperty("row").GetInt32());

            Assert.Equal("presented", await Focus("pages/buttons"));
            Assert.Equal("presented", await Key("Enter"));
            var buttons = await Request("snapshot");
            Assert.Equal("true", Control(buttons, "pages/buttons").GetProperty("value").GetString());
            Assert.False(Control(buttons, "disabled").GetProperty("enabled").GetBoolean());
            Assert.Contains("Variants", Screen(buttons));
            Assert.Contains("Sizes and icons", Screen(buttons));
            Assert.Contains("Groups, copy and hold to confirm", Screen(buttons));

            await Choose("save");
            Assert.Equal("loading", Control(await Request("snapshot"), "save").GetProperty("value").GetString());
            Assert.Equal("unchanged", await Turn(new() { ["kind"] = "activate", ["control"] = "save" }));
            await Choose("finish-save");
            await Choose("format-bold");
            await Choose("format-italic");
            var group = await Request("snapshot");
            Assert.Equal("true", Control(group, "format-bold").GetProperty("value").GetString());
            Assert.Equal("true", Control(group, "format-italic").GetProperty("value").GetString());

            Assert.Contains("npm i cui", Screen(group));
            Assert.Equal("copy-button", Control(group, "copy-install").GetProperty("role").GetString());
            Assert.False(Control(group, "copy-install").GetProperty("enabled").GetBoolean());

            var hold = Control(group, "hold-delete");
            Assert.Equal("hold-button", hold.GetProperty("role").GetString());
            int column = hold.GetProperty("column").GetInt32();
            int row = hold.GetProperty("row").GetInt32();
            Assert.Equal("presented", await Focus("hold-delete"));
            Assert.Equal("unchanged", await Key("Enter"));
            await Point("click", column, row);
            await Point("press", column, row);
            var earlyRelease = new JsonObject { ["kind"] = "release", ["column"] = column, ["row"] = row };
            Assert.Equal("queued", Status(await Request("send", new() { ["input"] = earlyRelease })));
            await Task.Delay(1300, token);
            await Request("step");
            Assert.DoesNotContain("Confirmed:", Screen(await Request("snapshot")));

            await Point("press", column, row);
            await Task.Delay(1300, token);
            await Request("step");
            var filled = await Request("snapshot");
            Assert.Equal("100", Control(filled, "hold-delete").GetProperty("value").GetString());
            Assert.DoesNotContain("Confirmed:", Screen(filled));
            await Point("move", 0, 0);
            await Point("release", column, row);
            Assert.DoesNotContain("Confirmed:", Screen(await Request("snapshot")));

            await Point("press", column, row);
            await Task.Delay(1300, token);
            await Point("release", column, row);
            await Point("release", column, row);
            Assert.Contains("Confirmed: 1", Screen(await Request("snapshot")));

            Assert.Equal("presented", await Turn(new()
                { ["kind"] = "scroll", ["column"] = 100, ["row"] = 30, ["delta"] = 100 }));
            await Choose("view/1");
            Assert.Equal("true", Control(await Request("snapshot"), "view/1").GetProperty("value").GetString());
            await Focus("range/0");
            await Key("Right");
            var segments = await Request("snapshot");
            Assert.Equal("true", Control(segments, "range/1").GetProperty("value").GetString());
            Assert.Equal("true", Control(segments, "view/1").GetProperty("value").GetString());

            await Focus("options");
            Assert.Equal("presented", await Key("Down"));
            var dropdown = await Request("snapshot");
            Assert.True(Control(dropdown, "options/rename").GetProperty("focused").GetBoolean());
            Assert.Contains("Project", Screen(dropdown));
            Assert.Contains("Ctrl D", Screen(dropdown));
            await Key("Down");
            await Key("Enter");
            Assert.Equal("false", Control(await Request("snapshot"), "options").GetProperty("value").GetString());
            await Focus("more-actions");
            await Key("Down");
            Assert.True(Control(await Request("snapshot"), "more-actions/archive").GetProperty("focused").GetBoolean());
            await Key("Escape");

            await Choose("pages/actions");
            await Choose("split-save");
            Assert.Contains("Saved", Screen(await Request("snapshot")));
            await Focus("split-save/menu");
            Assert.Equal("presented", await Key("Down"));
            var menu = await Request("snapshot");
            Assert.True(Control(menu, "split-save/menu/draft").GetProperty("focused").GetBoolean());
            Assert.False(Control(menu, "split-save").GetProperty("enabled").GetBoolean());
            await Key("Down");
            Assert.True(Control(await Request("snapshot"), "split-save/menu/publish").GetProperty("focused").GetBoolean());
            await Key("Enter");
            var chosen = await Request("snapshot");
            Assert.Contains("Save and publish", Screen(chosen));
            Assert.True(Control(chosen, "split-save/menu").GetProperty("focused").GetBoolean());
            Assert.DoesNotContain(chosen.GetProperty("controls").EnumerateArray(),
                item => item.GetProperty("id").GetString() == "split-save/menu/draft");
            await Key("Enter");
            await Key("Escape");
            Assert.True(Control(await Request("snapshot"), "split-save/menu").GetProperty("focused").GetBoolean());
            await Focus("split-export/menu");
            await Key("Down");
            await Choose("split-export/menu/schedule");
            Assert.Contains("Schedule…", Screen(await Request("snapshot")));
            await Focus("split-save/menu");
            await Key("Down");
            await Point("click", 0, 0);
            Assert.Equal("false", Control(await Request("snapshot"), "split-save/menu").GetProperty("value").GetString());
            await Focus("confirm-delete");
            await Key("Enter");
            var armed = await Request("snapshot");
            Assert.Equal("armed", Control(armed, "confirm-delete").GetProperty("value").GetString());
            Assert.True(Control(armed, "confirm-delete").GetProperty("focused").GetBoolean());
            Assert.Contains("Confirmed: 0", Screen(armed));
            await Key("Enter");
            var confirmed = await Request("snapshot");
            Assert.Equal("", Control(confirmed, "confirm-delete").GetProperty("value").GetString());
            Assert.Contains("Confirmed: 1", Screen(confirmed));
            await Key("Enter");
            await Key("Escape");
            Assert.Equal("", Control(await Request("snapshot"), "confirm-delete").GetProperty("value").GetString());
            await Focus("status-save");
            await Key("Enter");
            var busy = await Request("snapshot");
            Assert.Equal("busy", Control(busy, "status-save").GetProperty("value").GetString());
            Assert.True(Control(busy, "status-save").GetProperty("focused").GetBoolean());
            await Key("Enter");
            Assert.Contains("Requests: 1", Screen(await Request("snapshot")));
            await Choose("status-success");
            var success = await Request("snapshot");
            Assert.Equal("success", Control(success, "status-save").GetProperty("value").GetString());
            Assert.Equal("Saved", Control(success, "status-save").GetProperty("label").GetString());
            await Focus("status-save");
            await Key("Enter");
            Assert.Contains("Requests: 1", Screen(await Request("snapshot")));
            await Choose("status-reset");
            await Choose("status-save");
            await Choose("status-error");
            var error = await Request("snapshot");
            Assert.Equal("error", Control(error, "status-save").GetProperty("value").GetString());
            await Focus("status-save");
            await Key("Enter");
            Assert.Contains("Requests: 2", Screen(await Request("snapshot")));
            await Choose("theme-preview/light");
            var light = await Request("snapshot");
            Assert.Equal("true", Control(light, "theme/light").GetProperty("value").GetString());
            await Focus("theme-preview/light");
            await Key("Right");
            var dark = await Request("snapshot");
            Assert.Equal("true", Control(dark, "theme/dark").GetProperty("value").GetString());
            Assert.True(Control(dark, "theme-preview/dark").GetProperty("focused").GetBoolean());
            await Key("Right");
            await Key("Right");
            var system = await Request("snapshot");
            Assert.Equal("true", Control(system, "theme/system").GetProperty("value").GetString());
            Assert.True(Control(system, "theme-preview/system").GetProperty("focused").GetBoolean());
            await Choose("system-appearance");
            Assert.Equal("true", Control(await Request("snapshot"), "theme-preview/system").GetProperty("value").GetString());
            await Focus("theme-toggle");
            await Key("Enter");
            var toggled = await Request("snapshot");
            Assert.Equal("true", Control(toggled, "theme-preview/light").GetProperty("value").GetString());
            Assert.True(Control(toggled, "theme-toggle").GetProperty("focused").GetBoolean());
            Assert.Equal("light", Control(toggled, "theme-toggle").GetProperty("value").GetString());
            await Key("Enter");
            Assert.Equal("true", Control(await Request("snapshot"), "theme/dark").GetProperty("value").GetString());
            Assert.Equal("unchanged", await Turn(new() { ["kind"] = "activate", ["control"] = "theme/dark" }));
            await Choose("pages/buttons");

            await Resize(40, 17);
            var narrow = await Request("snapshot");
            Assert.DoesNotContain(narrow.GetProperty("controls").EnumerateArray(),
                item => item.GetProperty("id").GetString() == "pages/buttons");
            await Focus("drawer-open");
            await Choose("drawer-open");
            var drawer = await Request("snapshot");
            Assert.True(Control(drawer, "drawer/close").GetProperty("focused").GetBoolean());
            Assert.False(Control(drawer, "drawer-open").GetProperty("enabled").GetBoolean());
            Assert.Equal("presented", await Key("Escape"));
            Assert.True(Control(await Request("snapshot"), "drawer-open").GetProperty("focused").GetBoolean());

            await Choose("drawer-open");
            await Choose("pages/overlays");
            await Focus("sheet-open");
            await Choose("sheet-open");
            var sheet = await Request("snapshot");
            Assert.True(Control(sheet, "sheet/close").GetProperty("focused").GetBoolean());
            Assert.Contains("Notifications", Screen(sheet));
            Assert.Equal("presented", await Key("Escape"));
            Assert.True(Control(await Request("snapshot"), "sheet-open").GetProperty("focused").GetBoolean());

            await Resize(134, 38);
            await Choose("theme/light");
            await Choose("pages/buttons");
            var restored = await Request("snapshot");
            Assert.Equal("true", Control(restored, "format-bold").GetProperty("value").GetString());
            Assert.NotEqual("loading", Control(restored, "save").GetProperty("value").GetString());

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

        async Task Choose(string id) =>
            Assert.Equal("presented", await Turn(new() { ["kind"] = "activate", ["control"] = id }));

        Task<string?> Point(string kind, int column, int row) =>
            Turn(new() { ["kind"] = kind, ["column"] = column, ["row"] = row });

        async Task Resize(int columns, int rows)
        {
            Assert.Equal("queued", Status(await Request("resize", new() { ["columns"] = columns, ["rows"] = rows })));
            Assert.Equal("presented", Status(await Request("step")));
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
    }
}
