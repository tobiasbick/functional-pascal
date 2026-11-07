using System.Text.Json.Nodes;
using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;

namespace Fpas.Debugging.Dap;

internal sealed partial class DapServer
{
    private async Task<JsonObject> Execute(string command, JsonObject args, CancellationToken token)
    {
        if (command == "initialize")
        {
            if (initialized) throw new DebugException("invalid_state", "This adapter has already been initialized.");
            linesStartAt1 = args.Flag("linesStartAt1", true); columnsStartAt1 = args.Flag("columnsStartAt1", true);
            variablePaging = args.Flag("supportsVariablePaging");
            var format = args.Text("pathFormat", "path");
            if (format is not ("path" or "uri")) throw DapArguments.Invalid("pathFormat");
            uriPaths = format == "uri"; initialized = true;
            return new()
            {
                ["supportsConfigurationDoneRequest"] = true, ["supportsTerminateRequest"] = true, ["supportsCancelRequest"] = true,
                ["supportsEvaluateForHovers"] = true, ["supportsExceptionInfoRequest"] = true,
                ["exceptionBreakpointFilters"] = new JsonArray(
                    new JsonObject { ["filter"] = "unhandled", ["label"] = "Unhandled exceptions", ["default"] = true },
                    new JsonObject { ["filter"] = "raised", ["label"] = "Raised exceptions", ["default"] = false })
            };
        }
        if (!initialized) throw new DebugException("invalid_state", "Initialize the adapter first.");
        switch (command)
        {
            case "launch":
                if (launched) throw new DebugException("invalid_state", "An adapter supports one launched target.");
                var program = PathFromClient(args.Text("program"));
                var cwd = args["cwd"] is null ? Path.GetDirectoryName(program)! : PathFromClient(args.Text("cwd"));
                if (!Path.IsPathFullyQualified(program) || !Path.IsPathFullyQualified(cwd)) throw new ArgumentException("DAP program and cwd must be absolute paths.");
                if (args.Flag("noDebug") || args["env"] is not null || args["stdin"] is not null || args["console"] is { } console && console.GetValue<string>() != "internalConsole")
                    throw new DebugException("unsupported", "Launch supports an internal console with closed target stdin and inherited environment.");
                var argv = args.Array("args").Select(item => item?.GetValue<string>() ?? throw DapArguments.Invalid("args")).ToArray();
                TerminalLaunch? terminal;
                if (args.Flag("fpasTui"))
                {
                    var dimensions = args.Object("fpasTerminal");
                    var size = ReadSize(dimensions);
                    tui = new(size.Columns, size.Rows);
                    terminal = tui.Launch;
                }
                else terminal = args["fpasTerminal"] is null ? null : ReadTerminal(args.Object("fpasTerminal"));
                stopAtEntry = args.Flag("stopAtEntry"); launched = true;
                host = await DebugSessionHost.Start(WorkerLauncher.Current(), cwd, token);
                await host.Prepare(new(dotnet, program, cwd, Arguments: argv, Terminal: terminal), token);
                await host.Initialize(token); var entry = await host.WaitForStop(token);
                if (entry.State != "stopped") throw new DebugException("startup_failure", entry.Error?.Message ?? "The target did not reach entry.");
                return new();
            case "configurationDone":
                if (configured) throw new DebugException("invalid_state", "Configuration is already complete.");
                _ = Stop; configured = true;
                if (!stopAtEntry) await Resume("continue", token);
                return new();
            case "setBreakpoints":
                var source = PathFromClient(args.Object("source").Text("path"));
                var points = args.Array("breakpoints");
                if (points.Count > 64) throw new DebugException("limit_exceeded", "At most 64 source breakpoints are supported.");
                var positions = points.Select(item =>
                {
                    var point = item as JsonObject ?? throw DapArguments.Invalid("breakpoints");
                    if (point["condition"] is not null || point["hitCondition"] is not null || point["logMessage"] is not null)
                        throw new DebugException("unsupported", "Conditional breakpoints, hit conditions and logpoints are unsupported.");
                    return new BreakpointPosition(FromLine(point.Number("line")), point["column"] is null ? null : FromColumn(point.Number("column")));
                }).ToArray();
                var bound = await Host.SetBreakpoints(new(source, positions), token);
                foreach (var point in bound) knownBreakpoints[point.Id] = point.State;
                return new() { ["breakpoints"] = new JsonArray(bound.Select(point => (JsonNode)Breakpoint(point)).ToArray()) };
            case "setExceptionBreakpoints":
                if (args["filterOptions"] is not null || args["exceptionOptions"] is not null) throw new DebugException("unsupported", "Use raised/unhandled exception filter names.");
                var filters = args.Array("filters").Select(item => item?.GetValue<string>() ?? throw DapArguments.Invalid("filters")).ToArray();
                if (filters.Any(item => item is not ("raised" or "unhandled"))) throw new DebugException("unsupported", "Unknown exception filter.");
                await Host.SetExceptions(new(filters.Contains("raised", StringComparer.Ordinal), filters.Contains("unhandled", StringComparer.Ordinal)), token);
                return new();
            case "continue": case "next": case "stepIn": case "stepOut":
                if (!configured) throw new DebugException("invalid_state", "Finish configuration before resuming.");
                if (args.Flag("singleThread") || args.Text("granularity", "statement") != "statement") throw new DebugException("unsupported", "The debugger supports all-stop statement stepping.");
                await ValidateThread(args, token, stopThread: command != "continue");
                await Resume(command switch { "next" => "over", "stepIn" => "into", "stepOut" => "out", _ => "continue" }, token);
                return command == "continue" ? new() { ["allThreadsContinued"] = true } : new();
            case "pause":
                await Host.Pause(token); return new();
            case "fpasTuiSnapshot":
                return Tui.Snapshot(Host.Snapshot.State == "stopped", args.Flag("cells"));
            case "fpasTuiSend":
                return Host.Snapshot.State == "stopped" ? new() { ["status"] = "paused" } : await Tui.Forward("send", args, token);
            case "fpasTuiWait":
                if (args["frame"] is not null) return await Tui.WaitFrame(args, token);
                if (args["inputId"] is JsonValue inputId && inputId.TryGetValue<long>(out var id))
                    return await Tui.WaitInput(id, args.Number("timeoutMs", 5000), () => Host.Snapshot.State == "stopped", token);
                return await WaitForTuiQueue(args, token);
            case "fpasTuiResize":
                var requested = ReadSize(args);
                await Tui.Resize(requested.Columns, requested.Rows, token);
                return new();
            case "fpasTuiTerminalInput":
                await Tui.WriteTerminal(Convert.FromBase64String(args.Text("bytes")), token);
                return new();
            case "cancel":
                var cancelled = args.Number("requestId", 0);
                if (pending.TryGetValue(cancelled, out var waiting)) { try { waiting.Cancel(); } catch (ObjectDisposedException) { } }
                return new();
            case "disconnect": case "terminate":
                if (command == "disconnect" && !args.Flag("terminateDebuggee", true))
                    throw new DebugException("unsupported", "Disconnect must terminate the debugger-owned target.");
                if (host is not null) await host.DisposeAsync();
                return new();
            case "threads": return await ThreadList(token);
            case "stackTrace": return await Stack(args, token);
            case "scopes": return await Scopes(args, token);
            case "variables": return await Variables(args, token);
            case "evaluate": return await Evaluate(args, token);
            case "exceptionInfo": return await Exception(args, token);
            default: throw new DebugException("unsupported_request", "Unsupported DAP request: " + command + ".");
        }
    }

