using Fpas.LanguageServer.Protocol;
using Fpas.Workspaces.Live;
using OmniSharp.Extensions.LanguageServer.Protocol.Models;
using OmniSharp.Extensions.LanguageServer.Protocol.Server.Capabilities;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using StreamJsonRpc;
using System.Threading.Channels;

namespace Fpas.LanguageServer.Hosting;

internal enum SessionState { Created, Initialized, Running, ShuttingDown, Exited }

internal sealed partial class ServerSession : IAsyncDisposable
{
    private readonly Lock gate = new();
    private readonly CancellationTokenSource lifetime = new();
    private readonly Channel<byte> changes = Channel.CreateBounded<byte>(new BoundedChannelOptions(1) { FullMode = BoundedChannelFullMode.DropWrite });
    private readonly HashSet<string> desynchronized = new(PathComparer);
    private readonly TaskCompletionSource<int> exit = new(TaskCreationOptions.RunContinuationsAsynchronously);
    private readonly Task diagnostics;
    private readonly TextWriter errors;
    private JsonRpc rpc = null!;
    private NotificationQueue outgoing = null!;
    private LiveWorkspace? workspace;
    private SessionState state;
    private long revision;
    private bool watchRegistration;
    private Task? registration;
    private static StringComparer PathComparer => OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
    internal Task<int> Exit => exit.Task;
    internal SessionState State { get { lock (gate) return state; } }

    internal ServerSession(TextWriter errors) { this.errors = errors; diagnostics = Task.Run(DiagnosticsLoop); }
    internal void Attach(JsonRpc connection)
    { rpc = connection; outgoing = new(connection, () => Fail(), lifetime.Token); }

    [JsonRpcMethod("initialize", UseSingleObjectParameterDeserialization = true)]
    public JObject Initialize(JObject parameters, CancellationToken cancellationToken)
    {
        cancellationToken.ThrowIfCancellationRequested();
        lock (gate)
        {
            if (state != SessionState.Created) throw Invalid("The server is already initialized.", -32600);
            if (parameters is null || parameters["capabilities"] is not JObject || parameters["initializationOptions"] is not JObject options || options["context"]?.Type != JTokenType.String)
                throw Invalid("Supply capabilities and initializationOptions.context.");
            var capabilities = (JObject)parameters["capabilities"]!;
            static JObject? Section(JObject parent, string name) => parent[name] is null ? null : parent[name] as JObject ?? throw Invalid("Supply object " + name + ".");
            var general = Section(capabilities, "general");
            var watching = Section(capabilities, "workspace") is { } clientWorkspace ? Section(clientWorkspace, "didChangeWatchedFiles") : null;
            var edits = Section(capabilities, "workspace") is { } workspaceOptions ? Section(workspaceOptions, "workspaceEdit") : null;
            if (edits?["documentChanges"] is { Type: not JTokenType.Boolean }) throw Invalid("documentChanges must be a boolean.");
            var rename = Section(capabilities, "textDocument") is { } textOptions ? Section(textOptions, "rename") : null;
            if (rename?["prepareSupport"] is { Type: not JTokenType.Boolean }) throw Invalid("prepareSupport must be a boolean.");
            renameSupported = edits?.Value<bool>("documentChanges") == true; prepareSupported = rename?.Value<bool>("prepareSupport") == true;
            var symbolSupport = Section(capabilities, "workspace") is { } workspaceSymbols ? Section(workspaceSymbols, "symbol") : null;
            var kinds = symbolSupport is null ? null : Section(symbolSupport, "symbolKind")?["valueSet"];
            symbolKinds = Enumerable.Range(1, 18).ToHashSet();
            if (kinds is not null)
            {
                if (kinds is not JArray values || values.Count == 0 || values.Any(value => value.Type != JTokenType.Integer ||
                    !long.TryParse(value.ToString(), System.Globalization.NumberStyles.None, System.Globalization.CultureInfo.InvariantCulture, out var kind) || kind is < 1 or > 26))
                    throw Invalid("Supply supported symbolKind values from 1..26.");
                symbolKinds = values.Select(value => value.Value<int>()).ToHashSet();
            }
            if (watching?["dynamicRegistration"] is { Type: not JTokenType.Boolean }) throw Invalid("dynamicRegistration must be a boolean.");
            var encodings = general?["positionEncodings"];
            if (encodings is not null && (encodings is not JArray supported || !supported.Any(value => value.Type == JTokenType.String && value.Value<string>() == "utf-16")))
                throw Invalid("This server requires UTF-16 position encoding.");
            var context = FileUris.Context(options.Value<string>("context")!);
            if (!File.Exists(context) || System.IO.Path.GetExtension(context).ToLowerInvariant() is not (".fpasproj" or ".slnx"))
                throw Invalid("Select an existing .fpasproj or .slnx context.");
            if (options["configuration"] is { } configuration && (configuration.Type != JTokenType.String || string.IsNullOrWhiteSpace(configuration.Value<string>())))
                throw Invalid("configuration must be a nonempty string.");
            if (options["correctionCommand"] is { Type: not JTokenType.Boolean })
                throw Invalid("correctionCommand must be a boolean.");
            correctionCommand = options.Value<bool>("correctionCommand");
            workspace = new(new(context, options.Value<string>("configuration") ?? "Debug"));
            watchRegistration = watching?.Value<bool>("dynamicRegistration") == true;
            state = SessionState.Initialized;
            var response = ProtocolJson.Object(new InitializeResult { Capabilities = new ServerCapabilities { PositionEncoding = PositionEncodingKind.UTF16,
                DefinitionProvider = true, ReferencesProvider = true, WorkspaceSymbolProvider = true, DocumentFormattingProvider = true,
                RenameProvider = renameSupported ? new BooleanOr<RenameRegistrationOptions.StaticOptions>(new RenameRegistrationOptions.StaticOptions { PrepareProvider = prepareSupported }) : null,
                TextDocumentSync = new TextDocumentSyncOptions
                { OpenClose = true, Change = TextDocumentSyncKind.Incremental, Save = new SaveOptions { IncludeText = false } } } });
            EditorCapabilities((JObject)response["capabilities"]!, capabilities["textDocument"] as JObject);
            return response;
        }
    }

