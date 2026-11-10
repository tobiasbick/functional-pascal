using System.Globalization;
using System.Threading.Channels;
using Fpas.Debugging.Native;

namespace Fpas.Debugging.Core;

internal sealed partial class WorkerSession : IAsyncDisposable
{
    private readonly object gate = new();
    private readonly CancellationTokenSource lifetime = new();
    private readonly SemaphoreSlim execution = new(1);
    private readonly Channel<DebugSnapshot> events = Channel.CreateBounded<DebugSnapshot>(64);
    private CoreClrSession? native;
    private Task? eventPump;
    private Task<CoreClrSession>? preparation;
    private Task? disposal;
    private DebugSnapshot snapshot = new(Guid.NewGuid().ToString("N"), "created");
    private long stopSequence;
    private bool stopping;

    internal DebugSnapshot Snapshot { get { lock (gate) return snapshot; } }
    internal IAsyncEnumerable<DebugSnapshot> Events(CancellationToken token) => events.Reader.ReadAllAsync(token);

    private void Publish(DebugSnapshot next)
    {
        snapshot = next with { Revision = snapshot.Revision + 1 };
        if (!events.Writer.TryWrite(snapshot)) throw new DebugException("limit_exceeded", "The debugger event queue is full.");
    }

    internal async Task<DebugSnapshot> Prepare(LaunchTarget target)
    {
        Task<CoreClrSession> pending;
        lock (gate)
        {
            if (stopping || snapshot.State != "created") throw new DebugException("invalid_state", "This worker already owns a launch.");
            Publish(snapshot with { State = "starting" });
            preparation = pending = CoreClrSession.Prepare(target.Dotnet, target.Program, target.Cwd,
                target.InitialBreakpoint, lifetime.Token, arguments: target.Arguments, terminal: target.Terminal);
        }
        CoreClrSession prepared;
        try { prepared = await pending; }
        catch (Exception exception) { Failed(exception); throw; }
        lock (gate)
        {
            native = prepared;
            if (!stopping)
            {
                Publish(snapshot with { ProcessId = native.ProcessId });
                eventPump = ObserveStops(prepared);
            }
        }
        if (stopping) { await prepared.DisposeAsync(); throw new OperationCanceledException(lifetime.Token); }
        return Snapshot;
    }

    internal async Task<DebugSnapshot> Initialize()
    {
        await execution.WaitAsync(lifetime.Token);
        try
        {
            CoreClrSession selected;
            lock (gate)
            {
                if (snapshot.State != "starting" || native is null || stopping) throw new DebugException("invalid_state", "Prepare a launch first.");
                selected = native;
                Publish(snapshot with { Reason = "runtime_startup" });
            }
            try { await selected.Initialize(lifetime.Token); }
            catch (Exception exception) { Failed(exception); throw; }
            lock (gate)
            {
                if (snapshot.State == "starting") Publish(snapshot with { State = "running", Reason = null });
                return snapshot;
            }
        }
        finally { execution.Release(); }
    }

    private async Task ObserveStops(CoreClrSession selected)
    {
        try
        {
            while (!lifetime.IsCancellationRequested)
            {
                var stop = await selected.WaitForEvent(lifetime.Token);
                lock (gate)
                {
                    if (stopping) return;
                    if (stop.Reason == "breakpoints_changed") Publish(snapshot with { Breakpoints = stop.ChangedBreakpoints });
                    else if (stop.Reason == "modules_changed") Publish(snapshot with { ModuleIssues = stop.ModuleIssues });
                    else if (stop.Reason == "async_progress") { if (snapshot.State == "running") Publish(snapshot with { AsyncStep = stop.AsyncStep }); }
                    else Publish(snapshot with { State = "stopped", StopId = ++stopSequence, Reason = stop.Reason,
                        HitBreakpoints = stop.Breakpoints, Breakpoints = stop.ChangedBreakpoints, ModuleIssues = stop.ModuleIssues, AsyncStep = null });
                }
            }
        }
        catch (ChannelClosedException exception) when (exception.InnerException is null)
        {
            try
            {
                var status = await selected.ExitStatus(lifetime.Token);
                lock (gate) if (!stopping) Publish(snapshot with { State = "exited", StopId = null, Reason = null, ExitCode = status.Code, ExitSignal = status.Signal, AsyncStep = null });
            }
            catch (OperationCanceledException) when (lifetime.IsCancellationRequested) { }
            catch (Exception failure) { Failed(failure); }
        }
        catch (ChannelClosedException exception) { Failed(exception.InnerException ?? exception); }
        catch (OperationCanceledException) when (lifetime.IsCancellationRequested) { }
        catch (Exception exception) { Failed(exception); }
    }

