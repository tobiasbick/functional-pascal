using System.Collections.Concurrent;
using System.Text.Json.Nodes;
using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;

namespace Fpas.Debugging.Dap;

internal sealed partial class DapServer(string dotnet, Stream input, Stream output, CancellationToken cancellation)
{
    private readonly CancellationTokenSource lifetime = CancellationTokenSource.CreateLinkedTokenSource(cancellation);
    private readonly SemaphoreSlim operations = new(1);
    private readonly ConcurrentDictionary<int, CancellationTokenSource> pending = [];
    private readonly ConcurrentDictionary<int, Task> tasks = [];
    private readonly StopHandles<long> frames = new();
    private readonly StopHandles<ValueHandle> values = new();
    private DebugSessionHost? host;
    private DapTransport transport = null!;
    private Task monitor = Task.CompletedTask;
    private bool initialized, launched, configured, stopAtEntry, linesStartAt1 = true, columnsStartAt1 = true, uriPaths, terminal, variablePaging;
    private long reportedStop, outputCursor;
    private int lastRequest, closeRequest, failed;
    private readonly object terminalGate = new();
    private readonly Dictionary<long, string> knownBreakpoints = [];
    private sealed record ValueHandle(long Frame = 0, long Reference = 0, string View = "source", bool Static = false);

    internal async Task<int> Run()
    {
        transport = new(input, output, lifetime.Token); transport.Start();
        var result = 0;
        try
        {
            while (!lifetime.IsCancellationRequested)
            {
                var read = transport.Read();
                var done = await Task.WhenAny(read, transport.WriteTask).WaitAsync(lifetime.Token);
                if (done == transport.WriteTask) { await transport.WriteTask; if (terminal) break; throw new IOException("The DAP output transport closed."); }
                var message = await read;
                if (message is null) break;
                var id = message.Number("seq"); var command = message.Text("command");
                if (message.Text("type") != "request" || id < 1 || id <= lastRequest) throw new IOException("DAP requests require increasing positive sequence numbers.");
                lastRequest = id;
                var capacity = command is "disconnect" or "terminate" ? 64 : command == "cancel" ? 63 : command == "pause" ? 62 : 60;
                if (pending.Count >= capacity) { Reply(id, command, error: new("busy", "The bounded request queue reserves capacity for pause, cancellation and shutdown.")); continue; }
                var source = CancellationTokenSource.CreateLinkedTokenSource(lifetime.Token);
                if (!pending.TryAdd(id, source)) throw new IOException("Duplicate DAP request.");
                var task = Dispatch(id, command, message.Object("arguments"), source);
                tasks[id] = task;
                _ = task.ContinueWith(_ => tasks.TryRemove(id, out var ignored), CancellationToken.None, TaskContinuationOptions.ExecuteSynchronously, TaskScheduler.Default);
            }
        }
        catch (OperationCanceledException) when (lifetime.IsCancellationRequested) { result = cancellation.IsCancellationRequested ? 130 : Volatile.Read(ref failed); }
        catch (Exception failure) when (failure is not OutOfMemoryException)
        {
            result = 1;
            try { Event("output", new() { ["category"] = "stderr", ["output"] = "FPAS debugger transport failure: " + failure.Message + "\n" }); }
            catch (IOException) { }
        }
        finally
        {
            lifetime.Cancel();
            if (host is not null) { try { await host.DisposeAsync(); } catch (IOException) { result = 1; } }
            try { await Task.WhenAll(tasks.Values).WaitAsync(TimeSpan.FromSeconds(5)); await monitor; } catch (Exception) { }
            transport.Complete();
            try { await transport.WriteTask.WaitAsync(TimeSpan.FromSeconds(5)); } catch (Exception) { }
            lifetime.Dispose(); operations.Dispose();
        }
        return result;
    }

    private async Task Dispatch(int id, string command, JsonObject args, CancellationTokenSource request)
    {
        var held = false; var replied = false;
        try
        {
            request.CancelAfter(TimeSpan.FromSeconds(command == "launch" ? 30 : command is "disconnect" or "terminate" ? 5 : 10));
            if (command is "disconnect" or "terminate")
            {
                if (command == "disconnect" && !args.Flag("terminateDebuggee", true) || args.Flag("restart"))
                    throw new DebugException("unsupported", "Shutdown must terminate the owned target; detach and restart are unsupported.");
                if (Interlocked.CompareExchange(ref closeRequest, id, 0) != 0) throw new DebugException("invalid_state", "Shutdown is already in progress.");
                foreach (var item in pending.Where(item => item.Key != id)) { try { item.Value.Cancel(); } catch (ObjectDisposedException) { } }
            }
            else if (Volatile.Read(ref closeRequest) != 0) throw new DebugException("invalid_state", "The adapter is shutting down.");
            if (command is not ("cancel" or "disconnect" or "terminate" or "pause")) { await operations.WaitAsync(request.Token); held = true; }
            var body = await Execute(command, args, request.Token);
            if (command is "disconnect" or "terminate")
                await Task.WhenAll(tasks.Where(item => item.Key != id).Select(item => item.Value)).WaitAsync(request.Token);
            if (command is "disconnect" or "terminate")
            {
                await operations.WaitAsync(request.Token); held = true;
                await Output(); FlushOutputLoss();
            }
            Reply(id, command, body); replied = true;
            if (command is "continue" or "next" or "stepIn" or "stepOut")
                Event("continued", new() { ["threadId"] = args.Number("threadId"), ["allThreadsContinued"] = true });
            if (command == "launch") { Event("initialized", new()); monitor = Observe(); }
            if (command is "disconnect" or "terminate") { Terminated(); transport.Complete(); }
        }
        catch (OperationCanceledException)
        {
            if (!replied) Reply(id, command, error: new("cancelled", "The debugger operation was cancelled or timed out."));
            else { Interlocked.Exchange(ref failed, 1); lifetime.Cancel(); }
            if (host?.Snapshot.State == "failed") Terminated();
        }
        catch (Exception failure) when (failure is not OutOfMemoryException)
        {
            if (!replied) Reply(id, command, error: new(failure is DebugException debug ? debug.Code : "debugger_failure", failure.Message));
            else { Interlocked.Exchange(ref failed, 1); lifetime.Cancel(); }
            if (host?.Snapshot.State == "failed") Terminated();
        }
        finally { if (held) operations.Release(); pending.TryRemove(id, out _); request.Dispose(); }
    }

    private void Reply(int id, string command, JsonObject? body = null, DebugProblem? error = null)
    {
        var response = new JsonObject { ["type"] = "response", ["request_seq"] = id, ["command"] = command, ["success"] = error is null, ["body"] = body ?? new() };
        if (error is not null)
        {
            response["message"] = error.Code;
            response["body"] = new JsonObject { ["error"] = new JsonObject { ["id"] = 1, ["format"] = error.Message,
                ["showUser"] = error.Code is not ("notStopped" or "cancelled" or "stale_reference") } };
        }
        try { transport.Send(response); } catch (IOException) { Interlocked.Exchange(ref failed, 1); lifetime.Cancel(); throw; }
    }
    private void Event(string name, JsonObject body) => transport.Send(new() { ["type"] = "event", ["event"] = name, ["body"] = body });
    private void Terminated() { lock (terminalGate) { if (terminal) return; terminal = true; Event("terminated", new()); } }
    private DebugSessionHost Host => host ?? throw new DebugException("invalid_state", "Launch a target first.");
    private long Stop => Host.Snapshot is { State: "stopped", StopId: { } stop } ? stop : throw new DebugException("notStopped", "The target is not stopped.");
}