    [JsonRpcMethod("shutdown")]
    public async Task<object?> Shutdown()
    {
        LiveWorkspace? selected;
        using (gate.EnterScope())
        {
            if (state is not (SessionState.Initialized or SessionState.Running)) throw Invalid("The server is not accepting shutdown.", state == SessionState.Created ? -32002 : -32600);
            state = SessionState.ShuttingDown; revision++; selected = workspace; lifetime.Cancel(); changes.Writer.TryComplete();
        }
        if (selected is not null) await selected.DisposeAsync();
        await diagnostics;
        Task[] pending; using (gate.EnterScope()) pending = requests.ToArray();
        try { await Task.WhenAll(pending); } catch (Exception) { /* Request failures are returned by JSON-RPC. */ }
        return null;
    }

    internal void RequestExit()
    {
        lock (gate)
        {
            var code = state == SessionState.ShuttingDown ? 0 : 1;
            state = SessionState.Exited; revision++; lifetime.Cancel(); changes.Writer.TryComplete(); exit.TrySetResult(code);
        }
    }
    internal void Fail() { lock (gate) { state = SessionState.Exited; revision++; exit.TrySetResult(1); lifetime.Cancel(); } }
    private void Changed()
    {
        revision++;
        refactorings.Clear();
        changes.Writer.TryWrite(0);
        if (!viewsPublished) return;
        viewsPublished = false;
        outgoing.Enqueue("fpas/viewsInvalidated", new { revision }, () => State == SessionState.Running);
    }
    private static LocalRpcException Invalid(string message, int code = -32602) => new(message) { ErrorCode = code };
    private static JsonSerializer Serializer() => JsonSerializer.Create(new JsonSerializerSettings
        { MaxDepth = 64, TypeNameHandling = TypeNameHandling.None, DateParseHandling = DateParseHandling.None, NullValueHandling = NullValueHandling.Ignore });
    private void Log(string message)
    {
        if (message.Length > 8192) message = message[..8192];
        outgoing.Enqueue("window/logMessage", new LogMessageParams { Type = MessageType.Error, Message = message }, () => State is SessionState.Initialized or SessionState.Running);
    }

    public async ValueTask DisposeAsync()
    {
        using (gate.EnterScope()) { state = SessionState.Exited; revision++; lifetime.Cancel(); changes.Writer.TryComplete(); }
        if (workspace is not null) await workspace.DisposeAsync();
        await diagnostics;
        Task[] pending; using (gate.EnterScope()) pending = requests.ToArray();
        try { await Task.WhenAll(pending); } catch (Exception) { /* Request failures are returned by JSON-RPC. */ }
        if (registration is not null) try { await registration; } catch (OperationCanceledException) { }
        if (outgoing is not null) await outgoing.DisposeAsync();
        lifetime.Dispose();
    }
}
