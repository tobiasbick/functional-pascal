using System.Text.Json;
using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Projects;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Testing;

public sealed class TestingPlatformTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Malformed_native_MTP_discovery_returns_a_structured_failure()
    {
        using var directory = new TemporaryDirectory();
        var created = await ProjectCliFixture.Cli(directory, "init", "test", "Malformed", "--framework", "mstest", "--runner", "mtp", "--sdk-version", sdk.Version, "--sdk-feed", sdk.Feed);
        Assert.True(created.ExitCode == 0, created.Output + created.Error);
        File.WriteAllText(directory.File("Malformed/main.fpas"), """
            program Malformed.Runner;
            uses System as Sys;
            begin
              Sys.Console.WriteLine(Sys.IO.File.ReadAllText('discovery-response.json'));
            end;
            end program;
            """);
        var project = directory.File("Malformed/Malformed.fpasproj"); var config = TestEnvironment.Metadata("FpasBuildConfiguration");
        var build = await ProjectCliFixture.Cli(directory, "build", project, "--configuration", config, "--report", "json");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        foreach (var response in new[] { "{", "{}", "{\"schemaVersion\":\"1\",\"tests\":[]}", "{\"schemaVersion\":1,\"tests\":[{\"uid\":42}]}" })
        {
            File.WriteAllText(directory.File("Malformed/discovery-response.json"), response);
            var result = await ProjectCliFixture.Cli(directory, "test", project, "--configuration", config, "--no-build", "--list", "--report", "json");
            Assert.Equal(1, result.ExitCode);
            using var json = JsonDocument.Parse(result.Output);
            Assert.False(json.RootElement.GetProperty("success").GetBoolean());
            Assert.Equal("discovery_failed", json.RootElement.GetProperty("error").GetProperty("code").GetString());
        }
    }

    [Fact]
    public async Task A_timed_out_MTP_application_is_reaped_before_the_CLI_returns()
    {
        using var directory = new TemporaryDirectory();
        var created = await ProjectCliFixture.Cli(directory, "init", "test", "Waiting", "--framework", "mstest", "--runner", "mtp", "--sdk-version", sdk.Version, "--sdk-feed", sdk.Feed);
        Assert.True(created.ExitCode == 0, created.Output + created.Error);
        var source = directory.File("Waiting/Waiting.fpas");
        var identity = directory.File("runner.pid");
        File.WriteAllText(source, File.ReadAllText(source).Replace("T.Assert.AreEqual<int>(4, 2 + 2);", "Sys.IO.File.WriteAllText('" + identity.Replace("'", "''", StringComparison.Ordinal) + "', Sys.Environment.ProcessId.ToString()); Sys.Threading.Thread.Sleep(60000);", StringComparison.Ordinal)
            .Replace("uses Microsoft", "uses System as Sys, Microsoft", StringComparison.Ordinal));
        var project = directory.File("Waiting/Waiting.fpasproj"); var config = TestEnvironment.Metadata("FpasBuildConfiguration");
        var build = await ProjectCliFixture.Cli(directory, "build", project, "--configuration", config, "--report", "json");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var result = await ProjectCliFixture.Cli(directory, "test", project, "--configuration", config, "--no-build", "--timeout", "10", "--report", "json");
        Assert.Equal(1, result.ExitCode); using var json = JsonDocument.Parse(result.Output); Assert.Equal("timeout", json.RootElement.GetProperty("status").GetString());
        Assert.True(File.Exists(identity), result.Output);
        var pid = int.Parse(File.ReadAllText(identity), System.Globalization.CultureInfo.InvariantCulture);
        try { using var process = System.Diagnostics.Process.GetProcessById(pid); Assert.True(process.HasExited); } catch (ArgumentException) { }
    }

    [Fact]
    public async Task FPAS_self_hosted_MTP_uses_native_discovery_results_filters_and_extensions()
    {
        using var directory = new TemporaryDirectory();
        var created = await ProjectCliFixture.Cli(directory, "init", "test", "Native", "--framework", "mstest", "--runner", "mtp", "--sdk-version", sdk.Version, "--sdk-feed", sdk.Feed);
        Assert.True(created.ExitCode == 0, created.Output + created.Error);
        var project = directory.File("Native/Native.fpasproj"); var config = TestEnvironment.Metadata("FpasBuildConfiguration");
        var run = await ProjectCliFixture.Cli(directory, "test", project, "--configuration", config, "--report", "json", "--", "--minimum-expected-tests", "1");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        using (var json = JsonDocument.Parse(run.Output)) Assert.Equal(1, json.RootElement.GetProperty("projects")[0].GetProperty("result").GetProperty("counts").GetProperty("passed").GetInt32());
        var list = await ProjectCliFixture.Cli(directory, "test", project, "--configuration", config, "--no-build", "--list", "--report", "json");
        Assert.True(list.ExitCode == 0, list.Output + list.Error);
        using (var json = JsonDocument.Parse(list.Output))
        {
            var discovered = Assert.Single(json.RootElement.GetProperty("projects")[0].GetProperty("discovery").EnumerateArray());
            Assert.NotEmpty(discovered.GetProperty("uid").GetString()!); Assert.True(discovered.GetProperty("fullyQualifiedName").GetString()!.Contains("Addition", StringComparison.Ordinal), list.Output);
        }
        var vstest = await TestProjectFixture.Create(directory, sdk, "nunit", name: "Vstest");
        var solution = await ProjectCliFixture.Cli(directory, "init", "solution", "Mixed", "--project", project, "--project", vstest);
        Assert.True(solution.ExitCode == 0, solution.Output + solution.Error);
        var mixed = await ProjectCliFixture.Cli(directory, "test", directory.File("Mixed/Mixed.slnx"), "--configuration", config, "--report", "json");
        Assert.True(mixed.ExitCode == 0, mixed.Output + mixed.Error);
        using (var json = JsonDocument.Parse(mixed.Output))
        {
            var projects = json.RootElement.GetProperty("projects").EnumerateArray().ToArray(); Assert.Equal(2, projects.Length);
            Assert.Single(projects, item => item.GetProperty("project").GetProperty("testingPlatform").GetBoolean());
            Assert.All(projects, item => Assert.Equal(1, item.GetProperty("result").GetProperty("counts").GetProperty("passed").GetInt32()));
        }
        var missed = await ProjectCliFixture.Cli(directory, "test", project, "--configuration", config, "--no-build", "--filter", "FullyQualifiedName~Absent", "--report", "json", "--", "--minimum-expected-tests", "1");
        Assert.Equal(1, missed.ExitCode);
        var protectedOption = await ProjectCliFixture.Cli(directory, "test", project, "--configuration", config, "--no-build", "--report", "json", "--", "--results-directory", directory.File("other"));
        Assert.Equal(2, protectedOption.ExitCode);
        File.WriteAllText(directory.File("Native/Native.fpas"), File.ReadAllText(directory.File("Native/Native.fpas")).Replace("4, 2 + 2", "5, 2 + 2", StringComparison.Ordinal));
        var failing = await ProjectCliFixture.Cli(directory, "test", project, "--configuration", config, "--report", "json");
        Assert.Equal(1, failing.ExitCode);
        using var failure = JsonDocument.Parse(failing.Output); Assert.Equal(1, failure.RootElement.GetProperty("projects")[0].GetProperty("result").GetProperty("counts").GetProperty("failed").GetInt32());
    }
}
