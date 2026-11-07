using System.Buffers.Binary;
using System.Collections.Concurrent;
using System.IO.Pipes;
using System.Text.Json;
using System.Text.Json.Nodes;
using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;
using Fpas.Processes.Terminal;

namespace Fpas.Debugging.Dap;

internal sealed class TuiController : IAsyncDisposable
{
    private readonly object gate = new();
    private readonly NamedPipeServerStream terminal, control;
    private readonly WorkerChannel terminalChannel;
    private readonly CancellationTokenSource lifetime = new();
    private readonly SemaphoreSlim controlWriter = new(1);
    private readonly Task terminalReader, controlReader;
    private readonly TaskCompletionSource terminalConnected = Signal(), controlConnected = Signal();
    private readonly ConcurrentDictionary<long, TaskCompletionSource<WorkerMessage>> terminalPending = [];
    private readonly ConcurrentDictionary<long, TaskCompletionSource<JsonObject>> controlPending = [];
    private readonly TerminalDisplay screen;
    private readonly Queue<JsonElement> history = new();
    private TaskCompletionSource changed = Signal();
    private JsonElement? frame;
    private JsonElement? pendingWork;
    private long sequence;
    internal TerminalLaunch Launch { get; }

    internal TuiController(int columns, int rows)
    {
        var terminalName = "fpas-terminal-" + Guid.NewGuid().ToString("N");
        var controlName = "fpas-tui-" + Guid.NewGuid().ToString("N");
        terminal = Server(terminalName);
        control = Server(controlName);
        terminalChannel = new(terminal);
        screen = new(columns, rows);
        Launch = new(terminalName, columns, rows, controlName);
        terminalReader = ReadTerminal();
        controlReader = ReadControl();
    }

    private static NamedPipeServerStream Server(string name) => new(name, PipeDirection.InOut, 1, PipeTransmissionMode.Byte,
        PipeOptions.CurrentUserOnly | PipeOptions.Asynchronous);
    private static TaskCompletionSource Signal() => new(TaskCreationOptions.RunContinuationsAsynchronously);

    internal JsonObject Snapshot(bool paused, bool cells)
    {
        lock (gate)
        {
            var copy = frame is { } complete ? JsonNode.Parse(complete.GetRawText())!.AsObject() : null;
            if (copy is not null && !cells) copy.Remove("cells");
            var result = new JsonObject
            {
                ["paused"] = paused,
                ["frame"] = copy,
                ["pendingWork"] = pendingWork is { } pending ? JsonNode.Parse(pending.GetRawText()) : null,
                ["history"] = new JsonArray(history.Select(outcome => JsonNode.Parse(outcome.GetRawText())).ToArray()),
                ["terminalLines"] = new JsonArray(screen.Lines().Select(line => (JsonNode?)JsonValue.Create(line)).ToArray())
            };
            if (JsonSerializer.SerializeToUtf8Bytes(result, DapJson.Default.JsonObject).Length > DapTransport.MaximumBytes - 4096)
                throw new DebugException("limit_exceeded", "The TUI snapshot exceeds the DAP response limit.");
            return result;
        }
    }

    internal Task<JsonObject> WaitFrame(JsonObject args, CancellationToken token)
    {
        var condition = JsonSerializer.SerializeToElement(args.Object("frame"), DapJson.Default.JsonObject);
        return WaitFor(() => frame is { } complete && TuiControlConnection.Matches(complete, condition)
            ? new() { ["status"] = "matched", ["frameId"] = complete.GetProperty("id").GetInt64() }
            : null, args.Number("timeoutMs", 5000), () => "timeout", token);
    }

    internal Task<JsonObject> WaitInput(long id, int timeout, Func<bool> paused, CancellationToken token)
    {
        if (id <= 0) throw DapArguments.Invalid("inputId");
        return WaitFor(() =>
        {
            foreach (var outcome in history)
                if (outcome.GetProperty("inputId").GetInt64() == id)
                    return JsonNode.Parse(outcome.GetRawText())!.AsObject();
            return null;
        }, timeout, () => paused() ? "paused" : "timeout", token);
    }

    private async Task<JsonObject> WaitFor(Func<JsonObject?> observe, int timeout, Func<string> timedOut, CancellationToken token)
    {
        if (timeout is < 0 or > 60_000) throw DapArguments.Invalid("timeoutMs");
        using var deadline = CancellationTokenSource.CreateLinkedTokenSource(token);
        deadline.CancelAfter(timeout);
        try
        {
            while (true)
            {
                Task update;
                lock (gate)
                {
                    if (observe() is { } result) return result;
                    update = changed.Task;
                }
                await update.WaitAsync(deadline.Token);
            }
        }
        catch (OperationCanceledException) when (!token.IsCancellationRequested)
        { return new() { ["status"] = timedOut() }; }
    }

