using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Projects;
using Fpas.Tests.Support;
using System.Text.Json;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Testing;

public sealed class TestTemplateTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Test_templates_preserve_dry_run_idempotence_conflicts_and_framework_identity()
    {
        using var directory = new TemporaryDirectory();
        var options = new[] { "init", "test", "Tests", "--sdk-version", sdk.Version, "--sdk-feed", sdk.Feed, "--report", "json" };
        var preview = await ProjectCliFixture.Cli(directory, [.. options, "--dry-run"]);
        Assert.Equal(0, preview.ExitCode); Assert.Empty(Directory.EnumerateFileSystemEntries(directory.Path));
        using (var report = JsonDocument.Parse(preview.Output))
            Assert.Contains(report.RootElement.GetProperty("files").EnumerateArray(), file => file.GetProperty("content").GetString()!.Contains("X.FactAttribute", StringComparison.Ordinal));
        var first = await ProjectCliFixture.Cli(directory, options); Assert.Equal(0, first.ExitCode);
        var repeated = await ProjectCliFixture.Cli(directory, options); Assert.Equal(0, repeated.ExitCode);
        using (var report = JsonDocument.Parse(repeated.Output))
            Assert.All(report.RootElement.GetProperty("files").EnumerateArray(), file => Assert.Equal("unchanged", file.GetProperty("state").GetString()));
        var xml = XDocument.Load(directory.File("Tests/Tests.fpasproj"));
        Assert.Equal("true", xml.Root!.Element("PropertyGroup")!.Element("IsTestProject")!.Value);
        Assert.Contains(xml.Descendants("PackageReference"), package => (string?)package.Attribute("Include") == "xunit");
        var conflict = await ProjectCliFixture.Cli(directory, [.. options, "--framework", "nunit"]);
        Assert.Equal(1, conflict.ExitCode); Assert.Equal(xml.ToString(), XDocument.Load(directory.File("Tests/Tests.fpasproj")).ToString());
        var invalid = await ProjectCliFixture.Cli(directory, "init", "test", "Invalid", "--framework", "custom", "--report", "json");
        Assert.Equal(2, invalid.ExitCode); Assert.False(Directory.Exists(directory.File("Invalid")));
        using var failed = JsonDocument.Parse(invalid.Output); Assert.False(failed.RootElement.GetProperty("success").GetBoolean());
    }

    [Fact]
    public async Task Test_libraries_reject_program_entries_and_MTP_templates_require_MSTest()
    {
        using var directory = new TemporaryDirectory();
        var project = await TestProjectFixture.Create(directory, sdk, "xunit");
        File.WriteAllText(directory.File("Tests/Tests.fpas"), "program NotATest; begin end; end program;");
        var program = await ProjectCliFixture.Cli(directory, "build", project, "--report", "json");
        Assert.Equal(1, program.ExitCode);
        using (var report = JsonDocument.Parse(program.Output)) Assert.Contains("library", report.RootElement.GetProperty("build").GetProperty("output").GetString(), StringComparison.OrdinalIgnoreCase);
        var mode = await ProjectCliFixture.Cli(directory, "init", "test", "InvalidMtp", "--runner", "mtp", "--framework", "xunit", "--report", "json");
        Assert.Equal(2, mode.ExitCode);
        using var failure = JsonDocument.Parse(mode.Output); Assert.Equal("invalid_argument", failure.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.False(Directory.Exists(directory.File("InvalidMtp")));
    }
}
