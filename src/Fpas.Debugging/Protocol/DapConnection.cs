using Fpas.Debugging.Processes;
using System.Collections.Concurrent;
using System.Globalization;
using System.Text;
using System.Text.Json;

namespace Fpas.Debugging.Protocol;

internal sealed class DapConnection : IAsyncDisposable
{
    private readonly OwnedBackend backend;
    private readonly Action<JsonElement> onEvent;
    private readonly Action<Exception> onFailure;
    private readonly CancellationTokenSource lifetime = new();
    private readonly ConcurrentDictionary<int, TaskCompletionSource<JsonElement>> pending = [];
    private readonly ConcurrentDictionary<int, Task> sends = [];
    private readonly SemaphoreSlim writer = new(1);
    private readonly object disposalGate = new();
    private readonly TaskCompletionSource requestsDrained = new(TaskCreationOptions.RunContinuationsAsynchronously);
    private Task? disposal;
    private readonly Task reader, errors, exit;
    private int sequence, disposed;
    internal TaskCompletionSource Initialized { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);
    internal JsonElement Capabilities { get; private set; }

    internal DapConnection(string path, string cwd, Action<JsonElement> onEvent, Action<Exception> onFailure)
    {
        this.onEvent = onEvent; this.onFailure = onFailure;
        backend = OwnedBackend.Start(path, cwd);
        reader = Read(); errors = DrainErrors(); exit = ObserveExit();
    }

    internal async Task Initialize(CancellationToken token)
    {
        Capabilities = await Request("initialize", new
        {
            adapterID = "coreclr", pathFormat = "path", linesStartAt1 = true, columnsStartAt1 = true,
            supportsVariableType = true, supportsVariablePaging = true, supportsRunInTerminalRequest = false
        }, token);
        await Initialized.Task.WaitAsync(TimeSpan.FromSeconds(30), token);
    }

    internal async Task<JsonElement> Request(string command, object arguments, CancellationToken token)
    {
        var id = Interlocked.Increment(ref sequence);
        var completion = new TaskCompletionSource<JsonElement>(TaskCreationOptions.RunContinuationsAsynchronously);
        lock (disposalGate)
        {
            if (disposed != 0 || lifetime.IsCancellationRequested) throw new DebugFailure("backend_exited", "The debugger connection is closed.");
            if (pending.Count >= 64) throw new DebugFailure("busy", "Too many pending debugger requests.");
            pending[id] = completion;
        }
        using var deadline = CancellationTokenSource.CreateLinkedTokenSource(token, lifetime.Token);
        deadline.CancelAfter(TimeSpan.FromSeconds(10));
        Task? sending = null;
        try
        {
            sending = Send(new { seq = id, type = "request", command, arguments }, deadline.Token);
            sends[id] = sending;
            // Anonymous Windows pipes can keep an OS write blocked after token cancellation.
            // The deadline must return control so session cleanup can close the owned peer.
            await sending.WaitAsync(deadline.Token);
            var result = await completion.Task.WaitAsync(deadline.Token);
            if (!result.GetProperty("success").GetBoolean())
                throw new DebugFailure("backend_error", result.TryGetProperty("message", out var message) ? message.GetString() ?? command : command);
            return result.TryGetProperty("body", out var body) ? body : JsonSerializer.SerializeToElement(new { });
        }
        catch (OperationCanceledException) when (!token.IsCancellationRequested)
        {
            throw new DebugFailure(lifetime.IsCancellationRequested ? "backend_exited" : "backend_timeout", "Debugger request did not complete: " + command);
        }
        catch (Exception exception) when (exception is IOException or ObjectDisposedException)
        {
            Fail(exception);
            throw new DebugFailure("backend_exited", "The debugger pipe closed during " + command + ".");
        }
        finally
        {
            lock (disposalGate)
            {
                pending.TryRemove(id, out _);
                if (sending?.IsCompleted == true) { _ = sending.Exception; sends.TryRemove(id, out _); }
                if (disposed != 0 && pending.IsEmpty) requestsDrained.TrySetResult();
            }
        }
    }

