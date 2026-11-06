using System.Collections.Concurrent;
using System.Diagnostics;
using System.Globalization;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Threading.Channels;
using Fpas.Tests.Support;
using Fpas.IntegrationTests.Processes.Support;

namespace Fpas.IntegrationTests.Debugging;

// Independent stdio client: no product framing, DTOs or core handles are used.
internal sealed class DapProbe : IAsyncDisposable
{
    private readonly Process? process;
    private readonly InterruptibleProcess? console;
    private readonly Stream input, output;
    private readonly Task<string> error;
    private readonly CancellationTokenSource lifetime = new(TimeSpan.FromSeconds(45));
    private readonly SemaphoreSlim writes = new(1);
    private readonly ConcurrentDictionary<int, TaskCompletionSource<JsonElement>> pending = [];
    private readonly Channel<JsonElement> events = Channel.CreateBounded<JsonElement>(1024);
    private readonly Task reading;
    private int sequence;
    private TaskCompletionSource? readPause;
    internal void PauseReading() => readPause = new(TaskCreationOptions.RunContinuationsAsynchronously);
    internal void ResumeReading() { readPause?.TrySetResult(); readPause = null; }
    internal async Task Raw(string text)
    { await input.WriteAsync(Encoding.UTF8.GetBytes(text), lifetime.Token); await input.FlushAsync(lifetime.Token); }
    internal async Task<System.Text.Json.JsonElement[]> UntilTerminated()
    {
        var observed = new List<System.Text.Json.JsonElement>();
        await foreach (var item in events.Reader.ReadAllAsync(lifetime.Token))
        { observed.Add(item); if (item.GetProperty("event").GetString() == "terminated") return observed.ToArray(); }
        throw new IOException("Adapter EOF before terminated.");
    }

    internal DapProbe(string? cli = null, bool interruptible = false, bool withoutHostSearch = false)
    {
        var path = cli ?? TestEnvironment.Metadata("FpasCliPath");
        var managed = path.EndsWith(".dll", StringComparison.OrdinalIgnoreCase);
        var executable = managed ? TestEnvironment.DotnetHost : path;
        string[] prefix = managed ? [path] : [];
        if (interruptible)
        {
            console = InterruptibleProcess.Start(executable, [.. prefix, "debug", "--protocol", "dap"], Path.GetTempPath());
            input = console.Input; output = console.Output; error = new StreamReader(console.Error).ReadToEndAsync(lifetime.Token);
        }
        else
        {
            var start = new ProcessStartInfo(executable) { RedirectStandardInput = true, RedirectStandardOutput = true,
                RedirectStandardError = true, UseShellExecute = false, CreateNoWindow = true };
            if (withoutHostSearch)
            {
                foreach (var name in new[] { "DOTNET_HOST_PATH", "DOTNET_ROOT", "DOTNET_ROOT_X64" }) start.Environment.Remove(name);
                start.Environment["PATH"] = "";
            }
            foreach (var argument in prefix) start.ArgumentList.Add(argument);
            start.ArgumentList.Add("debug"); start.ArgumentList.Add("--protocol"); start.ArgumentList.Add("dap");
            process = Process.Start(start)!; input = process.StandardInput.BaseStream; output = process.StandardOutput.BaseStream;
            error = process.StandardError.ReadToEndAsync(lifetime.Token);
        }
        reading = Read();
    }

    internal async Task<JsonElement> Request(string command, JsonObject? arguments = null, bool split = false)
    {
        var id = Interlocked.Increment(ref sequence);
        var result = new TaskCompletionSource<JsonElement>(TaskCreationOptions.RunContinuationsAsynchronously); pending[id] = result;
        var body = Encoding.UTF8.GetBytes(new JsonObject { ["seq"] = id, ["type"] = "request", ["command"] = command, ["arguments"] = arguments ?? new() }.ToJsonString());
        var header = Encoding.ASCII.GetBytes($"Content-Length: {body.Length}\r\n\r\n");
        await writes.WaitAsync(lifetime.Token);
        try
        {
            if (split)
            {
                foreach (var value in header) { await input.WriteAsync(new[] { value }, lifetime.Token); await input.FlushAsync(lifetime.Token); }
                var middle = body.Length / 2;
                await input.WriteAsync(body.AsMemory(0, middle), lifetime.Token);
                await input.FlushAsync(lifetime.Token);
                await input.WriteAsync(body.AsMemory(middle), lifetime.Token);
            }
            else { await input.WriteAsync(header, lifetime.Token); await input.WriteAsync(body, lifetime.Token); }
            await input.FlushAsync(lifetime.Token);
        }
        finally { writes.Release(); }
        return await result.Task.WaitAsync(lifetime.Token);
    }

