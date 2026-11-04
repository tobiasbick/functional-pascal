using Fpas.Tests.Support;
using ModelContextProtocol.Client;
using ModelContextProtocol.Protocol;
using System.Diagnostics;
using System.Text.Json;

namespace Fpas.IntegrationTests.Debugging;

internal sealed class McpProbe : IAsyncDisposable
{
    private readonly CancellationTokenSource deadline = new(TimeSpan.FromSeconds(60));
    private readonly Process? server;
    internal McpClient Client { get; }
    internal CancellationToken Token => deadline.Token;
    private McpProbe(McpClient client, Process? server = null) { Client = client; this.server = server; }

    internal static async Task<McpProbe> Create(string? backend = null)
    {
        var transport = new StdioClientTransport(new()
        {
            Name = "FPAS test", Command = TestEnvironment.DotnetHost,
            Arguments = [TestEnvironment.Metadata("FpasCliPath"), "debug", "--netcoredbg", backend ?? Environment.GetEnvironmentVariable("FPAS_NETCOREDBG_PATH")!]
        });
        using var startup = new CancellationTokenSource(TimeSpan.FromSeconds(15));
        return new(await McpClient.CreateAsync(transport, cancellationToken: startup.Token));
    }

    internal static async Task<McpProbe> ObservedServer(string? backend = null)
    {
        var start = new ProcessStartInfo(TestEnvironment.DotnetHost) { UseShellExecute = false, RedirectStandardInput = true, RedirectStandardOutput = true, RedirectStandardError = true };
        foreach (var argument in new[] { TestEnvironment.Metadata("FpasCliPath"), "debug", "--netcoredbg", backend ?? Environment.GetEnvironmentVariable("FPAS_NETCOREDBG_PATH")! }) start.ArgumentList.Add(argument);
        var process = Process.Start(start)!;
        try
        {
            using var startup = new CancellationTokenSource(TimeSpan.FromSeconds(15));
            return new(await McpClient.CreateAsync(new StreamClientTransport(process.StandardInput.BaseStream, process.StandardOutput.BaseStream), cancellationToken: startup.Token), process);
        }
        catch { if (!process.HasExited) process.Kill(true); process.Dispose(); throw; }
    }

    internal async Task<JsonElement> Call(string tool, object arguments, string? error = null)
    {
        var values = JsonSerializer.SerializeToElement(arguments).EnumerateObject().ToDictionary(property => property.Name, property => (object?)property.Value.Clone());
        var result = await Client.CallToolAsync(tool, values, cancellationToken: Token);
        var body = result.StructuredContent ?? throw new InvalidDataException(string.Join(';', result.Content.OfType<TextContentBlock>().Select(block => block.Text)));
        Assert.True((result.IsError == true) == (error is not null), body.GetRawText());
        if (error is not null) Assert.Equal(error, body.GetProperty("error").GetProperty("code").GetString());
        return body;
    }

    internal async Task<JsonElement> State(string id, string state)
    {
        var status = await Call("debug_status", new { sessionId = id });
        while (status.GetProperty("state").GetString() != state)
        {
            Assert.NotEqual("failed", status.GetProperty("state").GetString());
            status = await Call("debug_status", new { sessionId = id, afterEvent = status.GetProperty("event").GetInt64(), waitMilliseconds = 1000 });
        }
        return status;
    }

    internal async Task<JsonElement> Stopped(string id, long previous)
    {
        var status = await State(id, "stopped");
        while (status.GetProperty("stopId").GetInt64() == previous)
            status = await Call("debug_status", new { sessionId = id, afterEvent = status.GetProperty("event").GetInt64(), waitMilliseconds = 1000 });
        Assert.Equal("stopped", status.GetProperty("state").GetString()); return status;
    }

    internal async Task<JsonElement[]> Locals(string id, JsonElement stop)
    {
        var stopId = stop.GetProperty("stopId").GetInt64();
        var stack = await Call("debug_stack", new { sessionId = id, stopId });
        var frame = stack.GetProperty("data").GetProperty("frames")[0].GetProperty("handle").GetString();
        var scopes = await Call("debug_variables", new { sessionId = id, stopId, frameHandle = frame });
        var handle = scopes.GetProperty("data").GetProperty("scopes")[0].GetProperty("handle").GetString();
        var values = await Call("debug_variables", new { sessionId = id, stopId, variablesHandle = handle });
        return values.GetProperty("data").GetProperty("variables").EnumerateArray().ToArray();
    }

    internal Task<JsonElement> Continue(string id, JsonElement stop)
        => Call("debug_continue", new { sessionId = id, stopId = stop.GetProperty("stopId").GetInt64() });

    internal async Task<JsonElement[]> Expand(string id, JsonElement stop, JsonElement variable)
    {
        var result = await Call("debug_variables", new { sessionId = id, stopId = stop.GetProperty("stopId").GetInt64(), variablesHandle = variable.GetProperty("handle").GetString() });
        return result.GetProperty("data").GetProperty("variables").EnumerateArray().ToArray();
    }

    public async ValueTask DisposeAsync()
    {
        try
        {
            await Client.DisposeAsync();
            if (server is not null)
            {
                server.StandardInput.Close();
                using var shutdown = new CancellationTokenSource(TimeSpan.FromSeconds(12));
                await server.WaitForExitAsync(shutdown.Token);
                Assert.Equal(0, server.ExitCode);
            }
        }
        finally
        {
            if (server is not null) { if (!server.HasExited) server.Kill(true); server.Dispose(); }
            deadline.Dispose();
        }
    }
}
