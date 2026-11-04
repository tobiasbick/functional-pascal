using Fpas.Debugging.Protocol;
using System.Text.Json;

namespace Fpas.Debugging.Sessions;

internal sealed partial class DebugSession
{
    internal async Task<SessionSnapshot> Launch(string program, string cwd, string[] args, CancellationToken token)
    {
        LaunchInputs.Validate(program, cwd, args);
        program = Path.GetFullPath(program); cwd = Path.GetFullPath(cwd);
        await control.WaitAsync(token);
        try
        {
            lock (gate)
                if (state is "starting" or "running" or "stopped") throw new DebugFailure("invalid_state", "Stop the active program before launching another.");
            await ShutdownCore(token);
            lock (gate)
            {
                sessionId = Guid.NewGuid().ToString("N"); state = "starting"; stopping = false;
                revision = 0; stopId = null; stopSequence = 0; threadId = null; exitCode = null;
                error = null; reason = null; location = null; exceptionInfo = null; output = new(); breakpoints.Clear(); Pulse();
            }
            using var startup = CancellationTokenSource.CreateLinkedTokenSource(token);
            startup.CancelAfter(TimeSpan.FromSeconds(30));
            try
            {
                DapConnection? launched = null;
                launched = new(backendPath, cwd, Event, exception => Failed(exception, launched));
                connection = launched;
                await connection.Initialize(startup.Token);
                await Request("launch", new { program, cwd, args, stopAtEntry = true, justMyCode = true }, startup.Token);
                await Request("setExceptionBreakpoints", new { filters = new[] { "user-unhandled" } }, startup.Token);
                await Request("configurationDone", new { }, startup.Token);
                var snapshot = await Status(sessionId, -1, 30000, startup.Token);
                while (snapshot.State == "starting") snapshot = await Status(sessionId, snapshot.Event, 30000, startup.Token);
                if (snapshot.State != "stopped") throw new DebugFailure("backend_error", "Program did not reach its entry stop.");
                return snapshot;
            }
            catch (Exception exception)
            {
                lock (gate)
                {
                    state = "failed"; error = new(exception is DebugFailure failure ? failure.Code : "launch_failed", "The program could not be launched."); Pulse();
                }
                await ShutdownCore(CancellationToken.None);
                if (exception is OperationCanceledException && !token.IsCancellationRequested)
                    throw new DebugFailure("backend_timeout", "Program launch exceeded 30 seconds.");
                throw;
            }
        }
        finally { control.Release(); }
    }

    internal async Task<SessionSnapshot> Resume(string id, long expected, string command, CancellationToken token)
    {
        await control.WaitAsync(token);
        try
        {
            int thread; SessionSnapshot previous; long runningRevision; int[] frames, variables;
            lock (gate)
            {
                Stop(id, expected); previous = Snapshot(); thread = threadId ?? throw new DebugFailure("invalid_state", "No stopped thread.");
                frames = frameHandles.ToArray(); variables = variableHandles.ToArray();
                Running(); runningRevision = revision;
            }
            try { await Request(command, new { threadId = thread }, token); }
            catch (DebugFailure failure) when (failure.Code == "backend_error")
            {
                lock (gate)
                    if (revision == runningRevision)
                    {
                        state = previous.State; stopId = previous.StopId; threadId = previous.ThreadId;
                        reason = previous.Reason; location = previous.Location; exceptionInfo = previous.Exception;
                        frameHandles.UnionWith(frames); variableHandles.UnionWith(variables); Pulse();
                    }
                throw;
            }
            lock (gate) return Snapshot();
        }
        finally { control.Release(); }
    }

    internal async Task<SessionSnapshot> Pause(string id, CancellationToken token)
    {
        await control.WaitAsync(token);
        try
        {
            lock (gate) { Session(id); if (state != "running") throw new DebugFailure("invalid_state", "Only a running program can be paused."); }
            var threads = await Request("threads", new { }, token);
            var thread = threads.GetProperty("threads").EnumerateArray().Select(item => Integer(item, "id")).FirstOrDefault(id => id is > 0)
                ?? throw new DebugFailure("invalid_state", "The backend has no live thread to pause.");
            await Request("pause", new { threadId = thread }, token);
            lock (gate) return Snapshot();
        }
        finally { control.Release(); }
    }

