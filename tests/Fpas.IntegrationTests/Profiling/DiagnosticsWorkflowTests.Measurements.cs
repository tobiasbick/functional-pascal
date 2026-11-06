using System.Globalization;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Profiling;

public sealed partial class DiagnosticsWorkflowTests(DiagnosticsFixture fixture) : IClassFixture<DiagnosticsFixture>
{
    [Fact]
    public async Task Workloads_verify_results_and_write_independent_unprofiled_samples()
    {
        foreach (var kind in new[] { "cpu", "alloc" })
        {
            var path = Path.Combine(fixture.Reports, "smoke-" + kind + ".csv");
            var result = await fixture.Run(fixture.Workload, "bench", kind, "8", "1", "3", path, kind + "-" + fixture.Configuration, "local");
            Assert.True(result.ExitCode == 0, result.Output + result.Error);
            Assert.Contains("verified:" + kind, result.Output);
            var expected = kind == "cpu" ? 32767L * 32768 * 65535 / 6 * 8 : (251L * 250 + 9 * 10 / 2) * 8;
            Assert.Contains("checksum=" + expected.ToString(CultureInfo.InvariantCulture), result.Output);
            var lines = File.ReadAllLines(path);
            Assert.Equal(4, lines.Length); Assert.Contains("milliseconds", lines[0]);
            Assert.All(lines.Skip(1), line => Assert.True(double.Parse(line.Split(',')[4], CultureInfo.InvariantCulture) >= 0));
            var previous = File.ReadAllBytes(path);
            var conflict = await fixture.Run(fixture.Workload, "bench", kind, "8", "1", "3", path, kind, "local");
            Assert.NotEqual(0, conflict.ExitCode); Assert.Equal(previous, File.ReadAllBytes(path));
        }
        var startup = Path.Combine(fixture.Reports, "smoke-startup.csv");
        var measured = await fixture.Run(fixture.Measure, "--output", startup, "--label", "startup-" + fixture.Configuration,
            "--boundary", "startup", "--warmup", "1", "--samples", "3", "--contains", "verified:noop", "--", TestEnvironment.DotnetHost, fixture.Workload, "noop");
        Assert.True(measured.ExitCode == 0, measured.Output + measured.Error);
        Assert.Equal(4, File.ReadAllLines(startup).Length);
        var rejected = await fixture.Run(fixture.Measure, "--output", Path.Combine(fixture.Reports, "rejected.csv"), "--label", "negative-control",
            "--boundary", "startup", "--warmup", "0", "--samples", "3", "--contains", "never-produced", "--", TestEnvironment.DotnetHost, fixture.Workload, "noop");
        Assert.Equal(1, rejected.ExitCode); Assert.False(File.Exists(Path.Combine(fixture.Reports, "rejected.csv")));
    }

    [Fact]
    public async Task Measurement_deadline_terminates_the_started_target_before_reporting_failure()
    {
        var ready = Path.Combine(fixture.Reports, "timeout.ready");
        var output = Path.Combine(fixture.Reports, "timeout.csv");
        var result = await fixture.Run(fixture.Measure, "--output", output, "--label", "timeout-control",
            "--boundary", "startup", "--warmup", "0", "--samples", "3", "--timeout", "10", "--",
            TestEnvironment.DotnetHost, fixture.Workload, "fault", ready, Path.Combine(fixture.Reports, "never.release"));
        Assert.Equal(1, result.ExitCode); Assert.Contains("timed out", result.Error);
        Assert.False(File.Exists(output)); Assert.True(File.Exists(ready));
        var pid = int.Parse(File.ReadAllText(ready), CultureInfo.InvariantCulture);
        try { using var process = System.Diagnostics.Process.GetProcessById(pid); Assert.True(process.HasExited); } catch (ArgumentException) { }
    }
}
