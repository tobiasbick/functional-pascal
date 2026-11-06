using System.Buffers.Binary;
using System.Text.Json;

namespace Fpas.Debugging.Hosting;

internal sealed class WorkerChannel(Stream stream) : IAsyncDisposable
{
    private readonly SemaphoreSlim writer = new(1);
    internal const int MaximumMessageBytes = 1_048_576;

    internal async Task Send(WorkerMessage message, CancellationToken token)
    {
        var content = JsonSerializer.SerializeToUtf8Bytes(message, WorkerJson.Default.WorkerMessage);
        if (content.Length > MaximumMessageBytes) throw new IOException("The debugger worker message exceeds 1 MiB.");
        var header = new byte[4]; BinaryPrimitives.WriteInt32BigEndian(header, content.Length);
        await writer.WaitAsync(token);
        try
        {
            await stream.WriteAsync(header, token);
            await stream.WriteAsync(content, token);
            await stream.FlushAsync(token);
        }
        finally { writer.Release(); }
    }

    internal async Task<WorkerMessage?> Read(CancellationToken token)
    {
        var header = new byte[4];
        var first = await stream.ReadAsync(header.AsMemory(0, 1), token);
        if (first == 0) return null;
        await stream.ReadExactlyAsync(header.AsMemory(1), token);
        var length = BinaryPrimitives.ReadInt32BigEndian(header);
        if (length is <= 0 or > MaximumMessageBytes) throw new IOException("Invalid debugger worker message length.");
        var content = new byte[length];
        await stream.ReadExactlyAsync(content, token);
        return JsonSerializer.Deserialize(content, WorkerJson.Default.WorkerMessage) ?? throw new IOException("Invalid debugger worker message.");
    }

    public async ValueTask DisposeAsync()
    {
        await stream.DisposeAsync();
        writer.Dispose();
    }
}
