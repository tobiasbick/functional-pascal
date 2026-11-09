using System.Collections.Concurrent;
using System.Diagnostics;
using System.IO.Pipes;
using System.Text.Json;
using System.Text.Json.Serialization.Metadata;
using Fpas.Debugging.Core;
using Fpas.Processes;

namespace Fpas.Debugging.Hosting;

internal sealed class DebugSessionHost : IAsyncDisposable
{
    private readonly object gate = new();
    private readonly NamedPipeServerStream pipe;
    private readonly CancellationTokenSource lifetime = new();
    private readonly ConcurrentDictionary<long, TaskCompletionSource<WorkerMessage>> pending = [];
    private readonly OutputCapture output;
    private readonly TimeSpan shutdownGrace;
    private readonly string pipeName = "fpas-debug-" + Guid.NewGuid().ToString("N");
    private OwnedProcess? worker;
    private WorkerChannel? channel;
    private Task reader = Task.CompletedTask, stdout = Task.CompletedTask, stderr = Task.CompletedTask, exit = Task.CompletedTask, input = Task.CompletedTask;
    private TaskCompletionSource changed = Signal();
    private DebugSnapshot snapshot = new("", "created");
    private Task? disposal;
    private Task<DebugSnapshot>? shutdown;
    private Task<DebugSnapshot>? stopAttempt;
    private long requestSequence;
    private long stopRequestId;
    private long cleanupStarted;
    private bool stopping, ending;

    private DebugSessionHost(int outputLimit, bool retainOutputPrefix, TimeSpan shutdownGrace)
    {
        this.shutdownGrace = shutdownGrace;
        output = new(outputLimit, retainOutputPrefix);
        pipe = new(pipeName, PipeDirection.InOut, 1, PipeTransmissionMode.Byte, PipeOptions.Asynchronous | PipeOptions.CurrentUserOnly);
    }
    private static TaskCompletionSource Signal() => new(TaskCreationOptions.RunContinuationsAsynchronously);
    internal DebugSnapshot Snapshot { get { lock (gate) return snapshot; } }
    internal OutputSnapshot Output(long after = 0) => output.Read(after);
    internal Task OutputLimitReached => output.LimitReached;
    internal int WorkerProcessId => worker?.Id ?? 0;

    internal static async Task<DebugSessionHost> Start(WorkerLauncher launcher, string cwd, CancellationToken token, ReadOnlyMemory<byte>? stdin = null,
        int outputLimit = OutputCapture.MaximumBytes, bool retainOutputPrefix = false, TimeSpan? shutdownGrace = null, ProcessStdio stdio = ProcessStdio.Capture)
    {
        token.ThrowIfCancellationRequested();
        if (stdin?.Length > 1_048_576) throw new ArgumentException("Explicit target input is bounded at 1 MiB.");
        if (stdin is not null && stdio != ProcessStdio.Capture) throw new ArgumentException("Explicit input requires captured stdin.");
        var grace = shutdownGrace ?? TimeSpan.FromSeconds(5);
        if (grace <= TimeSpan.Zero || grace > TimeSpan.FromSeconds(5)) throw new ArgumentOutOfRangeException(nameof(shutdownGrace));
        var host = new DebugSessionHost(outputLimit, retainOutputPrefix, grace);
        try
        {
            using var startup = CancellationTokenSource.CreateLinkedTokenSource(token);
            startup.CancelAfter(TimeSpan.FromSeconds(30));
            host.worker = OwnedProcess.Start(launcher.Executable, launcher.Arguments(host.pipeName), cwd, stdio: stdio);
            if (stdin is { } data) host.input = host.WriteInput(data); else if (stdio == ProcessStdio.Capture) host.worker.Input.Close();
            host.stdout = host.Capture(host.worker.Output, "stdout"); host.stderr = host.Capture(host.worker.Error, "stderr");
            var connection = host.pipe.WaitForConnectionAsync(startup.Token);
            var exited = host.worker.WaitForExit(startup.Token);
            if (await Task.WhenAny(connection, exited).WaitAsync(startup.Token) != connection && !connection.IsCompleted)
                throw new DebugException("worker_exited", "The debugger worker exited before connecting.");
            await connection;
            host.channel = new(host.pipe); host.reader = host.Read(); host.exit = host.ObserveExit();
            await host.Status(startup.Token);
            return host;
        }
        catch { await host.DisposeAsync(); throw; }
    }

