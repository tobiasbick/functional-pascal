using Fpas.Debugging.Protocol;

namespace Fpas.Debugging.Sessions;

internal sealed partial class DebugSession
{
    private async Task<SessionSnapshot> EnrichException(SessionSnapshot snapshot, CancellationToken token)
    {
        if (snapshot.State != "stopped" || snapshot.Reason != "exception" || snapshot.Exception is not null) return snapshot;
        DapConnection? current;
        lock (gate)
        {
            Session(snapshot.SessionId);
            if (stopId != snapshot.StopId || state != "stopped") return Snapshot();
            current = connection;
        }
        if (current is null || !current.Capabilities.TryGetProperty("supportsExceptionInfoRequest", out var supported) || !supported.GetBoolean()) return snapshot;
        try
        {
            var body = await Request("exceptionInfo", new { threadId = snapshot.ThreadId }, token, current);
            lock (gate)
            {
                Session(snapshot.SessionId);
                if (sessionId == snapshot.SessionId && stopId == snapshot.StopId && state == "stopped")
                {
                    body.TryGetProperty("details", out var details);
                    exceptionInfo = new(Text(body, "exceptionId") ?? "", Text(body, "description"), Text(body, "breakMode"),
                        details.ValueKind == System.Text.Json.JsonValueKind.Object ? Text(details, "typeName") : null,
                        details.ValueKind == System.Text.Json.JsonValueKind.Object ? Text(details, "stackTrace") : null);
                    Pulse();
                }
                return Snapshot();
            }
        }
        catch (DebugFailure failure) when (failure.Code == "backend_error")
        {
            lock (gate) { Session(snapshot.SessionId); return Snapshot(); }
        }
    }
}
