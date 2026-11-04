using System.Diagnostics;
using System.Globalization;
using System.Text;
using System.Text.Json;

namespace Fpas.IntegrationTests.Debugging;

// A serial, bounded integration-test client. This is not a product debug frontend.
internal sealed class NetCoreDbgSession : IAsyncDisposable
{
    private readonly Process adapter;
    private Process? target;
    private readonly CancellationTokenSource deadline = new(TimeSpan.FromSeconds(45));
    private readonly StreamWriter trace;
    private readonly Task<string> stderr;
    private readonly List<JsonElement> events = [];
    private int sequence;
    private int receivedBytes;
    private int messages;
    private bool disconnected;

    internal NetCoreDbgSession(Debuggee program)
    {
        var path = Environment.GetEnvironmentVariable("FPAS_NETCOREDBG_PATH")
            ?? throw new InvalidOperationException("Missing FPAS_NETCOREDBG_PATH.");
        var start = new ProcessStartInfo(Path.GetFullPath(path))
        {
            UseShellExecute = false, RedirectStandardInput = true,
            RedirectStandardOutput = true, RedirectStandardError = true,
            WorkingDirectory = program.DirectoryPath
        };
        start.ArgumentList.Add("--interpreter=vscode");
        trace = new StreamWriter(program.FilePath("dap.log")) { AutoFlush = true };
        try { adapter = Process.Start(start) ?? throw new InvalidOperationException("Unable to start NetCoreDbg."); }
        catch { trace.Dispose(); deadline.Dispose(); throw; }
        stderr = ReadError();
    }

    internal async Task<JsonElement> Request(string command, object? arguments = null, bool expectSuccess = true)
    {
        var id = ++sequence;
        var text = JsonSerializer.Serialize(new { seq = id, type = "request", command, arguments = arguments ?? new { } });
        trace.WriteLine("> " + text);
        var payload = Encoding.UTF8.GetBytes(text);
        await adapter.StandardInput.BaseStream.WriteAsync(Encoding.ASCII.GetBytes($"Content-Length: {payload.Length}\r\n\r\n"), deadline.Token);
        await adapter.StandardInput.BaseStream.WriteAsync(payload, deadline.Token);
        await adapter.StandardInput.BaseStream.FlushAsync(deadline.Token);
        while (true)
        {
            var item = await Receive();
            if (item.GetProperty("type").GetString() == "event") { events.Add(item); continue; }
            Assert.Equal("response", item.GetProperty("type").GetString());
            Assert.Equal(id, item.GetProperty("request_seq").GetInt32());
            Assert.True(item.GetProperty("success").GetBoolean() == expectSuccess, item.GetRawText());
            return item.TryGetProperty("body", out var body) ? body : default;
        }
    }

    internal async Task<JsonElement> Event(string name)
    {
        while (true)
        {
            var index = events.FindIndex(item => item.GetProperty("event").GetString() == name);
            if (index >= 0)
            {
                var item = events[index]; events.RemoveAt(index);
                return item.TryGetProperty("body", out var body) ? body : default;
            }
            var message = await Receive();
            Assert.Equal("event", message.GetProperty("type").GetString());
            events.Add(message);
        }
    }

    internal async Task<(int Thread, JsonElement Pending)> Launch(Debuggee program, params int[] lines)
    {
        var pending = await Start(program, lines, stopAtEntry: true);
        var stopped = await Event("stopped");
        Assert.Equal("entry", stopped.GetProperty("reason").GetString());
        Assert.NotNull(target);
        return (stopped.GetProperty("threadId").GetInt32(), pending);
    }

    internal async Task<JsonElement> Start(Debuggee program, int[] lines, string[]? args = null, string? cwd = null, bool stopAtEntry = false)
    {
        await Request("initialize", new { adapterID = "coreclr", pathFormat = "path", linesStartAt1 = true, columnsStartAt1 = true });
        await Event("initialized");
        await Request("launch", new { program = program.Assembly, cwd = cwd ?? program.DirectoryPath, args = args ?? [], stopAtEntry, justMyCode = true });
        var pending = await Breakpoints(program.Source, lines);
        await Request("setExceptionBreakpoints", new { filters = new[] { "user-unhandled" } });
        await Request("configurationDone");
        return pending;
    }

    internal Task<JsonElement> Breakpoints(string source, params int[] lines) => Request("setBreakpoints",
        new { source = new { path = source }, breakpoints = lines.Select(line => new { line }).ToArray() });

    internal Task<JsonElement> Continue(int thread) => Request("continue", new { threadId = thread });

    internal async Task<(int Thread, JsonElement Frame)> Stop(string reason = "breakpoint")
    {
        var stopped = await Event("stopped");
        Assert.Equal(reason, stopped.GetProperty("reason").GetString());
        var thread = stopped.GetProperty("threadId").GetInt32();
        var frames = await Request("stackTrace", new { threadId = thread, startFrame = 0, levels = 8 });
        return (thread, frames.GetProperty("stackFrames")[0]);
    }