    internal async Task<DebugSnapshot> Prepare(LaunchTarget target, CancellationToken token)
        => await Request("prepare", JsonSerializer.SerializeToElement(target, WorkerJson.Default.LaunchTarget), WorkerJson.Default.DebugSnapshot, token, 30);
    internal async Task<DebugSnapshot> Initialize(CancellationToken token)
        => await Request("initialize", null, WorkerJson.Default.DebugSnapshot, token, 30);
    internal async Task<DebugSnapshot> Status(CancellationToken token)
        => await Request("status", null, WorkerJson.Default.DebugSnapshot, token);
    internal async Task<DebugSnapshot> Pause(CancellationToken token)
        => await Request("pause", null, WorkerJson.Default.DebugSnapshot, token);
    internal Task<DebugSnapshot> Resume(long stopId, bool step, CancellationToken token) => Resume(stopId, step ? "over" : "continue", token);
    internal async Task<DebugSnapshot> Resume(long stopId, string kind, CancellationToken token)
        => await Request("resume", JsonSerializer.SerializeToElement(new ResumeRequest(stopId, kind), WorkerJson.Default.ResumeRequest), WorkerJson.Default.DebugSnapshot, token);
    internal async Task<BreakpointInfo[]> Breakpoints(CancellationToken token)
        => await Request("breakpoints", null, WorkerJson.Default.BreakpointInfoArray, token);
    internal async Task<BreakpointInfo[]> SetBreakpoints(BreakpointsRequest request, CancellationToken token)
        => await Request("breakpoints", JsonSerializer.SerializeToElement(request, WorkerJson.Default.BreakpointsRequest), WorkerJson.Default.BreakpointInfoArray, token);
    internal async Task<DebugBinding[]> Bindings(long stopId, CancellationToken token)
        => await Request("bindings", JsonSerializer.SerializeToElement(new StopRequest(stopId), WorkerJson.Default.StopRequest), WorkerJson.Default.DebugBindingArray, token);
    internal async Task<ExceptionPolicy> SetExceptions(ExceptionPolicy policy, CancellationToken token)
        => await Request("exceptions", JsonSerializer.SerializeToElement(policy, WorkerJson.Default.ExceptionPolicy), WorkerJson.Default.ExceptionPolicy, token);
    internal async Task<DebugExceptionInfo> ExceptionInfo(long stopId, CancellationToken token)
        => await Request("exceptionInfo", JsonSerializer.SerializeToElement(new StopRequest(stopId), WorkerJson.Default.StopRequest), WorkerJson.Default.DebugExceptionInfo, token);
    internal async Task<DebugThread[]> Threads(long stopId, CancellationToken token)
        => await Request("threads", JsonSerializer.SerializeToElement(new StopRequest(stopId), WorkerJson.Default.StopRequest), WorkerJson.Default.DebugThreadArray, token);
    internal async Task<StackPage> Stack(StackRequest request, CancellationToken token)
        => await Request("stack", JsonSerializer.SerializeToElement(request, WorkerJson.Default.StackRequest), WorkerJson.Default.StackPage, token);
    internal async Task<BindingPage> FrameBindings(FrameRequest request, CancellationToken token)
        => await Request("frameBindings", JsonSerializer.SerializeToElement(request, WorkerJson.Default.FrameRequest), WorkerJson.Default.BindingPage, token);
    internal async Task<BindingPage> ValueChildren(ValueRequest request, CancellationToken token)
        => await Request("valueChildren", JsonSerializer.SerializeToElement(request, WorkerJson.Default.ValueRequest), WorkerJson.Default.BindingPage, token);
    internal async Task<BindingPage> StaticBindings(StaticRequest request, CancellationToken token)
        => await Request("staticBindings", JsonSerializer.SerializeToElement(request, WorkerJson.Default.StaticRequest), WorkerJson.Default.BindingPage, token);
    internal async Task<EvaluationResult> Evaluate(EvaluationRequest request, CancellationToken token)
        => await Request("evaluate", JsonSerializer.SerializeToElement(request, WorkerJson.Default.EvaluationRequest), WorkerJson.Default.EvaluationResult, token);

    internal Task<DebugSnapshot> Stop(CancellationToken token)
    {
        lock (gate) return shutdown ??= Task.Run(() => StopCore(token));
    }

    private async Task<DebugSnapshot> StopCore(CancellationToken token)
    {
        Task? existing;
        lock (gate) { existing = disposal; if (existing is null) ending = true; }
        if (existing is not null) { await existing; return Snapshot; }
        try { return await OrderlyStop().WaitAsync(token); }
        catch (OperationCanceledException)
        {
            Failed(new(token.IsCancellationRequested ? "cancelled" : "worker_timeout", "The debugger stop did not complete."));
            throw;
        }
        finally { await DisposeAsync(); }
    }

