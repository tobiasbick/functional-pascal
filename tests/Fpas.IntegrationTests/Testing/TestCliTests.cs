// [Documentation](../../coverage.md#native-test-frameworks)
using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Projects;
using Fpas.Tests.Support;
using System.Text.Json;

namespace Fpas.IntegrationTests.Testing;

public sealed class TestCliTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    /// <summary>Checks native framework data and reruns a failing method by its verified output identity.</summary>
    /// <param name="framework">Native .NET test framework used by the fixture.</param>
    [Theory]
    [InlineData("xunit")]
    [InlineData("mstest")]
    [InlineData("nunit")]
    public async Task Native_results_cover_data_async_lifetimes_skips_assertions_and_exceptions(string framework)
    {
        using var directory = new TemporaryDirectory();
        var project = await TestProjectFixture.Create(directory, sdk, framework, detailed: true);
        var events = directory.File("events.log");
        var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "test", project, "--report", "json"],
            workingDirectory: directory.Path, environment: new Dictionary<string, string?> { ["FPAS_TEST_EVENTS"] = events }, timeLimit: TimeSpan.FromMinutes(3));
        Assert.True(result.ExitCode == 1, result.Output + result.Error); Assert.Empty(result.Error);
        using var report = JsonDocument.Parse(result.Output);
        Assert.Equal("test_failed", report.RootElement.GetProperty("status").GetString());
        var projectResult = Assert.Single(report.RootElement.GetProperty("projects").EnumerateArray());
        var native = projectResult.GetProperty("result");
        var counts = native.GetProperty("counts");
        Assert.Equal(2, counts.GetProperty("failed").GetInt32());
        Assert.Equal(1, counts.GetProperty("skipped").GetInt32());
        Assert.True(counts.GetProperty("passed").GetInt32() >= 4);
        Assert.Contains(native.GetProperty("tests").EnumerateArray(), test => test.TryGetProperty("output", out var output) && output.GetString()!.Contains("native output", StringComparison.Ordinal));
        Assert.Contains(native.GetProperty("tests").EnumerateArray(), test => test.TryGetProperty("error", out var error) && error.GetString()!.Contains("user failure", StringComparison.Ordinal));
        var log = File.ReadAllLines(events);
        Assert.Equal(6, log.Count(line => line == "setup")); Assert.Equal(6, log.Count(line => line == "cleanup"));
        if (framework == "xunit") Assert.Equal("disposed", File.ReadAllText(events + ".fixture"));
        var reruns = projectResult.GetProperty("reruns").EnumerateArray().ToArray();
        Assert.Equal(2, reruns.Length);
        var rerun = reruns[0];
        Assert.Equal("method", rerun.GetProperty("scope").GetString());
        var failedIndex = rerun.GetProperty("caseIndex").GetInt32();
        var failedMethod = native.GetProperty("tests")[failedIndex].GetProperty("fullyQualifiedName").GetString();
        var args = rerun.GetProperty("arguments").EnumerateArray().Select(item => item.GetString()!).ToArray();
        var identityIndex = Array.IndexOf(args, "--expected-output") + 1;
        Assert.True(identityIndex > 0);
        args[identityIndex] = args[identityIndex].ToLowerInvariant();
        var repeated = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), .. args],
            workingDirectory: directory.Path, environment: new Dictionary<string, string?> { ["FPAS_TEST_EVENTS"] = events }, timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(repeated.ExitCode == 1, repeated.Output + repeated.Error);
        using var repeatReport = JsonDocument.Parse(repeated.Output);
        var repeatedProject = Assert.Single(repeatReport.RootElement.GetProperty("projects").EnumerateArray());
        var repeatedCases = repeatedProject.GetProperty("result").GetProperty("tests").EnumerateArray().ToArray();
        Assert.NotEmpty(repeatedCases);
        Assert.All(repeatedCases, test => Assert.Equal(failedMethod, test.GetProperty("fullyQualifiedName").GetString()));
        if (framework == "xunit")
        {
            args[identityIndex] = "invalid";
            var invalid = await ProjectCliFixture.Cli(directory, args);
            Assert.Equal(2, invalid.ExitCode);
            using var invalidReport = JsonDocument.Parse(invalid.Output);
            Assert.Equal("invalid_argument", invalidReport.RootElement.GetProperty("error").GetProperty("code").GetString());
        }
    }

    [Theory]
    [InlineData("xunit")]
    [InlineData("mstest")]
    [InlineData("nunit")]
    public async Task Filtered_execution_and_native_discovery_share_project_selection(string framework)
    {
        using var directory = new TemporaryDirectory();
        var project = await TestProjectFixture.Create(directory, sdk, framework);
        var first = await ProjectCliFixture.Cli(directory, "test", project, "--report", "json");
        Assert.True(first.ExitCode == 0, first.Output + first.Error);
        var checkedProject = await ProjectCliFixture.Cli(directory, "check", project, "--no-restore", "--report", "json");
        Assert.True(checkedProject.ExitCode == 0, checkedProject.Output + checkedProject.Error);
        var listed = await ProjectCliFixture.Cli(directory, "test", project, "--no-build", "--list", "--filter", "FullyQualifiedName~Addition", "--report", "json");
        Assert.True(listed.ExitCode == 0, listed.Output + listed.Error); Assert.Empty(listed.Error);
        using (var report = JsonDocument.Parse(listed.Output))
        {
            var entry = Assert.Single(report.RootElement.GetProperty("projects").EnumerateArray());
            Assert.Contains("Addition", Assert.Single(entry.GetProperty("discovery").EnumerateArray()).GetProperty("fullyQualifiedName").GetString());
            Assert.False(report.RootElement.TryGetProperty("build", out _));
        }
        var empty = await ProjectCliFixture.Cli(directory, "test", project, "--no-build", "--filter", "FullyQualifiedName~DoesNotExist", "--report", "json");
        Assert.True(empty.ExitCode == 0, empty.Output + empty.Error);
        using (var report = JsonDocument.Parse(empty.Output))
            Assert.Equal(0, Assert.Single(report.RootElement.GetProperty("projects").EnumerateArray()).GetProperty("result").GetProperty("counts").GetProperty("total").GetInt32());
    }

    [Fact]
    public async Task Native_settings_plain_listing_and_unique_result_directories_work_without_a_JSON_preamble()
    {
        using var directory = new TemporaryDirectory();
        var project = await TestProjectFixture.Create(directory, sdk, "xunit");
        var settings = directory.File("native.runsettings");
        File.WriteAllText(settings, "<RunSettings><RunConfiguration><MaxCpuCount>1</MaxCpuCount><TestCaseFilter>FullyQualifiedName~Addition</TestCaseFilter></RunConfiguration></RunSettings>");
        var first = await ProjectCliFixture.Cli(directory, "test", project, "--settings", settings, "--results-directory", directory.File("results"), "--report", "json");
        Assert.True(first.ExitCode == 0, first.Output + first.Error);
        using var report = JsonDocument.Parse(first.Output);
        var folder = report.RootElement.GetProperty("resultsDirectory").GetString()!;
        var listed = await ProjectCliFixture.Cli(directory, "test", project, "--no-build", "--settings", settings, "--list");
        Assert.Equal(0, listed.ExitCode); Assert.Equal("Tests.Addition" + Environment.NewLine, listed.Output); Assert.Empty(listed.Error);
        var repeated = await ProjectCliFixture.Cli(directory, "test", project, "--no-build", "--results-directory", directory.File("results"), "--report", "json");
        Assert.Equal(0, repeated.ExitCode);
        using var again = JsonDocument.Parse(repeated.Output);
        Assert.NotEqual(folder, again.RootElement.GetProperty("resultsDirectory").GetString()); Assert.True(Directory.Exists(folder));
        var malformed = await ProjectCliFixture.Cli(directory, "test", project, "--no-build", "--filter", "FullyQualifiedName~Addition&", "--report", "json");
        // This adapter reports malformed filters as native warnings and a successful empty run.
        Assert.Equal(0, malformed.ExitCode); using var warning = JsonDocument.Parse(malformed.Output);
        var native = Assert.Single(warning.RootElement.GetProperty("projects").EnumerateArray()).GetProperty("result");
        Assert.Equal(0, native.GetProperty("counts").GetProperty("total").GetInt32());
        Assert.Contains(native.GetProperty("messages").EnumerateArray(), message => message.GetProperty("outcome").GetString() == "Warning");
    }
}
