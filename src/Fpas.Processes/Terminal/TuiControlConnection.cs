using System.Buffers.Binary;
using System.IO.Pipes;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Processes.Terminal;

// The controller owns the listening pipe and retains frames independently of the target.
public sealed class TuiControlConnection : IDisposable
{
    private static readonly JsonSerializerOptions Json = new(JsonSerializerDefaults.Web);
    private readonly NamedPipeClientStream pipe;
    private readonly CancellationTokenSource lifetime = new();
    private readonly CancellationTokenSource stopReading = new();
    private readonly SemaphoreSlim writer = new(1);
    private readonly Task requests;
    private int disposed;

    private TuiControlConnection(NamedPipeClientStream pipe, Func<JsonElement, Task<object>> handle)
    {
        this.pipe = pipe;
        requests = Task.Run(() => Read(handle));
    }

    public static TuiControlConnection? Open(Func<TerminalInput, TuiControlReply> send, Func<object> snapshot,
        Func<TuiControlReply> step, Func<long, int, Task<TuiControlReply>> waitInput,
        Func<CancellationToken, Task<bool>> waitWork, Func<CancellationToken, Task<bool>> waitCapacity, bool manual)
    {
        return Open(async request =>
        {
            switch (request.GetProperty("operation").GetString())
            {
                case "send": return send(ReadInput(request.GetProperty("input")));
                case "resize":
                    if (!manual) return new TuiControlReply("native_resize_required");
                    return send(new("resize", Column: request.GetProperty("columns").GetInt32(), Row: request.GetProperty("rows").GetInt32()));
                case "snapshot": return snapshot();
                case "step": return manual ? step() : new TuiControlReply("interactive");
                case "wait":
                    var timeout = request.GetProperty("timeoutMs").GetInt32();
                    if (timeout is < 0 or > 60_000) throw new ArgumentOutOfRangeException(nameof(timeout));
                    if (request.TryGetProperty("inputId", out var id)) return await waitInput(id.GetInt64(), timeout);
                    if (request.TryGetProperty("frame", out var condition))
                    {
                        using var deadline = new CancellationTokenSource(timeout);
                        try
                        {
                            do
                            {
                                var frame = JsonSerializer.SerializeToElement(snapshot(), Json);
                                if (Matches(frame, condition)) return new TuiControlReply("matched", FrameId: frame.GetProperty("id").GetInt64());
                                await Task.Delay(5, deadline.Token);
                            } while (true);
                        }
                        catch (OperationCanceledException) { return new TuiControlReply("timeout"); }
                    }
                    using (var deadline = new CancellationTokenSource(timeout))
                    {
                        try
                        {
                            var ready = await (request.GetProperty("queue").GetString() switch
                            {
                                "work" => waitWork(deadline.Token), "capacity" => waitCapacity(deadline.Token),
                                _ => throw new ArgumentException("Choose work or capacity for a queue wait.")
                            });
                            return new TuiControlReply(ready ? "ready" : "closed");
                        }
                        catch (OperationCanceledException) { return new TuiControlReply("timeout"); }
                    }
                default: throw new ArgumentException("Unknown TUI operation.");
            }
        });
    }

    private static TerminalInput ReadInput(JsonElement value)
    {
        return value.GetProperty("kind").GetString() switch
        {
            "activate" => new("activate", Text: value.GetProperty("control").GetString() ?? ""),
            "text" => new("text", Text: value.GetProperty("text").GetString() ?? ""),
            "character" => new("character", Text: value.GetProperty("text").GetString() ?? ""),
            "paste" => new("paste", Text: value.GetProperty("text").GetString() ?? ""),
            "key" => new("key", value.GetProperty("key").GetString() ?? "",
                value.TryGetProperty("modifiers", out var modifiers) ? modifiers.GetInt32() : 0),
            "click" or "move" or "press" or "release" => new(value.GetProperty("kind").GetString()!,
                Column: value.GetProperty("column").GetInt32(), Row: value.GetProperty("row").GetInt32(),
                Button: value.TryGetProperty("button", out var button) ? button.GetString() ?? "" : "left"),
            "scroll" => new("scroll", Column: value.GetProperty("column").GetInt32(), Row: value.GetProperty("row").GetInt32(),
                Delta: value.GetProperty("delta").GetInt32()),
            _ => throw new ArgumentException("Unknown TUI input kind.")
        };
    }

    public static bool Matches(JsonElement frame, JsonElement condition)
    {
        foreach (var property in condition.EnumerateObject())
        {
            switch (property.Name)
            {
                case "columns" or "rows":
                    if (frame.GetProperty(property.Name).GetInt32() != property.Value.GetInt32()) return false;
                    break;
                case "after":
                    if (frame.GetProperty("id").GetInt64() <= property.Value.GetInt64()) return false;
                    break;
                case "text":
                    if (!frame.GetProperty("lines").EnumerateArray().Any(line => line.GetString()!.Contains(property.Value.GetString()!, StringComparison.Ordinal)))
                        return false;
                    break;
                case "control":
                    if (!frame.GetProperty("controls").EnumerateArray().Any(control => control.GetProperty("id").GetString() == property.Value.GetString()))
                        return false;
                    break;
                default: throw new ArgumentException("Unknown frame wait condition.");
            }
        }
        return true;
    }