    private Task<DebugSnapshot> OrderlyStop()
    {
        lock (gate)
        {
            if (cleanupStarted == 0) cleanupStarted = Stopwatch.GetTimestamp();
            return stopAttempt ??= Task.Run(SendStop);
        }
    }

    private TimeSpan CleanupRemaining => TimeSpan.FromTicks(Math.Max(1, (shutdownGrace - Stopwatch.GetElapsedTime(cleanupStarted)).Ticks));

    // Shutdown has its own deadline and bypasses ordinary request cancellation.
    private async Task<DebugSnapshot> SendStop()
    {
        var selected = channel ?? throw new IOException("The debugger worker is not connected.");
        var id = Interlocked.Increment(ref requestSequence);
        Interlocked.Exchange(ref stopRequestId, id);
        var completion = new TaskCompletionSource<WorkerMessage>(TaskCreationOptions.RunContinuationsAsynchronously);
        pending[id] = completion;
        using var grace = new CancellationTokenSource(TimeSpan.FromTicks(Math.Min((shutdownGrace / 2).Ticks, CleanupRemaining.Ticks)));
        try
        {
            await selected.Send(new(id, "request", "stop"), grace.Token);
            var response = await completion.Task.WaitAsync(grace.Token);
            if (response.Error is { } error) throw new DebugException(error.Code, error.Message);
            var result = response.Payload?.Deserialize(WorkerJson.Default.DebugSnapshot) ?? throw new IOException("Missing debugger shutdown state.");
            await worker!.WaitForExit(grace.Token);
            return result;
        }
        finally { pending.TryRemove(id, out _); }
    }

    internal async Task<DebugSnapshot> WaitForChange(long after, CancellationToken token)
    {
        Task notification;
        lock (gate)
        {
            if (snapshot.Revision != after || snapshot.State is "failed" or "exited") return snapshot;
            notification = changed.Task;
        }
        await notification.WaitAsync(token);
        return Snapshot;
    }

    internal async Task<DebugSnapshot> WaitForStop(CancellationToken token)
    {
        while (true)
        {
            Task notification;
            lock (gate)
            {
                if (snapshot.State is "stopped" or "exited" or "failed") return snapshot;
                notification = changed.Task;
            }
            await notification.WaitAsync(token);
        }
    }

    private async Task<T> Request<T>(string operation, JsonElement? payload, JsonTypeInfo<T> type, CancellationToken token, int seconds = 10) where T : class
    {
        var id = Interlocked.Increment(ref requestSequence);
        var completion = new TaskCompletionSource<WorkerMessage>(TaskCreationOptions.RunContinuationsAsynchronously);
        lock (gate)
        {
            if (stopping || channel is null) throw new DebugException("invalid_state", "The debugger worker is closed.");
            if (pending.Count >= 63) throw new DebugException("busy", "The debugger request limit was reached."); // Reserve one slot for shutdown.
            pending[id] = completion;
        }
        using var deadline = CancellationTokenSource.CreateLinkedTokenSource(token, lifetime.Token);
        deadline.CancelAfter(TimeSpan.FromSeconds(seconds));
        try
        {
            await channel.Send(new(id, "request", operation, payload), deadline.Token);
            var response = await completion.Task.WaitAsync(deadline.Token);
            if (response.Error is { } error) throw new DebugException(error.Code, error.Message);
            return response.Payload?.Deserialize(type) ?? throw new IOException("The debugger worker returned no result.");
        }
        catch (OperationCanceledException)
        {
            Failed(new(token.IsCancellationRequested ? "cancelled" : "worker_timeout", "The debugger operation did not complete."));
            await DisposeAsync(); throw;
        }
        catch (Exception exception) when (exception is IOException or ObjectDisposedException)
        {
            Failed(new("worker_exited", "The debugger worker connection failed."));
            await DisposeAsync(); throw;
        }
        finally { pending.TryRemove(id, out _); }
    }

    private async Task Read()
    {
        try
        {
            while (await channel!.Read(lifetime.Token) is { } message)
            {
                if (message.Operation == "state" && message.Kind == "event")
                    Update(message.Payload?.Deserialize(WorkerJson.Default.DebugSnapshot) ?? throw new IOException("Missing debugger state."));
                else if (message.Kind == "reply" && pending.TryGetValue(message.Id, out var completion))
                {
                    if (message.Error is { Code: "worker_failure" } problem) Failed(problem);
                    if (message.Error is null && message.Operation is "status" or "prepare" or "initialize" or "resume" or "stop" or "pause")
                        Update(message.Payload?.Deserialize(WorkerJson.Default.DebugSnapshot) ?? throw new IOException("Missing debugger state."));
                    completion.TrySetResult(message);
                }
                else throw new IOException("Unexpected debugger worker message.");
            }
            if (!stopping && !ending) Failed(new("worker_exited", "The debugger worker disconnected."));
        }
        catch (OperationCanceledException) when (lifetime.IsCancellationRequested) { }
        catch (Exception) { if (!stopping && !ending) Failed(new("worker_exited", "The debugger worker connection failed.")); }
    }

