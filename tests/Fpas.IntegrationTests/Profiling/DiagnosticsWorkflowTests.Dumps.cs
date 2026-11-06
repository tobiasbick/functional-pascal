using System.Text;
using System.Text.RegularExpressions;
using Fpas.Processes;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Profiling;

public sealed partial class DiagnosticsWorkflowTests
{
    [Fact]
    public async Task Controlled_failure_dump_recovers_managed_frames_and_the_real_exception()
    {
        var ready = Path.Combine(fixture.Reports, "fault.ready"); var release = Path.Combine(fixture.Reports, "fault.release");
        var dump = Path.Combine(fixture.Reports, "failure.dmp");
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        await using var child = OwnedProcess.Start(TestEnvironment.DotnetHost, [fixture.Workload, "fault", ready, release], fixture.Repository);
        child.Input.Dispose();
        using var stdout = new StreamReader(child.Output, Encoding.UTF8); using var stderr = new StreamReader(child.Error, Encoding.UTF8);
        var output = stdout.ReadToEndAsync(deadline.Token); var error = stderr.ReadToEndAsync(deadline.Token);
        while (!File.Exists(ready)) { Assert.False(child.HasExited); await Task.Delay(20, deadline.Token); }
        Assert.Equal(child.Id.ToString(System.Globalization.CultureInfo.InvariantCulture), File.ReadAllText(ready));
        await fixture.Tool("dotnet-dump", "dump-collect.log", "collect", "--process-id", child.Id.ToString(System.Globalization.CultureInfo.InvariantCulture),
            "--type", "Heap", "--output", dump);
        Assert.True(new FileInfo(dump).Length > 1024);
        File.WriteAllText(release, "continue"); await child.WaitForExit(deadline.Token);
        Assert.True(child.HasExited); Assert.Equal(23, child.ExitCode);
        var diagnostic = await error; await output;
        Assert.Contains("FPAS_DIAGNOSTIC_FAILURE", diagnostic); Assert.Contains("workloads.fpas:line", diagnostic);
        File.WriteAllText(Path.Combine(fixture.Reports, "failure-stack.txt"), diagnostic);
        var stacks = await fixture.Tool("dotnet-dump", "dump-stacks.txt", "analyze", dump, "-c", "clrstack -all", "-c", "dumpheap -type System.InvalidOperationException -short", "-c", "exit");
        Assert.Contains("Workloads.HoldFailure", stacks.Output);
        var addresses = Regex.Matches(stacks.Output, @"(?m)^\s*([0-9a-fA-F]{8,16})\s*\r?$").Select(match => match.Groups[1].Value).Distinct().Take(16).ToArray();
        Assert.NotEmpty(addresses);
        var recovered = false;
        foreach (var address in addresses)
        {
            var printed = await fixture.Tool("dotnet-dump", "exception-" + address + ".txt", "analyze", dump, "-c", "pe " + address, "-c", "exit");
            if (printed.Output.Contains("FPAS_DIAGNOSTIC_FAILURE", StringComparison.Ordinal))
            { Assert.Contains("CauseFailure", printed.Output); recovered = true; break; }
        }
        Assert.True(recovered, "The actual failure object was absent from the dump.");
    }
}
