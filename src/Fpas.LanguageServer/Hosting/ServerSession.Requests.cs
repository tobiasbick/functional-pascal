using Fpas.LanguageServer.Protocol;
using Fpas.LanguageServer.Transport;
using Fpas.Workspaces.Live;
using Fpas.Workspaces.Loading;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using StreamJsonRpc;
using System.Text;

namespace Fpas.LanguageServer.Hosting;

internal sealed partial class ServerSession
{
    private static readonly string[] CoreMethods = ["textDocument/definition", "textDocument/references", "workspace/symbol", "textDocument/prepareRename", "textDocument/rename", "textDocument/formatting"];
    internal static string[] FeatureMethods => [.. CoreMethods, .. SyntaxMethods, .. EditorMethods];
    private readonly AsyncLocal<RequestStamp?> request = new();
    private readonly HashSet<Task> requests = [];
    private bool renameSupported;
    private bool prepareSupported;

    internal RequestStamp CaptureRequest() { lock (gate) return new(revision, workspace!.Revision); }
    internal bool Current(RequestStamp stamp) { lock (gate) return state == SessionState.Running && revision == stamp.Session && workspace!.Revision == stamp.Workspace; }
    internal void Publish(RequestStamp? stamp, Action<bool> write) { lock (gate) write(stamp is null || Current(stamp)); }
    internal IDisposable EnterRequest(RequestStamp? stamp)
    { var previous = request.Value; request.Value = stamp; return new RequestScope(() => request.Value = previous); }
    private sealed class RequestScope(Action restore) : IDisposable { public void Dispose() => restore(); }

    internal void RegisterFeatures(JsonRpc connection)
    {
        foreach (var method in FeatureMethods)
        {
            Func<JObject, CancellationToken, Task<JToken?>> action = (parameters, token) => StartRequest(method, parameters, token);
            connection.AddLocalRpcMethod(action.Method, action.Target, new JsonRpcMethodAttribute(method) { UseSingleObjectParameterDeserialization = true });
        }
    }

    private Task<JToken?> StartRequest(string method, JObject parameters, CancellationToken token)
    {
        var stamp = request.Value ?? throw Invalid("Request admission is missing.", -32603);
        lock (gate)
        {
            CheckRequest(stamp);
            var work = Task.Run(() => ProcessRequest(method, parameters, stamp, token)); requests.Add(work);
            _ = work.ContinueWith(completed => { lock (gate) requests.Remove(completed); _ = completed.Exception; },
                CancellationToken.None, TaskContinuationOptions.ExecuteSynchronously, TaskScheduler.Default);
            return work;
        }
    }

    private async Task<JToken?> ProcessRequest(string method, JObject parameters, RequestStamp stamp, CancellationToken token)
    {
        using var deadline = CancellationTokenSource.CreateLinkedTokenSource(token, lifetime.Token); deadline.CancelAfter(TimeSpan.FromSeconds(30));
        try
        {
            deadline.Token.ThrowIfCancellationRequested();
            if (parameters is null) throw Invalid("Supply a parameter object.");
            object? result;
            if (method == "textDocument/formatting") result = Format(parameters, stamp, deadline.Token);
            else if (SyntaxMethods.Contains(method, StringComparer.Ordinal)) result = SyntaxFeature(method, parameters, stamp, deadline.Token);
            else
            {
                var core = CoreMethods.Contains(method, StringComparer.Ordinal) ? ReadFeature(method, parameters) : null;
                using (gate.EnterScope()) { CheckRequest(stamp); if (desynchronized.Count != 0) throw Invalid("Resynchronize open documents before semantic requests.", -32803); }
                var snapshot = await AnalyzeForRequest(stamp, deadline.Token);
                CheckRequest(stamp);
                result = core is not null ? Feature(method, core, snapshot, deadline.Token) :
                    EditorFeature(method, parameters, snapshot, deadline.Token);
                await workspace!.VerifyAsync(snapshot, deadline.Token);
            }
            var json = result is null ? null : ProtocolJson.Token(result, omitNulls: !CoreMethods.Contains(method, StringComparer.Ordinal));
            if (json is not null && Encoding.UTF8.GetByteCount(json.ToString(Newtonsoft.Json.Formatting.None)) > BoundedFormatter.OutputLimit - 1024)
                throw Invalid("Result exceeds 4 MiB. Narrow the request or reduce the document.", -32803);
            deadline.Token.ThrowIfCancellationRequested(); CheckRequest(stamp);
            return json;
        }
        catch (OperationCanceledException)
        {
            using (gate.EnterScope()) if (Current(stamp) && !lifetime.IsCancellationRequested) changes.Writer.TryWrite(0);
            if (token.IsCancellationRequested || lifetime.IsCancellationRequested) throw Invalid("Request cancelled.", -32800);
            if (!Current(stamp)) throw Invalid("Document or workspace content changed during the request.", -32801);
            throw Invalid("Request exceeded its 30-second work deadline.", -32803);
        }
        catch (ObjectDisposedException) when (lifetime.IsCancellationRequested) { throw Invalid("Request cancelled.", -32800); }
        catch (ContextFailure failure)
        {
            if (failure.Code is "content_modified" or "inputs_changed")
            {
                using (gate.EnterScope()) if (state == SessionState.Running && revision == stamp.Session) { workspace!.Invalidate(); Changed(); }
                throw Invalid(failure.Message, -32801);
            }
            throw Invalid(failure.Code + ": " + failure.Message, failure.Code is "invalid_argument" ? -32602 : -32803);
        }
        catch (Exception failure) when (failure is ArgumentException or JsonException or FormatException or OverflowException)
        { throw Invalid(failure.Message); }
        catch (Exception failure) when (failure is IOException or UnauthorizedAccessException)
        { throw Invalid("Cannot read workspace inputs: " + failure.Message, -32803); }
    }

    private void CheckRequest(RequestStamp stamp)
    { if (!Current(stamp)) throw Invalid("Document or workspace content changed during the request.", -32801); }

    private async Task<LiveSnapshot> AnalyzeForRequest(RequestStamp stamp, CancellationToken token)
    {
        for (var attempt = 0; ; attempt++)
        {
            try { return await workspace!.AnalyzeAsync(token); }
            catch (OperationCanceledException) when (!token.IsCancellationRequested && Current(stamp))
            {
                // Another request may have started and then cancelled the shared analysis.
                if (attempt == 2) throw Invalid("A concurrent request interrupted workspace analysis. Retry the request.", -32801);
            }
        }
    }
}
