using System.Text.Json;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Profiling;

public sealed partial class DiagnosticsWorkflowTests
{
    [Fact]
    public async Task Profile_CLI_collects_real_artifacts_and_summarizes_thread_stacks_and_GC()
    {
        var cli = TestEnvironment.Metadata("FpasCliPath"); var tools = Path.Combine(fixture.Reports, "cli-tools");
        var prepared = await fixture.Run(cli, "profile", "setup", "--tools-directory", tools);
        Assert.True(prepared.ExitCode == 0, prepared.Output + prepared.Error);
        foreach (var mode in new[] { "threads", "gc", "counters" })
        {
            var artifact = Path.Combine(fixture.Reports, "cli-" + mode + (mode == "counters" ? ".csv" : ".nettrace"));
            var target = mode == "threads" ? fixture.Workload : Fpas.Workspaces.Processes.BuildProcess.DotnetHost();
            var args = mode == "threads" ? new[] { "cpu", "3000" } : new[] { fixture.Workload, "alloc", "3000" };
            var collected = await fixture.Run([cli, "profile", "collect", target, "--mode", mode,
                "--output", artifact, "--duration", "5", "--tools-directory", tools, "--", .. args]);
            Assert.True(collected.ExitCode == 0, collected.Output + collected.Error); Assert.True(new FileInfo(artifact).Length > 0);
            var previous = File.ReadAllBytes(artifact);
            var conflict = await fixture.Run(cli, "profile", "collect", Fpas.Workspaces.Processes.BuildProcess.DotnetHost(), "--output", artifact, "--tools-directory", tools);
            Assert.Equal(2, conflict.ExitCode); Assert.Equal(previous, File.ReadAllBytes(artifact));
            if (mode == "counters") { Assert.Contains("dotnet.gc.heap.total_allocated", File.ReadAllText(artifact), StringComparison.Ordinal); continue; }
            var summarized = await fixture.Run(cli, "profile", "summary", artifact, "--limit", "100");
            Assert.True(summarized.ExitCode == 0, summarized.Output + summarized.Error);
            using var report = JsonDocument.Parse(summarized.Output); var trace = report.RootElement.GetProperty("trace");
            if (mode == "threads")
            {
                Assert.True(trace.GetProperty("samples").GetInt64() > 0);
                Assert.Contains(trace.GetProperty("threads").EnumerateArray(), thread => thread.GetProperty("id").GetInt32() > 0 &&
                    thread.GetProperty("frames").EnumerateArray().Any(frame => frame.GetProperty("method").GetString()!.Contains("Workloads.CpuBatch", StringComparison.Ordinal)));
            }
            else { Assert.True(trace.GetProperty("collections").GetInt64() > 0); Assert.True(trace.GetProperty("allocationTicks").GetInt64() > 0); }
        }
        var invalid = Path.Combine(fixture.Reports, "invalid.nettrace"); File.WriteAllText(invalid, "not an EventPipe recording");
        var rejected = await fixture.Run(cli, "profile", "summary", invalid); Assert.Equal(1, rejected.ExitCode);
        using var failure = JsonDocument.Parse(rejected.Output); Assert.False(failure.RootElement.GetProperty("success").GetBoolean());
        var ready = Path.Combine(fixture.Reports, "collector-timeout.ready");
        var timedOut = await fixture.Run(cli, "profile", "collect", Fpas.Workspaces.Processes.BuildProcess.DotnetHost(), "--output", Path.Combine(fixture.Reports, "timeout.nettrace"),
            "--tools-directory", tools, "--timeout", "5", "--duration", "30", "--", fixture.Workload, "fault", ready, Path.Combine(fixture.Reports, "collector-never.release"));
        Assert.Equal(1, timedOut.ExitCode); using var timeout = JsonDocument.Parse(timedOut.Output);
        Assert.Equal("timeout", timeout.RootElement.GetProperty("status").GetString()); Assert.True(File.Exists(ready));
        var pid = int.Parse(File.ReadAllText(ready), System.Globalization.CultureInfo.InvariantCulture);
        try { using var process = System.Diagnostics.Process.GetProcessById(pid); Assert.True(process.HasExited); } catch (ArgumentException) { }
    }
}
