using System.Globalization;
using Fpas.IntegrationTests.Debugging;
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
    public async Task Measurement_rejects_a_relative_executable_before_starting_it()
    {
        var host = TestEnvironment.DotnetHost;
        if (!Path.IsPathFullyQualified(host))
        {
            var executable = OperatingSystem.IsWindows() && !Path.HasExtension(host) ? host + ".exe" : host;
            host = (Environment.GetEnvironmentVariable("PATH") ?? "").Split(Path.PathSeparator)
                .Select(directory => Path.GetFullPath(Path.Combine(directory, executable)))
                .First(File.Exists);
        }
        var relativeHost = Path.Combine(".", Path.GetFileName(host));
        var ready = Path.Combine(fixture.Reports, "relative-executable.ready");
        var output = Path.Combine(fixture.Reports, "relative-executable.csv");
        var result = await fixture.RunFrom(Path.GetDirectoryName(host)!, fixture.Measure,
            "--output", output, "--label", "relative-executable", "--boundary", "startup",
            "--warmup", "0", "--samples", "3", "--timeout", "1", "--", relativeHost,
            fixture.Workload, "fault", ready, Path.Combine(fixture.Reports, "never.release"));

        Assert.Equal(2, result.ExitCode);
        Assert.Contains("Executable path must be absolute", result.Error, StringComparison.Ordinal);
        Assert.False(File.Exists(ready));
        Assert.False(File.Exists(output));
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

    /// <summary>Checks that multibyte output triggers the byte limit and joins the still-running child.</summary>
    /// <returns>The completed process and publication assertions.</returns>
    [Fact]
    public async Task Measurement_output_limit_terminates_the_child_before_reporting_failure()
    {
        using var child = new Debuggee("measurement-output-limit");
        await child.Compile("""
            program OutputFlood;
            uses System as Sys, System.IO as IO, System.Text as Text, System.Threading as Threading;
            begin
              const Bytes: array of byte := Text.Encoding.UTF8.GetBytes(new Sys.String(#233, 40000));
              IO.File.WriteAllText(args[0], Sys.Environment.ProcessId.ToString());
              const Output: IO.Stream := Sys.Console.OpenStandardOutput();
              Output.Write(Bytes, 0, Bytes.Length);
              Threading.Thread.Sleep(Threading.Timeout.Infinite);
            end;
            end program;
            """);
        var ready = Path.Combine(fixture.Reports, "output-limit.ready");
        var output = Path.Combine(fixture.Reports, "output-limit.csv");
        var result = await fixture.Run(fixture.Measure, "--output", output, "--label", "output-limit-control",
            "--boundary", "startup", "--warmup", "0", "--samples", "3", "--timeout", "10", "--",
            TestEnvironment.DotnetHost, child.Assembly, ready);

        Assert.Equal(1, result.ExitCode);
        Assert.Contains("Child output exceeded 64 KiB", result.Error, StringComparison.Ordinal);
        Assert.False(File.Exists(output));
        var pid = int.Parse(File.ReadAllText(ready), CultureInfo.InvariantCulture);
        try { using var process = System.Diagnostics.Process.GetProcessById(pid); Assert.True(process.HasExited); } catch (ArgumentException) { }
    }

    /// <summary>Checks that invalid integer inputs use the documented argument-error exit path.</summary>
    /// <param name="value">An invalid or overflowing numeric argument.</param>
    /// <returns>The completed exit-code assertions.</returns>
    [Theory]
    [InlineData("invalid")]
    [InlineData("2147483648")]
    public async Task Invalid_numeric_arguments_report_usage_errors(string value)
    {
        var workload = await fixture.Run(fixture.Workload, "cpu", value);
        Assert.Equal(2, workload.ExitCode);
        Assert.Contains("Numeric argument", workload.Error, StringComparison.Ordinal);

        var measure = await fixture.Run(fixture.Measure, "--timeout", value, "--", TestEnvironment.DotnetHost);
        Assert.Equal(2, measure.ExitCode);
        Assert.Contains("Numeric option", measure.Error, StringComparison.Ordinal);
    }
}
