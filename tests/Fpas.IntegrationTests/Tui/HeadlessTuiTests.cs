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
            await Choose("sample-list/cedar");
            await Choose("theme-blue");
            var themed = await Request("snapshot");
            Assert.Equal(0x121B32, themed.GetProperty("cells")[0].GetProperty("style").GetProperty("background").GetInt32());
            Assert.Equal("true", Control(themed, "sample-list/cedar").GetProperty("value").GetString());
            Assert.True(Control(themed, "theme-blue").GetProperty("focused").GetBoolean());
            await Choose("list-empty");
            Assert.Contains("No items.", Screen(await Request("snapshot")));
            await Choose("list-empty");
            Assert.Equal("true", Control(await Request("snapshot"), "sample-list/cedar").GetProperty("value").GetString());

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
