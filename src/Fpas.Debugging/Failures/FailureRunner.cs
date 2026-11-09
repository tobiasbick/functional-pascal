using Fpas.Debugging.Batch;
using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;
using Fpas.Processes;

namespace Fpas.Debugging.Failures;

internal static class FailureRunner
{
    internal static async Task<FailureRunResult> Run(FailureRunOptions options, string dotnet, CancellationToken cancellation)
    {
        if (options.Timeout is < 0 or > 3600) throw new ArgumentException("Use --timeout 0..3600.");
        if (!options.CaptureOutput && !options.InheritInput) throw new ArgumentException("Closed input requires captured output.");
        using var deadline = CancellationTokenSource.CreateLinkedTokenSource(cancellation);
        if (options.Timeout > 0) deadline.CancelAfter(TimeSpan.FromSeconds(options.Timeout));
        var token = deadline.Token;
        var request = BatchRequest.Create(new(options.Program, options.Cwd, Arguments: options.Arguments));
        using var file = new FailureReportFile(options.ReportPath);
        var report = new BatchReport { Limits = new(options.Timeout, OutputCapture.MaximumBytes) };
        DebugSessionHost? host = null;
        int? exitCode = null;
        try
        {
            report.Toolchain = await BatchRunner.Toolchain(request.Program, token);
            var stdio = !options.CaptureOutput ? ProcessStdio.Inherit : options.InheritInput ? ProcessStdio.CaptureOutput : ProcessStdio.Capture;
            host = await DebugSessionHost.Start(WorkerLauncher.Current(), request.Cwd, token, retainOutputPrefix: true, stdio: stdio);
            await host.Prepare(new(dotnet, request.Program, request.Cwd, Arguments: request.Arguments), token);
            await host.Initialize(token);
            var stopped = await host.WaitForStop(token);
            RequireStop(stopped, "entry");
            await host.SetExceptions(new(), token);
            await host.Resume(stopped.StopId!.Value, "continue", token);
            while (true)
            {
                stopped = await host.WaitForStop(token);
                if (stopped.State == "exited")
                {
                    exitCode = stopped.ExitCode ?? throw new IOException("The debugger returned no target exit code.");
                    break;
                }
                RequireStop(stopped, "exception");
                if (!file.Saved)
                {
                    report.Outcome = "exception"; report.ExitCode = 3; report.Termination = "running";
                    report.Stop = new("exception", null, null);
                    report.Capture.Omit("task_inventory");
                    if (!options.CaptureOutput) report.Capture.Omit("output_inherited");
                    using var capture = CancellationTokenSource.CreateLinkedTokenSource(token);
                    capture.CancelAfter(TimeSpan.FromSeconds(10));
                    try { await new SnapshotCapture(host, stopped, report, capture.Token).Capture([]); }
                    catch (Exception failure) when (failure is not OutOfMemoryException)
                    {
                        report.Capture.Omit("interrupted");
                        report.Issue("capture_incomplete", "inspection", failure.Message);
                    }
                    report.Output = host.Output();
                    // Persist while the original stop still exists, before target execution resumes.
                    await file.Save(report);
                }
                await host.Resume(stopped.StopId!.Value, "continue", token);
            }
        }
        catch (DebugException failure) { throw new IOException($"{failure.Code}: {failure.Message}", failure); }
        finally
        {
            try { if (host is not null) await host.DisposeAsync(); }
            finally
            {
                if (file.Saved)
                {
                    report.TargetExitCode = exitCode;
                    report.Termination = exitCode is not null ? "natural" : "interrupted";
                    if (exitCode is null) report.Capture.Omit("execution_interrupted");
                    if (host is not null) report.Output = host.Output();
                    await file.Save(report);
                }
            }
        }
        var output = host!.Output();
        string Text(string stream) => string.Concat(output.Items.Where(item => item.Stream == stream).Select(item => item.Text));
        return new(exitCode!.Value, Text("stdout"), Text("stderr"), output.Truncated, file.Saved ? options.ReportPath : null);
    }

    private static void RequireStop(DebugSnapshot state, string reason)
    {
        if (state.State != "stopped" || state.StopId is null || state.Reason != reason)
            throw new IOException(state.Error?.Message ?? $"The target did not reach its expected {reason} stop.");
    }
}
