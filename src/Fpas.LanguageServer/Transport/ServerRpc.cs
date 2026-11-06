using StreamJsonRpc;
using StreamJsonRpc.Protocol;
using StreamJsonRpc.Reflection;
using Fpas.LanguageServer.Hosting;

namespace Fpas.LanguageServer.Transport;

internal sealed class ServerRpc(LspMessageHandler handler, ServerSession session, TextWriter error) : JsonRpc(handler)
{
    protected override async ValueTask<JsonRpcMessage> DispatchRequestAsync(JsonRpcRequest request, TargetMethod targetMethod, CancellationToken cancellationToken)
    {
        using var context = session.EnterRequest(handler.Stamp(request.RequestId));
        return await base.DispatchRequestAsync(request, targetMethod, cancellationToken);
    }
    protected override JsonRpcError.ErrorDetail CreateErrorDetails(JsonRpcRequest request, Exception exception)
    {
        if (exception is not (LocalRpcException or OperationCanceledException)) error.WriteLine("LSP request failed: " + exception.Message);
        return new()
        {
            Code = (JsonRpcErrorCode)(exception is LocalRpcException local ? local.ErrorCode : exception is OperationCanceledException ? -32800 : -32603),
            Message = exception is LocalRpcException or OperationCanceledException ? exception.Message : "The language server could not complete the request."
        };
    }
}
