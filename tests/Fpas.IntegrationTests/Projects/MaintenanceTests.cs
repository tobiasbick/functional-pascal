using System.Text.Json;
using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Projects;

public sealed class MaintenanceTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Restore_and_clean_use_the_real_SDK_and_preserve_unowned_files()
    {
        using var directory = new TemporaryDirectory();
        var project = await ProjectCliFixture.Init(directory, sdk);
        var configuration = TestEnvironment.Metadata("FpasBuildConfiguration");
        var restored = await ProjectCliFixture.Cli(directory, "restore", project, "--configuration", configuration, "--report", "json");
        Assert.True(restored.ExitCode == 0, restored.Output + restored.Error);
        using (var json = JsonDocument.Parse(restored.Output)) Assert.Equal("restore", json.RootElement.GetProperty("command").GetString());
        var built = await ProjectCliFixture.Cli(directory, "build", project, "--configuration", configuration, "--no-restore", "--report", "json");
        Assert.True(built.ExitCode == 0, built.Output + built.Error);
        var assembly = directory.File($"App/bin/{configuration}/net10.0/App.dll"); Assert.True(File.Exists(assembly));
        var unrelated = directory.File($"App/bin/{configuration}/net10.0/keep.txt"); File.WriteAllText(unrelated, "unowned");
        for (var count = 0; count < 2; count++)
        {
            var clean = await ProjectCliFixture.Cli(directory, "clean", project, "--configuration", configuration, "--report", "json");
            Assert.True(clean.ExitCode == 0, clean.Output + clean.Error);
            Assert.False(File.Exists(assembly)); Assert.Equal("unowned", File.ReadAllText(unrelated));
        }
        var invalid = await ProjectCliFixture.Cli(directory, "restore", directory.File("absent.csproj"), "--report", "json");
        Assert.Equal(2, invalid.ExitCode); using var failure = JsonDocument.Parse(invalid.Output);
        Assert.Equal("invalid_argument", failure.RootElement.GetProperty("error").GetProperty("code").GetString());
    }
}
