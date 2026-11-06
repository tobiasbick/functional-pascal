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
    internal static readonly string[] FeatureMethods = ["textDocument/definition", "textDocument/references", "workspace/symbol", "textDocument/prepareRename", "textDocument/rename", "textDocument/formatting"];
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
        Register("textDocument/definition", Definition); Register("textDocument/references", References); Register("workspace/symbol", Symbols);
        Register("textDocument/prepareRename", PrepareRename); Register("textDocument/rename", Rename); Register("textDocument/formatting", Formatting);
        void Register(string method, Func<JObject, CancellationToken, Task<JToken?>> action)
            => connection.AddLocalRpcMethod(action.Method, this, new JsonRpcMethodAttribute(method) { UseSingleObjectParameterDeserialization = true });
    }
    public Task<JToken?> Definition(JObject parameters, CancellationToken token) => StartRequest("textDocument/definition", parameters, token);
    public Task<JToken?> References(JObject parameters, CancellationToken token) => StartRequest("textDocument/references", parameters, token);
    public Task<JToken?> Symbols(JObject parameters, CancellationToken token) => StartRequest("workspace/symbol", parameters, token);
    public Task<JToken?> PrepareRename(JObject parameters, CancellationToken token) => StartRequest("textDocument/prepareRename", parameters, token);
    public Task<JToken?> Rename(JObject parameters, CancellationToken token) => StartRequest("textDocument/rename", parameters, token);
    public Task<JToken?> Formatting(JObject parameters, CancellationToken token) => StartRequest("textDocument/formatting", parameters, token);

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
            else
            {
                var input = ReadFeature(method, parameters);
                lock (gate) { CheckRequest(stamp); if (desynchronized.Count != 0) throw Invalid("Resynchronize open documents before semantic requests.", -32803); }
                var snapshot = await workspace!.AnalyzeAsync(deadline.Token);
                lock (gate) CheckRequest(stamp);
                result = Feature(method, input, snapshot, deadline.Token);
                await workspace.VerifyAsync(snapshot, deadline.Token);
            }
            var json = result is null ? null : ProtocolJson.Token(result);
            if (json is not null && Encoding.UTF8.GetByteCount(json.ToString(Newtonsoft.Json.Formatting.None)) > BoundedFormatter.OutputLimit - 1024)
                throw Invalid("Result exceeds 4 MiB. Narrow the request or reduce the document.", -32803);
            deadline.Token.ThrowIfCancellationRequested(); lock (gate) CheckRequest(stamp);
            return json;
        }
        catch (OperationCanceledException)
        {
            lock (gate) if (Current(stamp) && !lifetime.IsCancellationRequested) changes.Writer.TryWrite(0);
            if (token.IsCancellationRequested || lifetime.IsCancellationRequested) throw Invalid("Request cancelled.", -32800);
            if (!Current(stamp)) throw Invalid("Document or workspace content changed during the request.", -32801);
            throw Invalid("Request exceeded its 30-second work deadline.", -32803);
        }
        catch (ObjectDisposedException) when (lifetime.IsCancellationRequested) { throw Invalid("Request cancelled.", -32800); }
        catch (ContextFailure failure)
        {
            if (failure.Code is "content_modified" or "inputs_changed")
            {
                lock (gate) if (state == SessionState.Running && revision == stamp.Session) { workspace!.Invalidate(); Changed(); }
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
}
