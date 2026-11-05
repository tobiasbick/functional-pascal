using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Projects;
using Fpas.Tests.Support;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Testing;

public sealed class NativeFrameworkTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Theory]
    [InlineData("xunit", "Debug")]
    [InlineData("mstest", "Debug")]
    [InlineData("nunit", "Debug")]
    [InlineData("xunit", "Release")]
    [InlineData("mstest", "Release")]
    [InlineData("nunit", "Release")]
    public async Task A_generated_FPAS_test_project_runs_with_the_native_framework(string framework, string configuration)
    {
        using var directory = new TemporaryDirectory();
        var created = await ProjectCliFixture.Cli(directory, "init", "test", "Tests", "--framework", framework,
            "--sdk-version", sdk.Version, "--sdk-feed", sdk.Feed);
        Assert.True(created.ExitCode == 0, created.Output + created.Error);
        var result = await ProcessRunner.RunAsync(["test", directory.File("Tests/Tests.fpasproj"), "--configuration", configuration,
            "--logger", "trx", "--results-directory", directory.File("results")],
            workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(3));
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        var report = XDocument.Load(Assert.Single(Directory.GetFiles(directory.File("results"), "*.trx")));
        var counters = report.Descendants().Single(element => element.Name.LocalName == "Counters");
        Assert.Equal("1", (string?)counters.Attribute("passed"));
        Assert.Equal("0", (string?)counters.Attribute("failed"));
        Assert.True(File.Exists(directory.File($"Tests/bin/{configuration}/net10.0/Tests.runtimeconfig.json")));
        Assert.True(File.Exists(directory.File($"Tests/bin/{configuration}/net10.0/Tests.deps.json")));
        Assert.DoesNotContain(Directory.GetFiles(directory.File("Tests"), "*.fpas", SearchOption.AllDirectories), path => Path.GetFileName(path) == "main.fpas");
    }
}
