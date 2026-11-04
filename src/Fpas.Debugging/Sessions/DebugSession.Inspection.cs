using Fpas.Debugging.Protocol;
using System.Globalization;
using System.Text.Json;

namespace Fpas.Debugging.Sessions;

internal sealed partial class DebugSession
{
    private static void Page(int start, int count)
    {
        if (start < 0 || count is < 1 or > 128) throw new DebugFailure("invalid_argument", "start must be nonnegative and count must be 1..128.");
    }
    private int Handle(string value, string prefix, HashSet<int> known)
    {
        prefix = ReferencePrefix(prefix);
        if (string.IsNullOrEmpty(value) || !value.StartsWith(prefix, StringComparison.Ordinal) || !int.TryParse(value.AsSpan(prefix.Length), NumberStyles.None, CultureInfo.InvariantCulture, out var id) || !known.Contains(id))
            throw new DebugFailure("stale_reference", "Use a handle returned for the current stop.");
        return id;
    }
    private string ReferencePrefix(string prefix) => prefix + sessionId + ":" + stopId?.ToString(CultureInfo.InvariantCulture) + ":";
    private string Reference(string prefix, int id) => ReferencePrefix(prefix) + id.ToString(CultureInfo.InvariantCulture);
    private string? VariableHandle(JsonElement value)
    {
        var reference = Integer(value, "variablesReference") ?? 0;
        if (reference <= 0) return null;
        if (variableHandles.Count >= 4096 && !variableHandles.Contains(reference)) throw new DebugFailure("limit_exceeded", "Variable handle limit exceeded for this stop.");
        variableHandles.Add(reference); return Reference("v:", reference);
    }

    internal async Task<Reply<IReadOnlyList<ThreadInfo>>> Threads(string id, long expected, CancellationToken token)
    {
        DapConnection current;
        lock (gate) { Stop(id, expected); current = connection!; }
        var body = await Request("threads", new { }, token, current);
        lock (gate)
        {
            Stop(id, expected);
            var threads = body.GetProperty("threads").EnumerateArray().Take(129).ToArray();
            if (threads.Length > 128) throw new DebugFailure("limit_exceeded", "Backend returned more than 128 threads.");
            return new(Snapshot(), threads.Select(thread => new ThreadInfo(Integer(thread, "id") ?? 0, Text(thread, "name") ?? "")).ToArray());
        }
    }

    internal async Task<Reply<StackPage>> Stack(string id, long expected, int? thread, int start, int count, CancellationToken token)
    {
        Page(start, count);
        if (thread is <= 0) throw new DebugFailure("invalid_argument", "Use a positive threadId from debug_threads, or omit it for the stopping thread.");
        int selected; DapConnection current;
        lock (gate) { Stop(id, expected); current = connection!; selected = thread ?? threadId ?? throw new DebugFailure("invalid_state", "No stopped thread."); }
        var body = await Request("stackTrace", new { threadId = selected, startFrame = start, levels = count }, token, current);
        lock (gate)
        {
            Stop(id, expected);
            var frames = body.GetProperty("stackFrames").EnumerateArray().Take(count).Select(frame =>
            {
                var handle = Integer(frame, "id") ?? throw new InvalidDataException("Missing frame id.");
                if (frameHandles.Count >= 4096 && !frameHandles.Contains(handle)) throw new DebugFailure("limit_exceeded", "Frame handle limit exceeded for this stop.");
                frameHandles.Add(handle);
                return new FrameInfo(Reference("f:", handle), Text(frame, "name") ?? "", Position(frame));
            }).ToArray();
            var total = Integer(body, "totalFrames");
            if (start == 0 && selected == threadId && frames.Length > 0 && location != frames[0].Location)
            {
                location = frames[0].Location; Pulse();
            }
            return new(Snapshot(), new(frames, start, total, total is { } known ? (long)start + frames.Length < known : frames.Length == count));
        }
    }

    internal async Task<Reply<VariablesPage>> Variables(string id, long expected, string? frame, string? variables, int start, int count, CancellationToken token)
    {
        // Value displays/getters may execute target code; keep their stop stable until the reply.
        await control.WaitAsync(token);
        try { return await ReadVariables(id, expected, frame, variables, start, count, token); }
        finally { control.Release(); }
    }

    private async Task<Reply<VariablesPage>> ReadVariables(string id, long expected, string? frame, string? variables, int start, int count, CancellationToken token)
    {
        Page(start, count);
        if ((frame is null) == (variables is null)) throw new DebugFailure("invalid_argument", "Supply exactly one frameHandle or variablesHandle.");
        int reference; DapConnection current;
        lock (gate) { Stop(id, expected); current = connection!; reference = frame is not null ? Handle(frame, "f:", frameHandles) : Handle(variables!, "v:", variableHandles); }
        if (frame is not null)
        {
            if (start != 0) throw new DebugFailure("invalid_argument", "Scope discovery uses start=0; expand a returned scope handle for paged values.");
            var scopes = await Request("scopes", new { frameId = reference }, token, current);
            lock (gate)
            {
                Stop(id, expected);
                var items = scopes.GetProperty("scopes").EnumerateArray().Take(count + 1).ToArray();
                return new(Snapshot(), new(items.Take(count).Select(scope => new ScopeInfo(Text(scope, "name") ?? "", VariableHandle(scope),
                    scope.TryGetProperty("expensive", out var expensive) && expensive.GetBoolean())).ToArray(), [], 0, items.Length > count));
            }
        }
        var body = await Request("variables", new { variablesReference = reference, start, count = count + 1 }, token, current);
        lock (gate)
        {
            Stop(id, expected);
            var items = body.GetProperty("variables").EnumerateArray().Take(count + 1).ToArray();
            return new(Snapshot(), new([], items.Take(count).Select(value => new VariableInfo(Text(value, "name") ?? "", Text(value, "value") ?? "",
                Text(value, "type"), VariableHandle(value), Text(value, "evaluateName"))).ToArray(), start, items.Length > count));
        }
    }

    internal async Task<Reply<Evaluation>> Evaluate(string id, long expected, string frame, string expression, CancellationToken token)
    {
        await control.WaitAsync(token);
        try { return await EvaluateStopped(id, expected, frame, expression, token); }
        finally { control.Release(); }
    }

    private async Task<Reply<Evaluation>> EvaluateStopped(string id, long expected, string frame, string expression, CancellationToken token)
    {
        if (string.IsNullOrWhiteSpace(expression) || expression.Length > 8192) throw new DebugFailure("invalid_argument", "expression must contain 1..8192 characters in backend syntax.");
        int handle; DapConnection current;
        lock (gate) { Stop(id, expected); current = connection!; handle = Handle(frame, "f:", frameHandles); }
        var body = await Request("evaluate", new { frameId = handle, expression, context = "watch" }, token, current);
        lock (gate)
        {
            Stop(id, expected);
            return new(Snapshot(), new(Text(body, "result") ?? "", Text(body, "type"), VariableHandle(body)));
        }
    }
}
