using System.Text;
using System.Text.RegularExpressions;
using System.Text.Json;
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
        var cli = TestEnvironment.Metadata("FpasCliPath"); var tools = Path.Combine(fixture.Reports, "cli-tools");
        var prepared = await fixture.Run(cli, "profile", "setup", "--tools-directory", tools);
        Assert.True(prepared.ExitCode == 0, prepared.Output + prepared.Error);
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        await using var child = OwnedProcess.Start(TestEnvironment.DotnetHost, [fixture.Workload, "fault", ready, release], fixture.Repository);
        child.Input.Dispose();
        using var stdout = new StreamReader(child.Output, Encoding.UTF8); using var stderr = new StreamReader(child.Error, Encoding.UTF8);
        var output = stdout.ReadToEndAsync(deadline.Token); var error = stderr.ReadToEndAsync(deadline.Token);
        while (!File.Exists(ready)) { Assert.False(child.HasExited); await Task.Delay(20, deadline.Token); }
        Assert.Equal(child.Id.ToString(System.Globalization.CultureInfo.InvariantCulture), File.ReadAllText(ready));
        var collected = await fixture.Run(cli, "diagnose", "dump", "--tools-directory", tools, "--process-id", child.Id.ToString(System.Globalization.CultureInfo.InvariantCulture), "--output", dump);
        File.WriteAllText(Path.Combine(fixture.Reports, "dump-collect.log"), collected.Output + collected.Error);
        Assert.True(collected.ExitCode == 0, collected.Output + collected.Error);
        Assert.True(new FileInfo(dump).Length > 1024);
        File.WriteAllText(release, "continue"); await child.WaitForExit(deadline.Token);
        Assert.True(child.HasExited); Assert.Equal(23, child.ExitCode);
        var diagnostic = await error; await output;
        Assert.Contains("FPAS_DIAGNOSTIC_FAILURE", diagnostic); Assert.Contains("workloads.fpas:line", diagnostic);
        File.WriteAllText(Path.Combine(fixture.Reports, "failure-stack.txt"), diagnostic);
        async Task<string> Inspect(string log, params string[] commands)
        {
            var args = new List<string> { cli, "diagnose", "inspect-dump", dump, "--tools-directory", tools };
            foreach (var command in commands) args.AddRange(["--command", command]);
            var result = await fixture.Run(args.ToArray());
            File.WriteAllText(Path.Combine(fixture.Reports, log), result.Output + result.Error);
            Assert.True(result.ExitCode == 0, result.Output + result.Error);
            using var report = JsonDocument.Parse(result.Output); return report.RootElement.GetProperty("process").GetProperty("output").GetString()!;
        }
        var stacks = await Inspect("dump-stacks.txt", "clrstack -all", "dumpheap -type System.InvalidOperationException -short");
        Assert.Contains("Workloads.HoldFailure", stacks);
        var addresses = Regex.Matches(stacks, @"(?m)^\s*([0-9a-fA-F]{8,16})\s*\r?$").Select(match => match.Groups[1].Value).Distinct().Take(16).ToArray();
        Assert.NotEmpty(addresses);
        var recovered = false;
        foreach (var address in addresses)
        {
            var printed = await Inspect("exception-" + address + ".txt", "pe " + address);
            if (printed.Contains("FPAS_DIAGNOSTIC_FAILURE", StringComparison.Ordinal))
            { Assert.Contains("CauseFailure", printed); recovered = true; break; }
        }
        Assert.True(recovered, "The actual failure object was absent from the dump.");
    }
}
