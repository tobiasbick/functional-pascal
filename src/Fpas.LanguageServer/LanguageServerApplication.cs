using Fpas.LanguageServer.Hosting;
using Fpas.LanguageServer.Transport;
using StreamJsonRpc;

namespace Fpas.LanguageServer;

/// <summary>Hosts the FPAS language server over a JSON-RPC stream pair.</summary>
public static class LanguageServerApplication
{
    /// <summary>Serve LSP requests until the client exits, the connection closes or the caller cancels.</summary>
    /// <param name="input">Client-to-server LSP byte stream.</param>
    /// <param name="output">Server-to-client LSP byte stream.</param>
    /// <param name="error">Diagnostic log for transport and request failures.</param>
    /// <param name="token">Cancellation that ends the session with exit code 130.</param>
    /// <returns>0 after an orderly shutdown and exit or a closed connection, 1 after a failure or an exit without shutdown, or 130 when cancelled.</returns>
    public static async Task<int> RunAsync(Stream input, Stream output, TextWriter error, CancellationToken token = default)
    {
        await using var session = new ServerSession(error);
        using var handler = new LspMessageHandler(new(input), output, new(), session);
        using var rpc = new ServerRpc(handler, session, error) { CancelLocallyInvokedMethodsWhenConnectionIsClosed = true };
        session.Attach(rpc);
        var initialize = new Func<Newtonsoft.Json.Linq.JObject, CancellationToken, Newtonsoft.Json.Linq.JObject>(session.Initialize);
        rpc.AddLocalRpcMethod(initialize.Method, session, new JsonRpcMethodAttribute("initialize") { UseSingleObjectParameterDeserialization = true });
        rpc.AddLocalRpcMethod("shutdown", new Func<Task<object?>>(session.Shutdown));
        session.RegisterFeatures(rpc);
        try
        {
            rpc.StartListening();
            var cancelled = Task.Delay(Timeout.Infinite, token);
            var completed = await Task.WhenAny(rpc.Completion, session.Exit, cancelled);
            if (completed == cancelled) return 130;
            if (completed == session.Exit) return await session.Exit;
            await rpc.Completion; return 0;
        }
        catch (Exception failure) when (failure is IOException or InvalidDataException or Newtonsoft.Json.JsonException or InvalidOperationException or ArgumentException or StreamJsonRpc.ConnectionLostException or BadRpcHeaderException)
        { await error.WriteLineAsync("LSP connection failed: " + failure.Message); return 1; }
    }
}
