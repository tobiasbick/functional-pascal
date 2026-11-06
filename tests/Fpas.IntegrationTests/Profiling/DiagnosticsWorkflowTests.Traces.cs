using System.Globalization;
using Microsoft.Diagnostics.Tracing;

namespace Fpas.IntegrationTests.Profiling;

public sealed partial class DiagnosticsWorkflowTests
{
    [Fact]
    public async Task Real_EventPipe_traces_resolve_FPAs_stacks_allocations_and_GC()
    {
        var cpu = Path.Combine(fixture.Reports, "cpu.nettrace");
        await fixture.Tool("dotnet-trace", "trace-cpu.log", "collect", "--profile", "dotnet-sampled-thread-time,dotnet-common",
            "--output", cpu, "--", Fpas.Tests.Support.TestEnvironment.DotnetHost, fixture.Workload, "cpu", "5000");
        var top = await fixture.Tool("dotnet-trace", "cpu-top.txt", "report", cpu, "topN", "-n", "30");
        Assert.Contains("Workloads.CpuBatch", top.Output);

        var counters = Path.Combine(fixture.Reports, "runtime.csv");
        await fixture.Tool("dotnet-counters", "counters.log", "collect", "--format", "csv", "--counters", "System.Runtime",
            "--output", counters, "--", Fpas.Tests.Support.TestEnvironment.DotnetHost, fixture.Workload, "alloc", "5000");
        var lines = File.ReadAllLines(counters);
        foreach (var metric in new[] { "dotnet.gc.heap.total_allocated", "dotnet.gc.collections" })
            Assert.Contains(lines, line => line.Contains(metric, StringComparison.Ordinal) &&
                double.TryParse(line[(line.LastIndexOf(',') + 1)..], NumberStyles.Float, CultureInfo.InvariantCulture, out var value) && value > 0);

        var gc = Path.Combine(fixture.Reports, "gc.nettrace");
        await fixture.Tool("dotnet-trace", "trace-gc.log", "collect", "--profile", "gc-verbose", "--output", gc,
            "--", Fpas.Tests.Support.TestEnvironment.DotnetHost, fixture.Workload, "alloc", "3000");
        var collections = 0; var allocations = 0;
        using var source = new EventPipeEventSource(gc);
        source.Clr.GCStart += _ => collections++;
        source.Clr.GCAllocationTick += _ => allocations++;
        source.Process();
        Assert.True(collections > 0, "No GC collection events were captured.");
        Assert.True(allocations > 0, "No allocation samples were captured.");
        File.WriteAllText(Path.Combine(fixture.Reports, "gc-events.txt"), $"collections={collections}\nallocation_samples={allocations}\n");
    }
}