    internal async Task<JsonObject> Forward(string operation, JsonObject arguments, CancellationToken token)
    {
        await controlConnected.Task.WaitAsync(token);
        var id = Interlocked.Increment(ref sequence);
        var reply = new TaskCompletionSource<JsonObject>(TaskCreationOptions.RunContinuationsAsynchronously);
        controlPending[id] = reply;
        try
        {
            var request = arguments.DeepClone().AsObject();
            request["operation"] = operation;
            request["id"] = id;
            var bytes = JsonSerializer.SerializeToUtf8Bytes(request, DapJson.Default.JsonObject);
            if (bytes.Length > 65_536) throw new ArgumentException("TUI requests are bounded at 64 KiB.");
            var header = new byte[4];
            BinaryPrimitives.WriteInt32BigEndian(header, bytes.Length);
            await controlWriter.WaitAsync(token);
            try { await control.WriteAsync(header, token); await control.WriteAsync(bytes, token); await control.FlushAsync(token); }
            finally { controlWriter.Release(); }
            return await reply.Task.WaitAsync(token);
        }
        finally { controlPending.TryRemove(id, out _); }
    }

    internal async Task Resize(int columns, int rows, CancellationToken token)
    {
        if (columns is < 1 or > 500 || rows is < 1 or > 300) throw DapArguments.Invalid("terminal dimensions");
        await TerminalRequest("resize", JsonSerializer.SerializeToElement(new TerminalSize(columns, rows), WorkerJson.Default.TerminalSize), token);
        lock (gate) screen.Resize(columns, rows);
    }

    internal Task WriteTerminal(byte[] bytes, CancellationToken token)
        => TerminalRequest("input", JsonSerializer.SerializeToElement(bytes, WorkerJson.Default.ByteArray), token);

    private async Task TerminalRequest(string operation, JsonElement payload, CancellationToken token)
    {
        await terminalConnected.Task.WaitAsync(token);
        var id = Interlocked.Increment(ref sequence);
        var reply = new TaskCompletionSource<WorkerMessage>(TaskCreationOptions.RunContinuationsAsynchronously);
        terminalPending[id] = reply;
        try
        {
            await terminalChannel.Send(new(id, "request", operation, payload), token);
            var result = await reply.Task.WaitAsync(token);
            if (result.Error is { } error) throw new DebugException(error.Code, error.Message);
        }
        finally { terminalPending.TryRemove(id, out _); }
    }

    private async Task ReadTerminal()
    {
        try
        {
            await terminal.WaitForConnectionAsync(lifetime.Token);
            terminalConnected.TrySetResult();
            while (await terminalChannel.Read(lifetime.Token) is { } message)
            {
                if (message.Kind == "event" && message.Operation == "output" && message.Payload is { } output)
                { lock (gate) screen.Read(output.GetBytesFromBase64()); }
                else if (message.Kind == "reply" && terminalPending.TryRemove(message.Id, out var request))
                    request.TrySetResult(message);
            }
        }
        catch (Exception error) when (error is IOException or OperationCanceledException or ObjectDisposedException or JsonException) { }
        finally
        {
            var error = new IOException("The private terminal connection closed.");
            terminalConnected.TrySetException(error);
            foreach (var request in terminalPending.Values) request.TrySetException(error);
        }
    }

    private async Task ReadControl()
    {
        try
        {
            await control.WaitForConnectionAsync(lifetime.Token);
            controlConnected.TrySetResult();
            while (true)
            {
                var header = new byte[4];
                if (await control.ReadAsync(header.AsMemory(0, 1), lifetime.Token) == 0) break;
                await control.ReadExactlyAsync(header.AsMemory(1), lifetime.Token);
                var length = BinaryPrimitives.ReadInt32BigEndian(header);
                if (length is < 1 or > 16_777_216) throw new IOException("Invalid TUI message length.");
                var bytes = new byte[length];
                await control.ReadExactlyAsync(bytes, lifetime.Token);
                using var document = JsonDocument.Parse(bytes);
                var message = document.RootElement;
                switch (message.GetProperty("kind").GetString())
                {
                    case "frame" when message.GetProperty("version").GetInt32() == 1:
                        lock (gate)
                        {
                            frame = message.GetProperty("frame").Clone();
                            changed.TrySetResult(); changed = Signal();
                        }
                        break;
                    case "reply":
                        var id = message.GetProperty("id").GetInt64();
                        if (controlPending.TryRemove(id, out var request))
                            request.TrySetResult(JsonNode.Parse(message.GetProperty("result").GetRawText())!.AsObject());
                        break;
                    case "outcome" when message.GetProperty("version").GetInt32() == 1:
                        lock (gate)
                        {
                            pendingWork = message.GetProperty("pending").Clone();
                            history.Enqueue(message.GetProperty("outcome").Clone());
                            if (history.Count > 64) history.Dequeue();
                            changed.TrySetResult(); changed = Signal();
                        }
                        break;
                }
            }
        }
        catch (Exception error) when (error is IOException or OperationCanceledException or ObjectDisposedException or JsonException) { }
        finally
        {
            var error = new IOException("The TUI control connection closed.");
            controlConnected.TrySetException(error);
            foreach (var request in controlPending.Values) request.TrySetException(error);
        }
    }

    public async ValueTask DisposeAsync()
    {
        lifetime.Cancel();
        try { await Task.WhenAll(terminalReader, controlReader); }
        finally
        {
            await terminalChannel.DisposeAsync();
            await control.DisposeAsync();
            controlWriter.Dispose(); lifetime.Dispose();
        }
    }
}
