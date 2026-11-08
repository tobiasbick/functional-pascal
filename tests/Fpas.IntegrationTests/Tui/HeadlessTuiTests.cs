using System.Buffers.Binary;
using System.Diagnostics;
using System.IO.Pipes;
using System.Text.Json;
using System.Text.Json.Nodes;
using Fpas.IntegrationTests.Debugging;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Tui;

public sealed class HeadlessTuiTests
{
    [Theory]
    [InlineData("headless.fpas", "headless contracts verified\n")]
    [InlineData("canvas-input.fpas", "canvas input contracts verified\n")]
    [InlineData("effects.fpas", "effect contracts verified\n")]
    public async Task Headless_host_preserves_scheduling_focus_frame_and_effect_contracts(string fixture, string expected)
    {
        using var program = new Debuggee("headless-tui");
        await TuiFixture.Compile(program, fixture);
        Assert.Equal(expected, await program.Run());
    }

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
            Assert.False(Control(first, "button-disabled").GetProperty("enabled").GetBoolean());
            var queued = await Activate("button-press");
            Assert.Equal("queued", Status(queued));
            var wait = new JsonObject { ["inputId"] = queued.GetProperty("inputId").GetInt64(), ["timeoutMs"] = 0 };
            Assert.Equal("timeout", Status(await Request("wait", wait)));
            Assert.Equal(initialId, (await Request("snapshot")).GetProperty("id").GetInt64());
            Assert.Equal("presented", Status(await Request("step")));
            Assert.Equal("presented", Status(await Request("wait", wait)));
            Assert.Contains("Presses: 1", Screen(await Request("snapshot")));
            Assert.Equal("queued", Status(await Activate("button-disabled")));
            Assert.Equal("unchanged", Status(await Request("step")));

            await Choose("pages/lists");
            await Choose("list-empty");
            Assert.Contains("No items.", Screen(await Request("snapshot")));
            await Choose("list-empty");
            Assert.Equal("birch", Control(await Request("snapshot"), "sample-list").GetProperty("value").GetString());
            Assert.Equal("presented", await Key("Tab"));
            Assert.True(Control(await Request("snapshot"), "sample-list").GetProperty("focused").GetBoolean());

            Assert.Equal("presented", await Key("Down"));
            var disabled = await Request("snapshot");
            Assert.Equal("cedar", Control(disabled, "sample-list").GetProperty("value").GetString());
            await Key("Enter");
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

            await Choose("theme-blue");
            var themed = await Request("snapshot");
            Assert.Equal(0x121B32, themed.GetProperty("cells")[0].GetProperty("style").GetProperty("background").GetInt32());
            Assert.Equal("amber", Control(themed, "sample-list").GetProperty("value").GetString());
            Assert.Contains("▸ Amber", Screen(themed));
            Assert.True(Control(themed, "theme-blue").GetProperty("focused").GetBoolean());

            await CheckLog();

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

            await Choose("sample-scroll");
            Assert.Equal("presented", await Key("Home"));
            await Choose("log-append");
            var paused = await Request("snapshot");
            Assert.Contains("Paused", Screen(paused));
            Assert.Contains("Event 01", Screen(paused));
            Assert.DoesNotContain("Event 11", Screen(paused));
            Assert.Equal("0", Control(paused, "sample-scroll").GetProperty("value").GetString());
            Assert.Equal("█", Cell(paused, bar, row));

            await Choose("sample-scroll");
            Assert.Equal("presented", await Key("PageDown"));
            Assert.DoesNotContain("Event 01", Screen(await Request("snapshot")));
            Assert.Equal("presented", await Turn(new() { ["kind"] = "scroll", ["column"] = column, ["row"] = row, ["delta"] = -20 }));
            Assert.Contains("Event 01", Screen(await Request("snapshot")));
            Assert.Equal("presented", await Key("End"));
            Assert.Contains("Event 11", Screen(await Request("snapshot")));
            await Choose("log-append");
            Assert.Contains("Event 12", Screen(await Request("snapshot")));
        }

        Task<string?> Key(string key) => Turn(new() { ["kind"] = "key", ["key"] = key });

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
    }
}