    private static TerminalLaunch ReadTerminal(JsonObject args)
    {
        var pipe = args.Text("pipe");
        if (!pipe.StartsWith("fpas-terminal-", StringComparison.Ordinal) ||
            !Guid.TryParseExact(pipe["fpas-terminal-".Length..], "N", out _))
            throw DapArguments.Invalid("fpasTerminal.pipe");
        var size = ReadSize(args);
        return new(pipe, size.Columns, size.Rows);
    }

    private static TerminalSize ReadSize(JsonObject args)
    {
        var columns = args.Number("columns", 80);
        var rows = args.Number("rows", 24);
        if (columns is < 1 or > 500 || rows is < 1 or > 300) throw DapArguments.Invalid("fpasTerminal dimensions");
        return new(columns, rows);
    }

    private TuiController Tui => tui ?? throw new DebugException("invalid_state", "Launch with fpasTui enabled first.");

    private async Task<JsonObject> WaitForTuiQueue(JsonObject args, CancellationToken token)
    {
        var timeout = args.Number("timeoutMs", 5000);
        if (timeout is < 0 or > 60_000) throw DapArguments.Invalid("timeoutMs");
        if (Host.Snapshot.State == "stopped") return new() { ["status"] = "paused" };
        using var deadline = CancellationTokenSource.CreateLinkedTokenSource(token);
        deadline.CancelAfter(timeout);
        var request = args.DeepClone().AsObject();
        request["timeoutMs"] = timeout;
        try { return await Tui.Forward("wait", request, deadline.Token); }
        catch (OperationCanceledException) when (!token.IsCancellationRequested)
        { return new() { ["status"] = Host.Snapshot.State == "stopped" ? "paused" : "timeout" }; }
    }

