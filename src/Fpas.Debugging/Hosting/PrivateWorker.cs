using System.IO.Pipes;
using System.Text.Json;
using Fpas.Debugging.Core;

namespace Fpas.Debugging.Hosting;

public static class PrivateWorker
{
    public static bool IsInvocation(string[] arguments) => arguments.Length == 2 && arguments[0] == WorkerLauncher.WorkerArgument;
    public static bool IsTargetInvocation(string[] arguments) => arguments.Length == 2 && arguments[0] == WorkerLauncher.BrokerArgument;
    public static async Task<int> RunTargetAsync(string pipeName)
    {
        try { return await TargetBroker.Run(pipeName); }
        catch (Exception) { return 1; }
    }

    public static async Task<int> RunAsync(string pipeName)
    {
        try { return await RunCore(pipeName); }
        catch (Exception) { return 1; }
    }

    private static async Task<int> RunCore(string pipeName)
    {
        if (!pipeName.StartsWith("fpas-debug-", StringComparison.Ordinal) || pipeName.Length > 96) return 2;
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var pipe = new NamedPipeClientStream(".", pipeName, PipeDirection.InOut, PipeOptions.Asynchronous | PipeOptions.CurrentUserOnly);
        await pipe.ConnectAsync(deadline.Token);
        await using var channel = new WorkerChannel(pipe);
        await using var session = new WorkerSession();
        using var lifetime = new CancellationTokenSource();
        var events = Task.Run(async () =>
        {
            await foreach (var snapshot in session.Events(lifetime.Token))
                await channel.Send(new(0, "event", "state", JsonSerializer.SerializeToElement(snapshot, WorkerJson.Default.DebugSnapshot)), lifetime.Token);
        });
        var requests = new Dictionary<long, Task>();
        var stopping = false;
        try
        {
            while (!stopping && await channel.Read(lifetime.Token) is { } request)
            {
                if (request.Kind != "request" || request.Id <= 0) throw new IOException("Invalid debugger worker request.");
                foreach (var finished in requests.Where(pair => pair.Value.IsCompleted).Select(pair => pair.Key).ToArray())
                { await requests[finished]; requests.Remove(finished); }
                if (requests.Count >= 64 || requests.ContainsKey(request.Id)) throw new IOException("Debugger worker request limit exceeded.");
                stopping = request.Operation == "stop";
                requests.Add(request.Id, Task.Run(() => Dispatch(request, session, channel, lifetime.Token)));
            }
            await Task.WhenAll(requests.Values);
        }
        finally
        {
            await session.DisposeAsync();
            lifetime.Cancel();
            try { await events; } catch (OperationCanceledException) when (lifetime.IsCancellationRequested) { }
        }
        return 0;
    }