    internal async Task<JsonElement> Success(string command, JsonObject? arguments = null, bool split = false)
    {
        var response = await Request(command, arguments, split);
        Assert.True(response.GetProperty("success").GetBoolean(), response.ToString()); return response.GetProperty("body");
    }
    internal async Task<JsonElement> Event(string name)
    {
        await foreach (var item in events.Reader.ReadAllAsync(lifetime.Token))
            if (item.GetProperty("event").GetString() == name) return item.GetProperty("body");
        throw new IOException("DAP closed before event " + name);
    }
    internal async Task EndInput()
    {
        input.Close();
        await WaitForExit(); await reading;
        Assert.Equal("", await error);
    }
    internal Task Interrupt() => console!.Interrupt();
    private Task WaitForExit() => console?.WaitForExit(lifetime.Token) ?? process!.WaitForExitAsync(lifetime.Token);
    internal async Task End(int exitCode = 0)
    {
        await WaitForExit(); await reading;
        Assert.Equal(exitCode, console?.ExitCode ?? process!.ExitCode); Assert.Equal("", await error);
    }

    private async Task Read()
    {
        Exception? failure = null;
        try
        {
            var stream = output; var previous = 0; var single = new byte[1];
            while (true)
            {
                if (readPause is { } paused) await paused.Task.WaitAsync(lifetime.Token);
                var header = new List<byte>();
                while (true)
                {
                    if (await stream.ReadAsync(single, lifetime.Token) == 0)
                    { if (header.Count != 0) throw new IOException("Partial adapter header."); return; }
                    header.Add(single[0]);
                    Assert.True(header.Count <= 8192);
                    if (header.Count >= 4 && header.TakeLast(4).SequenceEqual(new byte[] { 13, 10, 13, 10 })) break;
                }
                var lengthLine = Encoding.ASCII.GetString(header.ToArray()).Split("\r\n").Single(line => line.StartsWith("Content-Length:", StringComparison.Ordinal));
                var length = int.Parse(lengthLine[15..].Trim(), CultureInfo.InvariantCulture); Assert.InRange(length, 1, 1_048_576);
                var bytes = new byte[length]; await stream.ReadExactlyAsync(bytes, lifetime.Token);
                using var json = JsonDocument.Parse(bytes); var root = json.RootElement.Clone();
                var current = root.GetProperty("seq").GetInt32(); Assert.Equal(previous + 1, current); previous = current;
                if (root.GetProperty("type").GetString() == "response")
                {
                    Assert.True(pending.TryRemove(root.GetProperty("request_seq").GetInt32(), out var response)); response!.TrySetResult(root);
                }
                else { Assert.Equal("event", root.GetProperty("type").GetString()); await events.Writer.WriteAsync(root, lifetime.Token); }
            }
        }
        catch (Exception exception) { failure = exception; }
        finally
        {
            foreach (var item in pending.Values) item.TrySetException(failure ?? new IOException("Adapter EOF."));
            events.Writer.TryComplete(failure);
        }
    }

    public async ValueTask DisposeAsync()
    {
        lifetime.Cancel();
        if (console is not null) await console.DisposeAsync();
        if (process is not null && !process.HasExited) { process.Kill(entireProcessTree: true); await process.WaitForExitAsync(); }
        try { await reading; } catch (Exception) { }
        process?.Dispose(); lifetime.Dispose(); writes.Dispose();
    }
}