    private async Task Send(object message, CancellationToken token)
    {
        var bytes = JsonSerializer.SerializeToUtf8Bytes(message);
        if (bytes.Length > 1_048_576) throw new DebugFailure("limit_exceeded", "Debugger request exceeds 1 MiB.");
        await writer.WaitAsync(token);
        try
        {
            await backend.Input.WriteAsync(Encoding.ASCII.GetBytes($"Content-Length: {bytes.Length}\r\n\r\n"), token);
            await backend.Input.WriteAsync(bytes, token);
            await backend.Input.FlushAsync(token);
        }
        finally { writer.Release(); }
    }

    private async Task Read()
    {
        try
        {
            while (!lifetime.IsCancellationRequested)
            {
                var header = new StringBuilder(); var one = new byte[1];
                while (!header.ToString().EndsWith("\r\n\r\n", StringComparison.Ordinal))
                {
                    await backend.Output.ReadExactlyAsync(one, lifetime.Token);
                    if (one[0] > 127 || header.Length >= 8192) throw new InvalidDataException("Invalid debugger header.");
                    header.Append((char)one[0]);
                }
                var lengths = header.ToString().Split("\r\n").Where(line => line.StartsWith("Content-Length:", StringComparison.OrdinalIgnoreCase)).ToArray();
                if (lengths.Length != 1 || !int.TryParse(lengths[0][15..].Trim(), NumberStyles.None, CultureInfo.InvariantCulture, out var length)
                    || length is < 0 or > 1_048_576) throw new InvalidDataException("Invalid debugger frame length.");
                var bytes = new byte[length];
                await backend.Output.ReadExactlyAsync(bytes, lifetime.Token);
                using var document = JsonDocument.Parse(bytes);
                var item = document.RootElement.Clone();
                switch (item.GetProperty("type").GetString())
                {
                    case "response":
                        if (pending.TryGetValue(item.GetProperty("request_seq").GetInt32(), out var request)) request.TrySetResult(item);
                        break;
                    case "event":
                        if (item.GetProperty("event").GetString() == "initialized") Initialized.TrySetResult();
                        onEvent(item);
                        break;
                    case "request":
                        await Send(new { seq = Interlocked.Increment(ref sequence), type = "response", request_seq = item.GetProperty("seq").GetInt32(),
                            command = item.GetProperty("command").GetString(), success = false, message = "unsupported_capability: reverse request is unsupported." }, lifetime.Token);
                        break;
                    default: throw new InvalidDataException("Unknown debugger message type.");
                }
            }
        }
        catch (Exception exception)
        {
            Fail(exception);
        }
    }

    private async Task DrainErrors()
    {
        try
        {
            var bytes = new byte[4096]; var count = 0; int read;
            while ((read = await backend.Error.ReadAsync(bytes, lifetime.Token)) > 0)
                if ((count += read) > 65536) throw new InvalidDataException("Debugger diagnostic output limit exceeded.");
        }
        catch (Exception exception) { Fail(exception); }
    }

    private async Task ObserveExit()
    {
        try { await backend.WaitForExit(lifetime.Token); Fail(new EndOfStreamException("Debugger process exited.")); }
        catch (OperationCanceledException) { }
    }

    private void Fail(Exception exception)
    {
        if (lifetime.IsCancellationRequested) return;
        lifetime.Cancel();
        var failure = new DebugFailure("backend_exited", "The debugger connection ended.");
        Initialized.TrySetException(failure);
        foreach (var request in pending.Values) request.TrySetException(failure);
        onFailure(exception);
    }

    public ValueTask DisposeAsync()
    {
        lock (disposalGate) return new(disposal ??= DisposeCore());
    }

    private async Task DisposeCore()
    {
        disposed = 1;
        lifetime.Cancel();
        if (pending.IsEmpty) requestsDrained.TrySetResult();
        foreach (var request in pending.Values) request.TrySetException(new DebugFailure("backend_exited", "Debugger stopped."));
        try { await backend.DisposeAsync(); await Task.WhenAll(reader, errors, exit, DrainSends()).WaitAsync(TimeSpan.FromSeconds(5)); }
        finally { lifetime.Dispose(); writer.Dispose(); }
    }

    private async Task DrainSends()
    {
        await requestsDrained.Task;
        foreach (var sending in sends.Values)
        {
            try { await sending; }
            catch (Exception) { } // The request already reported cancellation/failure.
        }
        sends.Clear();
    }
}
