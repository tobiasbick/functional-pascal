using Fpas.Debugging.Protocol;
using System.Text.Json;

namespace Fpas.Debugging.Sessions;

internal sealed partial class DebugSession : IAsyncDisposable
{
    private readonly string backendPath;
    private readonly object gate = new();
    private readonly SemaphoreSlim control = new(1);
    private readonly HashSet<int> frameHandles = [], variableHandles = [];
    private readonly List<BreakpointInfo> breakpoints = [];
    private TaskCompletionSource changed = Signal();
    private DapConnection? connection;
    private OutputBuffer output = new();
    private string sessionId = "", state = "exited";
    private long revision, stopSequence;
    private long? stopId;
    private int? threadId, exitCode;
    private string? reason;
    private SourcePosition? location;
    private FailureInfo? error;
    private ExceptionInfo? exceptionInfo;
    private Task cleanup = Task.CompletedTask;
    private bool stopping;

    internal DebugSession(string backendPath) => this.backendPath = backendPath;
    private static TaskCompletionSource Signal() => new(TaskCreationOptions.RunContinuationsAsynchronously);
    private void Pulse() { revision++; var previous = changed; changed = Signal(); previous.TrySetResult(); }
    private SessionSnapshot Snapshot() => new(sessionId, state, revision, stopId, reason, threadId, location, exitCode, error, breakpoints.ToArray(), exceptionInfo);
    private void Session(string id)
    {
        if (string.IsNullOrEmpty(id) || id != sessionId) throw new DebugFailure("invalid_session", "Use the sessionId returned by debug_launch in this server. Launch again if the server restarted.");
    }
    private void Stop(string id, long expected)
    {
        Session(id);
        if (state != "stopped" || expected != stopId) throw new DebugFailure("stale_reference", "Use the current stopped session/stopId; references expire on resume.");
    }
    private void Running()
    {
        state = "running"; stopId = null; reason = null; threadId = null; location = null; exceptionInfo = null;
        frameHandles.Clear(); variableHandles.Clear(); Pulse();
    }

    private void Event(JsonElement item)
    {
        var name = item.GetProperty("event").GetString();
        var body = item.TryGetProperty("body", out var eventBody) ? eventBody : JsonSerializer.SerializeToElement(new { });
        lock (gate)
        {
            if (stopping) return;
            switch (name)
            {
                case "continued": Running(); break;
                case "stopped":
                    state = "stopped"; stopId = ++stopSequence;
                    reason = Text(body, "reason"); threadId = Integer(body, "threadId"); location = Position(body);
                    exceptionInfo = null;
                    frameHandles.Clear(); variableHandles.Clear(); Pulse(); break;
                case "exited":
                    state = "exited"; exitCode = Integer(body, "exitCode"); stopId = null;
                    frameHandles.Clear(); variableHandles.Clear(); Pulse(); break;
                case "terminated":
                    if (state != "failed") { state = "exited"; stopId = null; Pulse(); }
                    BeginCleanup();
                    break;
                case "breakpoint":
                    if (body.TryGetProperty("breakpoint", out var point))
                    {
                        var value = Breakpoint(point);
                        var index = breakpoints.FindIndex(existing => existing.Id == value.Id);
                        if (Text(body, "reason") == "removed")
                        {
                            if (index >= 0) breakpoints.RemoveAt(index);
                        }
                        else if (index >= 0) breakpoints[index] = value with { Source = value.Source ?? breakpoints[index].Source };
                        else if (breakpoints.Count < 4096) breakpoints.Add(value);
                        else throw new InvalidDataException("Breakpoint limit exceeded.");
                        Pulse();
                    }
                    break;
                case "output":
                    var category = Text(body, "category");
                    if (category is "stdout" or "stderr") output.Add(category, Text(body, "output") ?? "");
                    break;
            }
        }
    }

    private void Failed(Exception exception, DapConnection? expected = null)
    {
        lock (gate)
        {
            if (stopping || state == "failed" || (expected is not null && !ReferenceEquals(expected, connection))) return;
            if (state == "exited") { BeginCleanup(); return; }
            state = "failed"; stopId = null; frameHandles.Clear(); variableHandles.Clear();
            error = new(exception is DebugFailure failure ? failure.Code : "backend_exited", "The debugger connection failed; the owned session is being stopped."); Pulse();
            BeginCleanup();
        }
    }

    private void BeginCleanup()
    {
        var current = connection;
        connection = null;
        if (current is not null) cleanup = Task.Run(async () => await Disconnect(current, CancellationToken.None));
    }

    private static string? Text(JsonElement value, string name) => value.TryGetProperty(name, out var item) && item.ValueKind == JsonValueKind.String ? item.GetString() : null;
    private static int? Integer(JsonElement value, string name) => value.TryGetProperty(name, out var item) && item.ValueKind == JsonValueKind.Number && item.TryGetInt32(out var number) ? number : null;
    private static SourcePosition? Position(JsonElement value) => value.TryGetProperty("source", out var source) && Text(source, "path") is { } path
        ? new(path, Integer(value, "line") ?? 0, Integer(value, "column") ?? 0) : null;
    private static BreakpointInfo Breakpoint(JsonElement point) => new(Integer(point, "id"), point.TryGetProperty("verified", out var verified) && verified.GetBoolean(),
        Integer(point, "line"), point.TryGetProperty("source", out var source) ? Text(source, "path") : null, Text(point, "message"));

    private async Task<JsonElement> Request(string command, object arguments, CancellationToken token, DapConnection? selected = null)
    {
        DapConnection current;
        lock (gate) current = selected ?? connection ?? throw new DebugFailure("invalid_state", "Launch a program first.");
        try { return await current.Request(command, arguments, token); }
        catch (Exception exception) when (exception is OperationCanceledException || exception is DebugFailure { Code: "backend_timeout" or "backend_exited" })
        {
            Task pendingCleanup;
            lock (gate) { Failed(exception, current); pendingCleanup = cleanup; }
            await pendingCleanup; throw;
        }
    }

    public async ValueTask DisposeAsync()
    {
        await Shutdown(null, CancellationToken.None);
        control.Dispose();
    }
}
