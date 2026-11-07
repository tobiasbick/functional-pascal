using Fpas.LanguageServer.Protocol;
using Fpas.Workspaces.Live;
using Fpas.Workspaces.Loading;
using OmniSharp.Extensions.LanguageServer.Protocol.Models;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using StreamJsonRpc.Protocol;

namespace Fpas.LanguageServer.Hosting;

internal sealed partial class ServerSession
{
    // Notifications mutate documents on the reader before the next frame is read.
    // Their payloads never accumulate behind asynchronous compilation work.
    internal bool ReceiveNotification(JsonRpcRequest request)
    {
        if (request.Method == "$/cancelRequest") return false;
        if (request.Method == "exit") { RequestExit(); return true; }
        lock (gate)
        {
            if (request.Method == "initialized" && state == SessionState.Initialized)
            {
                state = SessionState.Running; Changed();
                if (watchRegistration) registration = RegisterWatchers();
                return true;
            }
            if (state != SessionState.Running) return true;
            string? path = null;
            try
            {
                var parameters = request.Arguments is null ? new JObject() : JObject.FromObject(request.Arguments, Serializer());
                if (request.Method == "workspace/didChangeWatchedFiles")
                {
                    if (parameters["changes"] is not JArray files || files.Count > 1000) throw Invalid("Supply at most 1,000 watched file changes.");
                    foreach (var file in files)
                    {
                        _ = FileUris.Path(new Uri(RequiredString(file, "uri"), UriKind.RelativeOrAbsolute));
                        var type = RequiredInteger(file, "type"); if (type is < 1 or > 3) throw Invalid("Invalid watched-file change type.");
                    }
                    workspace!.Invalidate(); Changed(); return true;
                }
                if (request.Method is not ("textDocument/didOpen" or "textDocument/didChange" or "textDocument/didSave" or "textDocument/didClose")) return true;
                if (parameters["textDocument"] is not JObject document) throw Invalid("Supply textDocument.");
                path = FileUris.Path(new Uri(RequiredString(document, "uri"), UriKind.RelativeOrAbsolute));
                switch (request.Method)
                {
                    case "textDocument/didOpen":
                        if (RequiredString(document, "languageId") != "fpas") throw Invalid("The document languageId must be fpas.");
                        workspace!.Open(path, RequiredString(document, "text"), RequiredInteger(document, "version"));
                        desynchronized.Remove(path); break;
                    case "textDocument/didChange":
                        if (parameters["contentChanges"] is not JArray edits || edits.Count is 0 or > 1000) throw Invalid("Supply 1..1,000 contentChanges.");
                        var changes = edits.Select(edit => new TextChange(RequiredString(edit, "text"), ChangeRange(edit["range"]))).ToArray();
                        if (desynchronized.Contains(path) && changes.Any(change => change.Range is not null)) throw Invalid("Send a full-text change or close and reopen this document to resynchronize.");
                        workspace!.Change(path, RequiredInteger(document, "version"), changes); desynchronized.Remove(path); break;
                    case "textDocument/didSave": workspace!.Save(path); break;
                    case "textDocument/didClose":
                        workspace!.Close(path); desynchronized.Remove(path); Changed();
                        // The FIFO clear must survive unrelated edits and a quick reopen.
                        outgoing.Enqueue("textDocument/publishDiagnostics", new PublishDiagnosticsParams { Uri = FileUris.Uri(path), Diagnostics = [] }, () => State == SessionState.Running);
                        return true;
                }
                Changed(); return true;
            }
            catch (Exception error) when (error is JsonException or ArgumentException or OverflowException or InvalidOperationException or ContextFailure or StreamJsonRpc.LocalRpcException)
            {
                if (path is not null && workspace!.OpenDocuments.Documents.Any(document => PathComparer.Equals(document.Path, path)))
                {
                    desynchronized.Add(path); workspace.Invalidate(); Changed(); ClearDiagnostics(revision);
                }
                Log("Invalid " + request.Method + ": " + error.Message); return true;
            }
        }
    }

    private bool IsCurrent(long selected) { lock (gate) return state == SessionState.Running && revision == selected; }
    private static string RequiredString(JToken token, string name) => token[name]?.Type == JTokenType.String ? token.Value<string>(name)! : throw Invalid("Supply string " + name + ".");
    private static int RequiredInteger(JToken token, string name) => token[name]?.Type == JTokenType.Integer && token[name]!.Value<long>() is >= int.MinValue and <= int.MaxValue
        ? token[name]!.Value<int>() : throw Invalid("Supply a 32-bit integer " + name + ".");
    private static TextRange? ChangeRange(JToken? token)
    {
        if (token is null || token.Type == JTokenType.Null) return null;
        TextPosition Position(JToken? value)
        {
            if (value is not JObject) throw Invalid("Supply a range position.");
            var line = RequiredInteger(value, "line"); var character = RequiredInteger(value, "character");
            if (line < 0 || character < 0) throw Invalid("Range positions must be nonnegative.");
            return new(checked(line + 1), checked(character + 1));
        }
        return new(Position(token["start"]), Position(token["end"]));
    }
    private async Task RegisterWatchers()
    {
        try
        {
            using var deadline = CancellationTokenSource.CreateLinkedTokenSource(lifetime.Token); deadline.CancelAfter(TimeSpan.FromSeconds(30));
            await rpc.InvokeWithParameterObjectAsync<object?>("client/registerCapability", new
            {
                registrations = new[] { new { id = "fpas-inputs", method = "workspace/didChangeWatchedFiles", registerOptions = new
                { watchers = new[] { new { globPattern = "**/*.{fpas,fpasproj,slnx,cs,csproj,fs,fsi,fsproj,props,targets,json,config,Config,dll,pdb,xml}", kind = 7 } } } } }
            }, deadline.Token);
        }
        catch (Exception error) when (error is StreamJsonRpc.RemoteRpcException or OperationCanceledException or StreamJsonRpc.ConnectionLostException)
        { if (!lifetime.IsCancellationRequested) Log("Input watching could not be registered: " + error.Message); }
    }
}