    internal async Task<SessionSnapshot> Status(string id, long after, int wait, CancellationToken token)
    {
        if (wait is < 0 or > 30000) throw new DebugFailure("invalid_argument", "waitMilliseconds must be 0..30000.");
        Task notification; SessionSnapshot snapshot; bool shouldWait;
        lock (gate)
        {
            Session(id); snapshot = Snapshot(); notification = changed.Task;
            if (after < -1 || after > revision) throw new DebugFailure("invalid_argument", "Invalid event cursor.");
            shouldWait = after == revision && wait > 0 && state is not ("exited" or "failed");
        }
        if (shouldWait)
        {
            try { await notification.WaitAsync(TimeSpan.FromMilliseconds(wait), token); }
            catch (TimeoutException) { }
        }
        lock (gate) { Session(id); snapshot = Snapshot(); }
        return await EnrichException(snapshot, token);
    }

    internal async Task<SessionSnapshot> Shutdown(string? id, CancellationToken token)
    {
        await control.WaitAsync(token);
        try
        {
            lock (gate) { if (id is not null) Session(id); }
            await ShutdownCore(token);
            lock (gate)
            {
                if (state != "failed") state = "exited";
                stopId = null; frameHandles.Clear(); variableHandles.Clear(); Pulse(); return Snapshot();
            }
        }
        finally { control.Release(); }
    }

    private async Task ShutdownCore(CancellationToken token)
    {
        DapConnection? current;
        lock (gate) { stopping = true; current = connection; connection = null; }
        try
        {
            if (current is not null)
            {
                await Disconnect(current, token);
            }
            await cleanup;
        }
        finally { lock (gate) stopping = false; }
    }

    private static async Task Disconnect(DapConnection current, CancellationToken token)
    {
        using var shutdown = CancellationTokenSource.CreateLinkedTokenSource(token);
        shutdown.CancelAfter(TimeSpan.FromSeconds(5));
        try { await current.Request("disconnect", new { terminateDebuggee = true }, shutdown.Token); }
        catch (Exception exception) when (exception is DebugFailure or OperationCanceledException or IOException) { }
        finally { await current.DisposeAsync(); }
    }

    internal async Task<Reply<IReadOnlyList<BreakpointInfo>>> SetBreakpoints(string id, string source, int[] lines, CancellationToken token)
    {
        if (!Path.IsPathFullyQualified(source) || !File.Exists(source) || lines is null || lines.Length > 4096 || lines.Any(line => line <= 0))
            throw new DebugFailure("invalid_argument", "Provide an absolute existing source and up to 4096 positive line numbers.");
        source = Path.GetFullPath(source);
        await control.WaitAsync(token);
        try
        {
            lock (gate)
            {
                Session(id); if (state is not ("stopped" or "running")) throw new DebugFailure("invalid_state", "Launch a program first.");
                if (breakpoints.Count(point => point.Source != source) + lines.Distinct().Count() > 4096)
                    throw new DebugFailure("limit_exceeded", "Session breakpoint limit exceeded.");
            }
            var body = await Request("setBreakpoints", new { source = new { path = source }, breakpoints = lines.Distinct().Select(line => new { line }).ToArray() }, token);
            var points = body.GetProperty("breakpoints").EnumerateArray().Select(point => Breakpoint(point) with { Source = source }).ToArray();
            lock (gate)
            {
                var ids = points.Where(point => point.Id is not null).Select(point => point.Id).ToHashSet();
                breakpoints.RemoveAll(point => point.Source == source || (point.Id is not null && ids.Contains(point.Id)));
                if (breakpoints.Count + points.Length > 4096) throw new DebugFailure("limit_exceeded", "Session breakpoint limit exceeded.");
                breakpoints.AddRange(points); Pulse(); return new(Snapshot(), points);
            }
        }
        finally { control.Release(); }
    }

    internal Reply<OutputPage> Output(string id, long after, int limit)
    {
        lock (gate) { Session(id); return new(Snapshot(), output.Read(after, limit)); }
    }
}
