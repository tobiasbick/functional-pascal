using Fpas.LanguageServer.Transport;
using Fpas.LanguageServer.Protocol;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using StreamJsonRpc;
using System.Text;
using System.Threading.Channels;

namespace Fpas.LanguageServer.Hosting;

internal sealed class NotificationQueue : IAsyncDisposable
{
    private sealed record Pending(string Method, JObject Parameters, Func<bool> Current, int Bytes);
    private readonly Channel<Pending> queue = Channel.CreateBounded<Pending>(64);
    private readonly CancellationTokenSource stop;
    private readonly Task writing;
    private readonly Action fail;
    private long bytes;

    internal NotificationQueue(JsonRpc rpc, Action fail, CancellationToken token)
    { this.fail = fail; stop = CancellationTokenSource.CreateLinkedTokenSource(token); writing = Task.Run(() => Write(rpc)); }

    internal void Enqueue(string method, object parameters, Func<bool> current)
    {
        if (stop.IsCancellationRequested) return;
        var json = ProtocolJson.Object(parameters);
        var size = Encoding.UTF8.GetByteCount(json.ToString(Formatting.None));
        if (size > BoundedFormatter.OutputLimit - 1024 || Interlocked.Add(ref bytes, size) > BoundedFormatter.InputLimit)
        { fail(); return; }
        if (!queue.Writer.TryWrite(new(method, json, current, size))) { Interlocked.Add(ref bytes, -size); fail(); }
    }

    private async Task Write(JsonRpc rpc)
    {
        try
        {
            await foreach (var item in queue.Reader.ReadAllAsync(stop.Token))
            {
                try
                {
                    if (item.Current()) await rpc.NotifyWithParameterObjectAsync(item.Method, item.Parameters).WaitAsync(TimeSpan.FromSeconds(5), stop.Token);
                }
                finally { Interlocked.Add(ref bytes, -item.Bytes); }
            }
        }
        catch (Exception error) when (error is OperationCanceledException or IOException or InvalidDataException or JsonException or TimeoutException or ObjectDisposedException or ConnectionLostException)
        { if (!stop.IsCancellationRequested) fail(); }
    }

    public async ValueTask DisposeAsync()
    { stop.Cancel(); queue.Writer.TryComplete(); await writing; while (queue.Reader.TryRead(out _)) { } stop.Dispose(); }
}
