using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Projects;

internal static class ProjectCliFixture
{
    internal static Task<ProcessResult> Cli(TemporaryDirectory directory, params string[] args)
        => ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), .. args], workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(3));

    internal static async Task<string> Init(TemporaryDirectory directory, SdkPackageFixture sdk, string name = "App", string kind = "project")
    {
        var result = await Cli(directory, "init", kind, name, "--name", name, "--sdk-version", sdk.Version, "--sdk-feed", sdk.Feed, "--report", "json");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        return directory.File(name + "/" + name + ".fpasproj");
    }
}
