using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Projects;
using Fpas.Tests.Support;
using System.Text.Json;

namespace Fpas.IntegrationTests.Testing;

public sealed class TestCommandErrors(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Theory]
    [InlineData("--unknown")]
    [InlineData("--timeout", "0")]
    [InlineData("--settings", "missing.runsettings")]
    public async Task Argument_errors_are_one_JSON_result(params string[] options)
    {
        using var directory = new TemporaryDirectory();
        var project = await TestProjectFixture.Create(directory, sdk, "xunit");
        var result = await ProjectCliFixture.Cli(directory, ["test", project, "--report", "json", .. options]);
        Assert.Equal(2, result.ExitCode); Assert.Empty(result.Error);
        using var report = JsonDocument.Parse(result.Output); Assert.Equal("invalid_argument", report.RootElement.GetProperty("error").GetProperty("code").GetString());
    }

    [Fact]
    public async Task No_restore_no_build_non_test_inputs_and_failed_builds_are_explicit()
    {
        using var directory = new TemporaryDirectory();
        var project = await TestProjectFixture.Create(directory, sdk, "xunit");
        var absent = await ProjectCliFixture.Cli(directory, "test", project, "--no-build", "--report", "json");
        Assert.Equal(1, absent.ExitCode);
        using (var report = JsonDocument.Parse(absent.Output)) Assert.Equal("missing_test_output", report.RootElement.GetProperty("error").GetProperty("code").GetString());
        var noRestore = await ProjectCliFixture.Cli(directory, "test", project, "--no-restore", "--report", "json");
        Assert.Equal(1, noRestore.ExitCode);
        var valid = await ProjectCliFixture.Cli(directory, "test", project, "--report", "json"); Assert.True(valid.ExitCode == 0, valid.Output + valid.Error);
        File.WriteAllText(directory.File("Tests/Tests.fpas"), "unit Broken; public procedure F();begin Missing();end procedure;end unit;");
        var invalid = await ProjectCliFixture.Cli(directory, "test", project, "--no-restore", "--report", "json");
        Assert.Equal(1, invalid.ExitCode);
        using (var report = JsonDocument.Parse(invalid.Output)) { Assert.Equal("build_failed", report.RootElement.GetProperty("error").GetProperty("code").GetString()); Assert.Empty(report.RootElement.GetProperty("projects").EnumerateArray()); }
        var stale = await ProjectCliFixture.Cli(directory, "test", project, "--no-build", "--report", "json"); Assert.Equal(0, stale.ExitCode);
        var library = await ProjectCliFixture.Init(directory, sdk, "Library", "library");
        var wrong = await ProjectCliFixture.Cli(directory, "test", library, "--no-build", "--report", "json"); Assert.Equal(2, wrong.ExitCode);
    }
}