    internal async Task<DebugSnapshot> Resume(long stopId, string kind)
    {
        if (kind is not ("continue" or "into" or "over" or "out" or "detach")) throw new DebugException("invalid_argument", "Unknown resume kind.");
        await execution.WaitAsync(lifetime.Token);
        try
        {
            lock (gate)
            {
                CheckStop(stopId); var selected = native!;
                void Running() => Publish(snapshot with { State = "running", StopId = null, Reason = null, HitBreakpoints = null });
                try
                {
                    switch (kind)
                    {
                        case "continue": selected.Continue(Running); break;
                        case "detach": selected.DetachAtUnhandledException(Running); break;
                        default: selected.Step(kind, Running); break;
                    }
                }
                catch (DebugException exception) when (exception.Code is "unavailable" or "ambiguous_caller" or "limit_exceeded"
                    && selected.IsStopped && snapshot.State == "stopped") { throw; }
                catch (Exception exception) { Failed(exception); throw; }
                return snapshot;
            }
        }
        finally { execution.Release(); }
    }

    internal DebugBinding[] Bindings(long stopId)
    {
        lock (gate)
        {
            CheckStop(stopId);
            return native!.IntegerLocals().Select(pair => new DebugBinding(pair.Key, "int", "integer", pair.Value.ToString(CultureInfo.InvariantCulture))).ToArray();
        }
    }

    internal async Task<DebugSnapshot> Pause()
    {
        await execution.WaitAsync(lifetime.Token);
        try
        {
            CoreClrSession selected;
            lock (gate)
            {
                if (!stopping && snapshot.State is "stopped" or "exited") return snapshot;
                if (snapshot.State != "running" || stopping) throw new DebugException("invalid_state", "Only a running target can be paused.");
                selected = native!;
            }
            selected.Pause();
            return Snapshot;
        }
        finally { execution.Release(); }
    }

    private void CheckStop(long stopId)
    {
        if (stopping || snapshot.State != "stopped" || snapshot.StopId != stopId)
            throw new DebugException("stale_reference", "Use the current stopped session identity.");
    }

    internal void Failed(Exception exception)
    {
        lock (gate)
        {
            if (stopping || snapshot.State == "failed") return;
            Publish(snapshot with { State = "failed", StopId = null, AsyncStep = null, Error = new(
                exception is DebugException failure ? failure.Code : "runtime_failure", exception.Message) });
        }
    }

    public ValueTask DisposeAsync() { lock (gate) return new(disposal ??= Task.Run(DisposeCore)); }

    private async Task DisposeCore()
    {
        CoreClrSession? selected;
        Task<CoreClrSession>? preparing;
        lock (gate) { stopping = true; selected = native; preparing = preparation; }
        lifetime.Cancel();
        try
        {
            if (selected is null && preparing is not null)
            {
                try { selected = await preparing; }
                catch (Exception) { } // Native preparation cleans a failed launch before returning its error.
            }
            await execution.WaitAsync();
            try { if (selected is not null) await selected.DisposeAsync(); }
            finally { execution.Release(); }
        }
        finally
        {
            if (eventPump is not null) await eventPump;
            lock (gate)
            {
                snapshot = snapshot with { State = snapshot.State == "failed" ? "failed" : "exited", Revision = snapshot.Revision + 1, StopId = null, AsyncStep = null };
                events.Writer.TryComplete();
            }
            lifetime.Dispose();
        }
    }
}