    private static TuiControlConnection? Open(Func<JsonElement, Task<object>> handle)
    {
        var name = Environment.GetEnvironmentVariable("FPAS_TUI_CONTROL_PIPE");
        if (string.IsNullOrEmpty(name)) return null;
        const string prefix = "fpas-tui-";
        if (!name.StartsWith(prefix, StringComparison.Ordinal) || !Guid.TryParseExact(name[prefix.Length..], "N", out _))
            throw new ArgumentException("FPAS_TUI_CONTROL_PIPE requires fpas-tui- followed by a GUID in N format.");
        var pipe = new NamedPipeClientStream(".", name, PipeDirection.InOut, PipeOptions.Asynchronous | PipeOptions.CurrentUserOnly);
        try
        {
            pipe.Connect(5000);
            return new(pipe, handle);
        }
        catch { pipe.Dispose(); throw; }
    }

    public void Publish(object frame) => Notify(new { kind = "frame", version = 1, frame });

    public void PublishOutcome(TuiControlReply outcome, int inputs, int background)
        => Notify(new { kind = "outcome", version = 1, outcome, pending = new { inputs, background } });

    private void Notify(object value)
    {
        if (lifetime.IsCancellationRequested) return;
        try { Send(value).GetAwaiter().GetResult(); }
        catch (Exception error) when (error is IOException or OperationCanceledException or ObjectDisposedException)
        { lifetime.Cancel(); }
    }

    private async Task Read(Func<JsonElement, Task<object>> handle)
    {
        var pending = new List<Task>();
        using var reading = CancellationTokenSource.CreateLinkedTokenSource(lifetime.Token, stopReading.Token);
        try
        {
            while (!reading.IsCancellationRequested)
            {
                var header = new byte[4];
                if (await pipe.ReadAsync(header.AsMemory(0, 1), reading.Token) == 0) break;
                await pipe.ReadExactlyAsync(header.AsMemory(1), reading.Token);
                var length = BinaryPrimitives.ReadInt32BigEndian(header);
                if (length is < 1 or > 65_536) throw new IOException("Invalid TUI request length.");
                var buffer = new byte[length];
                await pipe.ReadExactlyAsync(buffer, reading.Token);
                using var document = JsonDocument.Parse(buffer);
                var request = document.RootElement.Clone();
                pending.RemoveAll(task => task.IsCompletedSuccessfully);
                if (pending.Count >= 16) throw new IOException("Too many pending TUI requests.");
                if (request.ValueKind != JsonValueKind.Object) continue;
                // Start admissions on the reader so pipelined inputs enter the FIFO in wire order.
                var admission = request.TryGetProperty("operation", out var operation)
                    && operation.ValueKind == JsonValueKind.String && operation.GetString() is "send" or "resize";
                pending.Add(admission ? Reply(request, handle) : Task.Run(() => Reply(request, handle)));
            }
        }
        catch (Exception error) when (error is IOException or OperationCanceledException or ObjectDisposedException or JsonException) { }
        finally
        {
            var replies = Task.WhenAll(pending);
            if (stopReading.IsCancellationRequested)
            {
                // The final manual step can close the session while its reply is still being written.
                try { await replies.WaitAsync(TimeSpan.FromSeconds(2)); }
                catch (TimeoutException) { }
            }
            lifetime.Cancel();
            await replies;
        }
    }

    private async Task Reply(JsonElement request, Func<JsonElement, Task<object>> handle)
    {
        if (!request.TryGetProperty("id", out var value) || !value.TryGetInt64(out var id) || id <= 0) return;
        TuiControlReply? receipt = null;
        try
        {
            var result = await handle(request).WaitAsync(lifetime.Token);
            receipt = result as TuiControlReply;
            await Send(new { kind = "reply", id, result });
        }
        catch (Exception error) when (error is IOException or OperationCanceledException or ObjectDisposedException)
        { lifetime.Cancel(); }
        catch (Exception error)
        {
            try { await Send(new { kind = "reply", id, result = new { status = "error", message = error.Message } }); }
            catch (Exception transport) when (transport is IOException or OperationCanceledException or ObjectDisposedException) { lifetime.Cancel(); }
        }
        finally { receipt?.Acknowledgement?.TrySetResult(); }
    }

    private async Task Send(object value)
    {
        var payload = JsonSerializer.SerializeToUtf8Bytes(value, Json);
        if (payload.Length > 16_777_216) throw new IOException("TUI frame or reply exceeds 16 MiB.");
        var header = new byte[4];
        BinaryPrimitives.WriteInt32BigEndian(header, payload.Length);
        using var deadline = CancellationTokenSource.CreateLinkedTokenSource(lifetime.Token);
        deadline.CancelAfter(TimeSpan.FromSeconds(2));
        await writer.WaitAsync(deadline.Token);
        try
        {
            await pipe.WriteAsync(header, deadline.Token);
            await pipe.WriteAsync(payload, deadline.Token);
            await pipe.FlushAsync(deadline.Token);
        }
        finally { writer.Release(); }
    }

    public void Dispose()
    {
        if (Interlocked.Exchange(ref disposed, 1) != 0) return;
        stopReading.Cancel();
        try { requests.GetAwaiter().GetResult(); }
        finally
        {
            pipe.Dispose();
            writer.Dispose();
            stopReading.Dispose();
            lifetime.Dispose();
        }
    }
}

public sealed record TuiControlReply(string Status, long InputId = 0, long FrameId = 0, string Stage = "", string Message = "",
    [property: JsonIgnore] TaskCompletionSource? Acknowledgement = null);