    private async Task ValidateThread(JsonObject args, CancellationToken token, bool stopThread = false)
    {
        var id = args.Number("threadId");
        var thread = (await Host.Threads(Stop, token)).FirstOrDefault(thread => thread.Id == id) ?? throw DapArguments.Invalid("threadId");
        if (stopThread && !thread.IsStopThread) throw new DebugException("unsupported_thread", "Source stepping is supported on the actual stop thread only.");
    }
    private async Task Resume(string kind, CancellationToken token)
    {
        var stop = Stop;
        await Host.Resume(stop, kind, token);
        frames.Clear(); values.Clear();
    }

    private int FromLine(int line) => checked(line + (linesStartAt1 ? 0 : 1));
    private int FromColumn(int column) => checked(column + (columnsStartAt1 ? 0 : 1));
    private int ToLine(int line) => line - (linesStartAt1 ? 0 : 1);
    private int ToColumn(int column) => column - (columnsStartAt1 ? 0 : 1);
    private string PathFromClient(string path)
    {
        if (!uriPaths) return Path.IsPathFullyQualified(path) ? Path.GetFullPath(path) : throw new ArgumentException("DAP source/target paths must be absolute.");
        if (!Uri.TryCreate(path, UriKind.Absolute, out var uri) || !uri.IsFile || !string.IsNullOrEmpty(uri.Host) && uri.Host != "localhost")
            throw new ArgumentException("Only local file: source/target URIs are supported.");
        return Path.GetFullPath(uri.LocalPath);
    }
    private JsonObject Source(string path) => new() { ["name"] = Path.GetFileName(path), ["path"] = uriPaths ? new Uri(path).AbsoluteUri : path };
    private JsonObject Breakpoint(BreakpointInfo point)
    {
        var location = point.Actual;
        return new() { ["id"] = checked((int)point.Id), ["verified"] = point.State == "bound", ["source"] = Source(location?.Path ?? point.Requested.Source),
            ["line"] = ToLine(location?.Line ?? point.Requested.Line), ["column"] = location is null ? point.Requested.Column is { } column ? ToColumn(column) : null : ToColumn(location.Column),
            ["message"] = point.Error?.Message ?? (point.State == "pending" ? "Pending module load." : null) };
    }
}
