using Fpas.LanguageServer.Hosting;
using StreamJsonRpc;
using StreamJsonRpc.Protocol;
using StreamJsonRpc.Reflection;

namespace Fpas.LanguageServer.Transport;

internal sealed class LspMessageHandler : HeaderDelimitedMessageHandler
{
    private readonly ServerSession session;
    private readonly object gate = new();
    private readonly Dictionary<RequestId, RequestStamp?> pending = [];
    internal RequestStamp? Stamp(RequestId id) { lock (gate) return pending.GetValueOrDefault(id); }

    internal LspMessageHandler(QuotaReadStream input, Stream output, BoundedFormatter formatter, ServerSession session) : base(output, input, formatter)
    { this.session = session; }

    protected override async ValueTask<JsonRpcMessage?> ReadCoreAsync(CancellationToken cancellationToken)
    {
        while (true)
        {
            var message = await base.ReadCoreAsync(cancellationToken);
            if (message is not JsonRpcRequest request) return message;
            if (request.IsNotification)
            {
                if (!session.ReceiveNotification(request)) return message;
                ((IJsonRpcMessageBufferManager)this).DeserializationComplete(message); continue;
            }
            int? error = null; string reason = "Invalid request.";
            var feature = ServerSession.FeatureMethods.Contains(request.Method, StringComparer.Ordinal);
            if (feature && session.State != SessionState.Running)
            { error = session.State is SessionState.Created or SessionState.Initialized ? -32002 : -32600; reason = "Initialize the server before feature requests."; }
            else if (!feature && request.Method is not ("initialize" or "shutdown"))
            { error = session.State == SessionState.Created ? -32002 : session.State is SessionState.ShuttingDown or SessionState.Exited ? -32600 : -32601; reason = "This request is not available in the current server state."; }
            else if (request.Method == "shutdown" && session.State == SessionState.Created) { error = -32002; reason = "Initialize the server first."; }
            lock (gate)
            {
                if (pending.ContainsKey(request.RequestId)) throw new InvalidDataException("Duplicate outstanding LSP request ID.");
                if (error is null && pending.Count >= 64 && !(request.Method == "shutdown" && pending.Count == 64))
                { error = -32803; reason = "Too many outstanding requests."; }
                if (error is null) pending.Add(request.RequestId, feature ? session.CaptureRequest() : null);
            }
            if (error is null) return request;
            await WriteAsync(new JsonRpcError { RequestId = request.RequestId, Error = new() { Code = (JsonRpcErrorCode)error.Value, Message = reason } }, cancellationToken);
            ((IJsonRpcMessageBufferManager)this).DeserializationComplete(message);
        }
    }

    protected override void Write(JsonRpcMessage content, CancellationToken cancellationToken)
    {
        var stamp = content is JsonRpcResult result ? Stamp(result.RequestId) : null;
        session.Publish(stamp, current =>
        {
            var response = !current && content is JsonRpcResult stale ? new JsonRpcError { RequestId = stale.RequestId, Error = new()
                { Code = (JsonRpcErrorCode)(-32801), Message = "Document or workspace content changed before publication." } } : content;
            base.Write(response, cancellationToken);
        });
        if (content is IJsonRpcMessageWithId identified && content is not JsonRpcRequest) lock (gate) pending.Remove(identified.RequestId);
    }
}
