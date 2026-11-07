using System.Runtime.InteropServices;
using System.Security.Cryptography;
using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;
using Fpas.Debugging.Symbols;

namespace Fpas.Debugging.Batch;

internal static class BatchRunner
{
    internal static async Task<BatchReport> Run(BatchOptions options, string dotnet, CancellationToken cancellation)
    {
        var report = new BatchReport();
        var seconds = options.Timeout is >= 1 and <= 3600 ? options.Timeout : 30;
        var reserve = TimeSpan.FromSeconds(Math.Min(5, seconds / 5.0));
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(seconds) - reserve);
        using var limit = new CancellationTokenSource();
        using var work = CancellationTokenSource.CreateLinkedTokenSource(cancellation, deadline.Token, limit.Token);
        var token = work.Token;
        DebugSessionHost? host = null;
        Task watching = Task.CompletedTask;
        var stage = "arguments";
        BatchRequest? request = null;
        try
        {
            request = BatchRequest.Create(options);
            report.Limits = new(request.Timeout, request.OutputLimit);
            stage = "preparation"; token.ThrowIfCancellationRequested();
            byte[]? input = null;
            if (request.Stdin is not null)
            {
                await using var file = File.OpenRead(request.Stdin);
                if (file.Length > 1_048_576) throw new ArgumentException("--stdin accepts at most 1 MiB of file bytes.");
                input = new byte[checked((int)file.Length)];
                await file.ReadExactlyAsync(input, token);
                if (file.ReadByte() != -1) throw new IOException("The input file changed during preparation.");
            }
            report.Toolchain = await Toolchain(request.Program, token);
            host = await DebugSessionHost.Start(WorkerLauncher.Current(), request.Cwd, token, input, request.OutputLimit, retainOutputPrefix: true, shutdownGrace: reserve);
            watching = WatchOutput(host, limit, work.Token);
            await host.Prepare(new(dotnet, request.Program, request.Cwd, Arguments: request.Arguments), token);
            await host.Initialize(token);
            var entry = await host.WaitForStop(token);
            RequireStop(entry);
            stage = "configuration";
            foreach (var source in request.Breakpoints.GroupBy(item => item.Source, OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal))
            {
                var points = await host.SetBreakpoints(new(source.Key, source.Select(item => new BreakpointPosition(item.Line, item.Column)).ToArray(), entry.StopId), token);
                if (points.Any(item => item.State == "rejected")) throw new DebugException("breakpoint_rejected", "A requested source breakpoint could not be configured.");
            }
            await host.SetExceptions(new(), token);
            stage = "execution";
            await host.Resume(entry.StopId!.Value, "continue", token);
            var stopped = await host.WaitForStop(token);
            while (options.Tracepoints is { Length: > 0 } && stopped.State == "stopped" && stopped.Reason == "breakpoint")
            {
                var observation = new BatchReport { Outcome = "tracepoint", ExitCode = 0, Termination = "continued" };
                await new SnapshotCapture(host, stopped, observation, token).Capture(request.Expressions);
                report.Tracepoints.Add(observation);
                if (observation.Expressions.Any(item => item.Error is not null || item.Value?.Status != "available"))
                    report.Issue("trace_observation_failed", "inspection", "A tracepoint expression was unavailable; inspect the detached observation.");
                if (report.Tracepoints.Count >= options.MaxHits || System.Text.Json.JsonSerializer.SerializeToUtf8Bytes(report, BatchJson.Default.BatchReport).Length >= ReportLimits.ReportBytes)
                {
                    observation.Termination = "debugger_requested";
                    report.Outcome = "limit"; report.Termination = "debugger_requested"; report.ExitCode = 4;
                    report.Issue("trace_limit", "inspection", "Tracepoint capture reached its hit or report limit before natural exit.");
                    return report;
                }
                await host.Resume(stopped.StopId!.Value, "continue", token);
                stopped = await host.WaitForStop(token);
            }
            report.Breakpoints = stopped.Breakpoints ?? await host.Breakpoints(token);
            report.TargetExitCode = stopped.ExitCode;
            if (stopped.State == "failed") throw new DebugException(stopped.Error?.Code ?? "worker_failure", stopped.Error?.Message ?? "Debugger execution failed.");
            if (stopped.State == "exited")
            {
                report.Outcome = "completed"; report.Termination = "natural";
                var trace = options.Tracepoints is { Length: > 0 };
                var missed = trace && report.Breakpoints.Any(point => point.Hits == 0);
                report.ExitCode = stopped.ExitCode == 0 && (trace ? report.Tracepoints.Count > 0 && !missed && report.Issues.Count == 0 : request.Breakpoints.Length == 0 && request.Expressions.Length == 0) ? 0 : 3;
                if (request.Breakpoints.Length != 0 && (!trace || report.Tracepoints.Count == 0 || missed)) report.Issue("breakpoint_missed", stage, "The program exited before a requested breakpoint/tracepoint.");
                if (!trace) foreach (var expression in request.Expressions) report.Expressions.Add(new(expression, Error: new("unavailable", "Runtime inspection is unavailable after process exit.")));
            }
            else
            {
                RequireStop(stopped);
                report.Outcome = stopped.Reason == "exception" ? "exception" : "breakpoint";
                report.Termination = "debugger_requested";
                stage = "inspection";
                await new SnapshotCapture(host, stopped, report, token).Capture(request.Expressions);
                report.ExitCode = report.Outcome == "exception" || report.Expressions.Any(item => item.Error is not null || item.Value is null || item.Value.Status != "available") ? 3 : 0;
            }
        }
        catch (OperationCanceledException)
        {
            report.Outcome = cancellation.IsCancellationRequested ? "cancelled" : "limit";
            report.ExitCode = cancellation.IsCancellationRequested ? 130 : 4;
            report.Termination = report.Outcome;
            report.Issue(cancellation.IsCancellationRequested ? "cancelled" : limit.IsCancellationRequested ? "output_limit" : "deadline", stage,
                cancellation.IsCancellationRequested ? "Debugging was cancelled." : "The debugging invocation reached its execution limit.");
            report.Capture.Omit("interrupted");
        }
        catch (Exception failure) when (failure is not OutOfMemoryException)
        {
            report.ExitCode = failure is ArgumentException && stage == "arguments" ? 2 : 1;
            report.Termination = host is null ? "not_started" : "failure";
            report.Issue(failure is DebugException debug ? debug.Code : failure is ArgumentException ? "invalid_argument" : "setup_failure", stage, failure.Message);
            report.Capture.Omit("failed");
        }
        finally
        {
            work.Cancel();
            if (host is not null)
            {
                try { using var cleanup = new CancellationTokenSource(reserve); await host.Stop(cleanup.Token); }
                catch (Exception failure)
                {
                    report.Issue("cleanup_failure", "cleanup", failure.Message);
                    if (report.ExitCode is 0 or 3) report.ExitCode = 1;
                }
                report.Output = host.Output();
            }
            await watching;
            if (report.Output.LimitReached && report.ExitCode != 130)
            {
                report.Outcome = "limit"; report.ExitCode = 4; report.Termination = "limit";
                if (!report.Issues.Any(item => item.Code == "output_limit")) report.Issue("output_limit", "execution", "Captured target output reached the byte limit.");
            }
            if (cancellation.IsCancellationRequested) { report.Outcome = "cancelled"; report.ExitCode = 130; report.Termination = "cancelled"; }
        }
        return report;
    }

    private static async Task WatchOutput(DebugSessionHost host, CancellationTokenSource limit, CancellationToken token)
    {
        try { await host.OutputLimitReached.WaitAsync(token); limit.Cancel(); }
        catch (OperationCanceledException) when (token.IsCancellationRequested) { }
    }

    private static void RequireStop(DebugSnapshot state)
    {
        if (state.State != "stopped" || state.StopId is null)
            throw new DebugException(state.Error?.Code ?? "startup_failure", state.Error?.Message ?? "The target did not reach the internal entry stop.");
    }

    private static async Task<ReportToolchain> Toolchain(string program, CancellationToken token)
    {
        var deployment = await DebuggerDeployment.Inspect(token);
        if (deployment.Status != "available") throw new DebugException(deployment.Status, "The own debugger requires its packaged x64 shim on Windows or Linux and .NET 10.");
        using var symbols = new ModuleSymbols(program);
        symbols.ValidateSources();
        return new(deployment.Version, deployment.ShimVersion, deployment.ShimSha256!, RuntimeInformation.FrameworkDescription,
            deployment.Architecture, await Hash(program, token), await Hash(Path.ChangeExtension(program, ".pdb"), token), symbols.Identity);
    }

    private static async Task<string> Hash(string path, CancellationToken token)
    {
        await using var stream = File.OpenRead(path);
        return Convert.ToHexString(await SHA256.HashDataAsync(stream, token));
    }
}