    private static async Task Dispatch(WorkerMessage request, WorkerSession session, WorkerChannel channel, CancellationToken token)
    {
        try
        {
            var payload = request.Payload;
            JsonElement result;
            switch (request.Operation)
            {
                case "prepare":
                    var target = payload?.Deserialize(WorkerJson.Default.LaunchTarget) ?? throw new ArgumentException("Missing launch target.");
                    result = JsonSerializer.SerializeToElement(await session.Prepare(target), WorkerJson.Default.DebugSnapshot); break;
                case "initialize": result = JsonSerializer.SerializeToElement(await session.Initialize(), WorkerJson.Default.DebugSnapshot); break;
                case "resume":
                    var resume = payload?.Deserialize(WorkerJson.Default.ResumeRequest) ?? throw new ArgumentException("Missing stopped context.");
                    result = JsonSerializer.SerializeToElement(await session.Resume(resume.StopId, resume.Kind), WorkerJson.Default.DebugSnapshot); break;
                case "breakpoints":
                    var configured = payload is null ? session.Breakpoints()
                        : await session.SetBreakpoints(payload.Value.Deserialize(WorkerJson.Default.BreakpointsRequest) ?? throw new ArgumentException("Missing breakpoint request."));
                    result = JsonSerializer.SerializeToElement(configured, WorkerJson.Default.BreakpointInfoArray); break;
                case "bindings":
                    var stop = payload?.Deserialize(WorkerJson.Default.StopRequest) ?? throw new ArgumentException("Missing stopped context.");
                    result = JsonSerializer.SerializeToElement(session.Bindings(stop.StopId), WorkerJson.Default.DebugBindingArray); break;
                case "exceptions":
                    var policy = payload?.Deserialize(WorkerJson.Default.ExceptionPolicy) ?? throw new ArgumentException("Missing exception policy.");
                    result = JsonSerializer.SerializeToElement(session.SetExceptions(policy), WorkerJson.Default.ExceptionPolicy); break;
                case "exceptionInfo":
                    var failure = payload?.Deserialize(WorkerJson.Default.StopRequest) ?? throw new ArgumentException("Missing stopped context.");
                    result = JsonSerializer.SerializeToElement(session.Exception(failure.StopId), WorkerJson.Default.DebugExceptionInfo); break;
                case "threads":
                    var threads = payload?.Deserialize(WorkerJson.Default.StopRequest) ?? throw new ArgumentException("Missing stopped context.");
                    result = JsonSerializer.SerializeToElement(session.Threads(threads.StopId), WorkerJson.Default.DebugThreadArray); break;
                case "stack":
                    var stack = payload?.Deserialize(WorkerJson.Default.StackRequest) ?? throw new ArgumentException("Missing stack request.");
                    result = JsonSerializer.SerializeToElement(session.Stack(stack), WorkerJson.Default.StackPage); break;
                case "frameBindings":
                    var frame = payload?.Deserialize(WorkerJson.Default.FrameRequest) ?? throw new ArgumentException("Missing frame context.");
                    result = JsonSerializer.SerializeToElement(session.FrameBindings(frame), WorkerJson.Default.BindingPage); break;
                case "valueChildren":
                    var value = payload?.Deserialize(WorkerJson.Default.ValueRequest) ?? throw new ArgumentException("Missing value context.");
                    result = JsonSerializer.SerializeToElement(session.ValueChildren(value), WorkerJson.Default.BindingPage); break;
                case "staticBindings":
                    var statics = payload?.Deserialize(WorkerJson.Default.StaticRequest) ?? throw new ArgumentException("Missing static context.");
                    result = JsonSerializer.SerializeToElement(session.StaticBindings(statics), WorkerJson.Default.BindingPage); break;
                case "evaluate":
                    var expression = payload?.Deserialize(WorkerJson.Default.EvaluationRequest) ?? throw new ArgumentException("Missing expression context.");
                    result = JsonSerializer.SerializeToElement(session.Evaluate(expression), WorkerJson.Default.EvaluationResult); break;
                case "status": result = JsonSerializer.SerializeToElement(session.Snapshot, WorkerJson.Default.DebugSnapshot); break;
                case "pause": result = JsonSerializer.SerializeToElement(await session.Pause(), WorkerJson.Default.DebugSnapshot); break;
                case "stop":
                    await session.DisposeAsync(); result = JsonSerializer.SerializeToElement(session.Snapshot, WorkerJson.Default.DebugSnapshot); break;
                default: throw new DebugException("unsupported_operation", "Unknown private debugger operation.");
            }
            await channel.Send(new(request.Id, "reply", request.Operation, result), token);
        }
        catch (Exception exception) when (exception is not OperationCanceledException || !token.IsCancellationRequested)
        {
            var problem = new DebugProblem(exception is DebugException failure ? failure.Code : exception is OperationCanceledException ? "cancelled"
                : exception is ArgumentException ? "invalid_argument" : "worker_failure", exception.Message);
            await channel.Send(new(request.Id, "reply", request.Operation, Error: problem), token);
        }
    }
}
