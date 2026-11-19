// [Documentation](../../../docs/architecture/debugger-core.md#passive-exception-facts)
using Fpas.Debugging.Batch;
using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;
using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Debugging;

/// <summary>Verifies that a snapshot budget cannot terminate the original failing target.</summary>
public sealed class FailureCaptureTests
{
    /// <summary>Exhaust a deterministic capture budget, inspect the held failure and observe its natural crash exit.</summary>
    [Fact]
    public async Task Snapshot_budget_exhaustion_preserves_the_exception_stop_and_natural_target_exit()
    {
        using var program = new Debuggee("failure-capture-budget");
        await program.Compile("program Failure; uses System.InvalidOperationException as Failure; begin raise new Failure('original'); end; end program;");
        var baseline = await ProcessRunner.RunAsync([program.Assembly], workingDirectory: program.DirectoryPath);
        Assert.NotEqual(0, baseline.ExitCode);
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        var launcher = WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost());
        await using var host = await DebugSessionHost.Start(launcher, program.DirectoryPath, deadline.Token);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath), deadline.Token);
        await host.Initialize(deadline.Token);
        var entry = await host.WaitForStop(deadline.Token);
        await host.Resume(entry.StopId!.Value, "continue", deadline.Token);
        var stopped = await host.WaitForStop(deadline.Token);
        Assert.Equal("exception", stopped.Reason);

        var report = new BatchReport();
        var limit = await Assert.ThrowsAsync<DebugException>(() =>
            new SnapshotCapture(host, stopped, report, deadline.Token, TimeSpan.Zero).Capture([]));
        Assert.Equal("capture_limit", limit.Code);
        Assert.Equal(stopped.StopId, host.Snapshot.StopId);
        Assert.Equal(stopped.Revision, host.Snapshot.Revision);
        Assert.Equal("original", (await host.ExceptionInfo(stopped.StopId!.Value, deadline.Token)).Message.Text);
        await host.Resume(stopped.StopId.Value, OperatingSystem.IsWindows() ? "detach" : "continue", deadline.Token);
        var ended = await host.WaitForStop(deadline.Token);
        Assert.Equal("exited", ended.State);
        Assert.Equal(baseline.ExitCode, ended.ExitCode);
        await host.Stop(deadline.Token);
    }
}