    private void Update(DebugSnapshot value)
    {
        var failed = false;
        lock (gate)
        {
            if (snapshot.State == "failed") return;
            if (snapshot.SessionId.Length != 0 && value.SessionId != snapshot.SessionId) throw new IOException("The debugger worker changed session identity.");
            if (value.Revision < snapshot.Revision) return;
            snapshot = value; Pulse();
            failed = value.State == "failed";
        }
        if (failed) _ = CleanupAfterFailure();
    }
    private void Pulse() { var previous = changed; changed = Signal(); previous.TrySetResult(); }
    private void Failed(DebugProblem problem)
    {
        lock (gate)
        {
            if (stopping || snapshot.State == "failed") return;
            snapshot = snapshot with { State = "failed", StopId = null, AsyncStep = null, Revision = snapshot.Revision + 1, Error = problem }; Pulse();
        }
        foreach (var request in pending)
            if (request.Key != Interlocked.Read(ref stopRequestId))
                request.Value.TrySetException(new DebugException(problem.Code, problem.Message));
        _ = CleanupAfterFailure();
    }

    private async Task CleanupAfterFailure()
    {
        try { await DisposeAsync(); }
        catch (Exception)
        {
            lock (gate)
            {
                snapshot = snapshot with { State = "failed", Revision = snapshot.Revision + 1,
                    Error = new(snapshot.Error?.Code ?? "cleanup_failed", (snapshot.Error?.Message ?? "Debugger shutdown failed.") + " Owned cleanup also reported a failure.") };
                Pulse();
            }
        }
    }

    private async Task Capture(Stream stream, string category)
    {
        var buffer = new byte[4096];
        try
        {
            while (await stream.ReadAsync(buffer, lifetime.Token) is var count && count != 0)
                output.AddBytes(category, buffer.AsSpan(0, count));
        }
        catch (OperationCanceledException) when (lifetime.IsCancellationRequested) { }
        catch (Exception exception) when (stopping && exception is IOException or ObjectDisposedException) { }
        finally { output.Complete(category); }
    }

    private async Task ObserveExit()
    {
        try
        {
            await worker!.WaitForExit(lifetime.Token);
            if (!stopping && !ending) Failed(new("worker_exited", "The debugger worker exited unexpectedly."));
        }
        catch (OperationCanceledException) when (lifetime.IsCancellationRequested) { }
    }

    public ValueTask DisposeAsync() { lock (gate) return new(disposal ??= Task.Run(DisposeCore)); }

    private async Task DisposeCore()
    {
        lock (gate) stopping = true;
        Exception? failure = null;
        async Task Release(Func<Task> action)
        {
            try { await action(); }
            catch (Exception exception) { failure ??= exception; }
        }
        try
        {
            // Let a responsive worker unregister callbacks and reap its target first.
            // A blocked native operation cannot prevent the bounded kill fallback.
            if (channel is not null)
            {
                try { await OrderlyStop(); }
                catch (Exception) { }
            }
            if (worker is not null) await Release(worker.TerminateAsync);
            await Release(() => Task.WhenAll(stdout, stderr, input).WaitAsync(CleanupRemaining));
        }
        finally
        {
            lifetime.Cancel();
            if (channel is not null) await Release(() => channel.DisposeAsync().AsTask());
            else await Release(() => pipe.DisposeAsync().AsTask());
            if (worker is not null) await Release(() => worker.DisposeAsync().AsTask());
            await Release(() => Task.WhenAll(reader, exit).WaitAsync(CleanupRemaining));
            foreach (var completion in pending.Values) completion.TrySetException(new DebugException("worker_exited", "The debugger worker is closed."));
            lifetime.Dispose();
        }
        if (failure is not null) throw new IOException("Debugger worker cleanup reported a failure.", failure);
    }

    private async Task WriteInput(ReadOnlyMemory<byte> data)
    {
        try { await worker!.Input.WriteAsync(data, lifetime.Token); await worker.Input.FlushAsync(lifetime.Token); }
        catch (OperationCanceledException) when (lifetime.IsCancellationRequested || stopping) { }
        catch (Exception exception) when (exception is IOException or ObjectDisposedException)
        { if (!stopping && !ending) Failed(new("input_failure", "The target input pipe failed before input delivery completed.")); }
        finally { worker!.Input.Close(); }
    }
}