    internal async Task<JsonElement[]> Locals(JsonElement frame)
    {
        var scopes = await Request("scopes", new { frameId = frame.GetProperty("id").GetInt32() });
        var values = new List<JsonElement>();
        foreach (var scope in scopes.GetProperty("scopes").EnumerateArray())
            if (scope.GetProperty("variablesReference").GetInt32() is var reference && reference > 0)
                values.AddRange(await Variables(reference));
        return values.ToArray();
    }

    internal async Task<JsonElement[]> Variables(int reference)
    {
        var result = await Request("variables", new { variablesReference = reference, start = 0, count = 64 });
        return result.GetProperty("variables").EnumerateArray().ToArray();
    }

    internal async Task<string> Finish(int thread)
    {
        await Continue(thread);
        var exited = await Event("exited");
        Assert.Equal(0, exited.GetProperty("exitCode").GetInt32());
        await Event("terminated");
        var output = Output("stdout");
        await Disconnect();
        return output;
    }

    internal string Output(string category) => string.Concat(events.Where(item => item.GetProperty("event").GetString() == "output")
        .Select(item => item.GetProperty("body")).Where(body => body.GetProperty("category").GetString() == category)
        .Select(body => body.GetProperty("output").GetString())).Replace("\r\n", "\n", StringComparison.Ordinal);

    internal async Task Disconnect()
    {
        await Request("disconnect", new { terminateDebuggee = true });
        disconnected = true;
        adapter.StandardInput.Close();
        await adapter.WaitForExitAsync(deadline.Token);
        Assert.Equal(0, adapter.ExitCode);
        if (target is not null)
        {
            await target.WaitForExitAsync(deadline.Token);
            Assert.True(target.HasExited, "Launch-owned debuggee did not exit.");
        }
    }

    private async Task<JsonElement> Receive()
    {
        if (++messages > 4096) throw new InvalidDataException("DAP message limit exceeded.");
        var header = new StringBuilder();
        var one = new byte[1];
        while (!header.ToString().EndsWith("\r\n\r\n", StringComparison.Ordinal))
        {
            await adapter.StandardOutput.BaseStream.ReadExactlyAsync(one, deadline.Token);
            header.Append((char)one[0]);
            if (header.Length > 8192) throw new InvalidDataException("DAP header limit exceeded.");
        }
        var lengthHeader = header.ToString().Split("\r\n").Single(line => line.StartsWith("Content-Length:", StringComparison.OrdinalIgnoreCase));
        var length = int.Parse(lengthHeader[15..].Trim(), CultureInfo.InvariantCulture);
        if (length is < 0 or > 1_048_576 || (receivedBytes += length) > 8_388_608)
            throw new InvalidDataException("DAP payload limit exceeded.");
        var bytes = new byte[length];
        await adapter.StandardOutput.BaseStream.ReadExactlyAsync(bytes, deadline.Token);
        using var document = JsonDocument.Parse(bytes);
        var item = document.RootElement.Clone();
        trace.WriteLine("< " + item.GetRawText());
        if (item.TryGetProperty("event", out var name) && name.GetString() == "process")
        {
            target = Process.GetProcessById(item.GetProperty("body").GetProperty("systemProcessId").GetInt32());
            _ = target.Handle;
        }
        return item;
    }

    private async Task<string> ReadError()
    {
        var buffer = new char[4096]; var output = new StringBuilder(); int count;
        while ((count = await adapter.StandardError.ReadAsync(buffer, deadline.Token)) != 0)
        {
            if (output.Length + count > 65536) { deadline.Cancel(); throw new InvalidDataException("Adapter stderr limit exceeded."); }
            output.Append(buffer, 0, count);
        }
        return output.ToString();
    }

    public async ValueTask DisposeAsync()
    {
        try
        {
            if (!disconnected && !adapter.HasExited && !deadline.IsCancellationRequested)
                await Disconnect();
        }
        finally
        {
            try
            {
                if (target is not null && !target.HasExited) target.Kill(entireProcessTree: true);
                if (!adapter.HasExited) adapter.Kill(entireProcessTree: true);
                using var cleanup = new CancellationTokenSource(TimeSpan.FromSeconds(5));
                await adapter.WaitForExitAsync(cleanup.Token);
                if (target is not null) await target.WaitForExitAsync(cleanup.Token);
                try { trace.WriteLine("stderr: " + await stderr.WaitAsync(cleanup.Token)); }
                catch (OperationCanceledException) { }
            }
            finally { deadline.Cancel(); target?.Dispose(); adapter.Dispose(); deadline.Dispose(); trace.Dispose(); }
        }
    }
}
