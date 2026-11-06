using System.Globalization;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Text.Json.Serialization;
using System.Threading.Channels;

namespace Fpas.Debugging.Dap;

internal sealed class DapTransport(Stream input, Stream output, CancellationToken token)
{
    internal const int MaximumBytes = 1_048_576;
    private readonly object gate = new();
    private readonly Channel<byte[]> queue = Channel.CreateBounded<byte[]>(128);
    private int sequence, queued, queuedMessages;
    internal Task WriteTask { get; private set; } = Task.CompletedTask;
    internal void Start() => WriteTask = Write();

    internal async Task<JsonObject?> Read()
    {
        var header = new List<byte>(); var single = new byte[1];
        while (true)
        {
            var read = await input.ReadAsync(single, token);
            if (read == 0) return header.Count == 0 ? null : throw new IOException("Incomplete DAP header.");
            if (single[0] > 127) throw new IOException("DAP headers must be ASCII.");
            header.Add(single[0]);
            if (header.Count > 8192) throw new IOException("DAP headers exceed 8 KiB.");
            if (header.Count >= 4 && header[^4] == 13 && header[^3] == 10 && header[^2] == 13 && header[^1] == 10) break;
        }
        int? length = null;
        foreach (var line in Encoding.ASCII.GetString(header.ToArray()).Split("\r\n", StringSplitOptions.RemoveEmptyEntries))
        {
            var colon = line.IndexOf(':');
            if (colon <= 0) throw new IOException("Malformed DAP header.");
            if (!line[..colon].Equals("Content-Length", StringComparison.OrdinalIgnoreCase)) continue;
            if (length is not null || !int.TryParse(line[(colon + 1)..].Trim(), NumberStyles.None, CultureInfo.InvariantCulture, out var parsed) || parsed is < 1 or > MaximumBytes)
                throw new IOException("DAP requires one Content-Length in 1..1048576.");
            length = parsed;
        }
        if (length is null) throw new IOException("DAP Content-Length is missing.");
        var body = new byte[length.Value]; await input.ReadExactlyAsync(body, token);
        _ = new UTF8Encoding(false, true).GetCharCount(body);
        return JsonNode.Parse(body, documentOptions: new() { MaxDepth = 64 }) as JsonObject ?? throw new IOException("A DAP message must be an object.");
    }

    internal bool Send(JsonObject message, bool lossy = false)
    {
        lock (gate)
        {
            message["seq"] = checked(++sequence);
            var data = JsonSerializer.SerializeToUtf8Bytes(message, DapJson.Default.JsonObject);
            var cost = data.Length + Encoding.ASCII.GetByteCount($"Content-Length: {data.Length}\r\n\r\n");
            if (data.Length > MaximumBytes || queued + cost > MaximumBytes || lossy && (queuedMessages >= 120 || queued + cost > MaximumBytes - 16384) || !queue.Writer.TryWrite(data))
            {
                sequence--;
                if (lossy) return false;
                throw new IOException("The bounded DAP event/response queue is full.");
            }
            queued += cost; queuedMessages++; return true;
        }
    }

    internal void Complete() => queue.Writer.TryComplete();

    private async Task Write()
    {
        await foreach (var body in queue.Reader.ReadAllAsync())
        {
            using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(5));
            await output.WriteAsync(Encoding.ASCII.GetBytes($"Content-Length: {body.Length}\r\n\r\n"), deadline.Token);
            await output.WriteAsync(body, deadline.Token); await output.FlushAsync(deadline.Token);
            lock (gate) { queued -= body.Length + Encoding.ASCII.GetByteCount($"Content-Length: {body.Length}\r\n\r\n"); queuedMessages--; }
        }
    }
}

[JsonSerializable(typeof(JsonObject))]
internal sealed partial class DapJson : JsonSerializerContext;
