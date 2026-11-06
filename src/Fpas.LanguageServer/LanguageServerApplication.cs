using Fpas.LanguageServer.Hosting;
using Fpas.LanguageServer.Transport;
using StreamJsonRpc;

namespace Fpas.LanguageServer;

public static class LanguageServerApplication
{
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
        finally { rpc.Dispose(); }
    }
}
