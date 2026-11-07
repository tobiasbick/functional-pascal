using System.Buffers.Binary;
using System.IO.Pipes;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class DapDebuggerTests
{
    [Fact]
    public async Task Private_terminal_resizes_while_target_is_at_a_source_breakpoint()
    {
        using var program = new Debuggee("dap-private-terminal");
        await program.Compile("""
            program TerminalTarget;
            uses System.Console as Console, System.Threading.Thread as Thread;
            begin
              const Width: int := Console.WindowWidth;
              Console.WriteLine('before:' + Width.ToString() + 'x' + Console.WindowHeight.ToString());
              for Attempt: int := 1 to 100 do
                if Console.WindowWidth <> Width then break; end if;
                Thread.Sleep(20);
              end for;
              Console.WriteLine('after:' + Console.WindowWidth.ToString() + 'x' + Console.WindowHeight.ToString());
            end;
            end program;
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        var token = deadline.Token;
        var name = "fpas-terminal-" + Guid.NewGuid().ToString("N");
        await using var pipe = new NamedPipeServerStream(name, PipeDirection.InOut, 1, PipeTransmissionMode.Byte,
            PipeOptions.Asynchronous | PipeOptions.CurrentUserOnly);
        await using var client = new DapProbe();
        await client.Success("initialize");
        var launch = client.Success("launch", new()
        {
            ["program"] = program.Assembly,
            ["fpasTerminal"] = new JsonObject { ["pipe"] = name, ["columns"] = 80, ["rows"] = 24 }
        });
        await pipe.WaitForConnectionAsync(token);
        await launch;
        await client.Event("initialized");
        await client.Success("setBreakpoints", new()
        {
            ["source"] = new JsonObject { ["path"] = program.Source },
            ["breakpoints"] = new JsonArray(new JsonObject { ["line"] = 6 })
        });
        await client.Success("configurationDone");
        var stop = await client.Event("stopped");
        Assert.Equal("breakpoint", stop.GetProperty("reason").GetString());

        using var output = new MemoryStream();
        await ExpectOutput("before:80x24");
        Assert.DoesNotContain("after:", Encoding.UTF8.GetString(output.ToArray()));
        var resize = Encoding.UTF8.GetBytes("""{"id":1,"kind":"request","operation":"resize","payload":{"columns":53,"rows":17}}""");
        var header = new byte[4];
        BinaryPrimitives.WriteInt32BigEndian(header, resize.Length);
        await pipe.WriteAsync(header, token);
        await pipe.WriteAsync(resize, token);
        await pipe.FlushAsync(token);
        while (true)
        {
            var reply = await Read();
            if (reply.GetProperty("kind").GetString() == "reply")
            {
                Assert.Equal(1, reply.GetProperty("id").GetInt32());
                Assert.Equal("resize", reply.GetProperty("operation").GetString());
                Assert.False(reply.TryGetProperty("error", out var error) && error.ValueKind != JsonValueKind.Null, reply.ToString());
                break;
            }
            CopyOutput(reply);
        }
        await client.Success("continue", new() { ["threadId"] = stop.GetProperty("threadId").GetInt32() });
        await ExpectOutput("after:53x17");
        Assert.Equal(0, (await client.Event("exited")).GetProperty("exitCode").GetInt32());
        await client.Event("terminated");
        await client.Success("disconnect");
        await client.End();

        async Task<JsonElement> Read()
        {
            var length = new byte[4];
            await pipe.ReadExactlyAsync(length, token);
            var size = BinaryPrimitives.ReadInt32BigEndian(length);
            Assert.InRange(size, 1, 1_048_576);
            var bytes = new byte[size];
            await pipe.ReadExactlyAsync(bytes, token);
            using var document = JsonDocument.Parse(bytes);
            return document.RootElement.Clone();
        }

        void CopyOutput(JsonElement message)
        {
            Assert.Equal("output", message.GetProperty("operation").GetString());
            output.Write(message.GetProperty("payload").GetBytesFromBase64());
            Assert.InRange(output.Length, 1, 1_048_576);
        }

        async Task ExpectOutput(string expected)
        {
            while (!Encoding.UTF8.GetString(output.ToArray()).Contains(expected, StringComparison.Ordinal))
                CopyOutput(await Read());
        }
    }
}
