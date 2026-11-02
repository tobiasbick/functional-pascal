using System.Text.Json;
using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

public sealed class DocumentationCliTests
{
    [Fact]
    public async Task CLI_documentation_warnings_are_structured_and_warning_errors_preserve_existing_outputs()
    {
        using var directory = new TemporaryDirectory();
        var output = directory.File("Api.dll"); var documentation = directory.File("api docs/custom.xml");
        const string source = """
            unit Api;
            // Summary.
            // ## Returns
            // Invalid for a procedure.
            public procedure P(); begin end procedure;
            end unit;
            """;
        async Task<ProcessResult> Compile(params string[] options) => await ProcessRunner.RunAsync([
            TestEnvironment.Metadata("FpasCliPath"), "compile", "-", "--output", output, "--target", "library",
            "--reference-dir", TestEnvironment.ReferenceDirectory, "--documentation", documentation, "--diagnostics", "json", .. options], source);
        var warning = await Compile(); Assert.True(warning.ExitCode == 0, warning.Output + warning.Error);
        using (var result = JsonDocument.Parse(warning.Output))
        {
            Assert.True(result.RootElement.GetProperty("success").GetBoolean());
            var diagnostic = Assert.Single(result.RootElement.GetProperty("diagnostics").EnumerateArray());
            Assert.Equal("warning", diagnostic.GetProperty("severity").GetString());
            Assert.Equal("FP6001", diagnostic.GetProperty("code").GetString());
            Assert.Equal(3, diagnostic.GetProperty("location").GetProperty("line").GetInt32());
        }
        Assert.Equal("M:Api.P", (string)XDocument.Load(documentation).Descendants("member").Single().Attribute("name")!);
        var assembly = File.ReadAllBytes(output); var pdb = File.ReadAllBytes(Path.ChangeExtension(output, ".pdb")); var xml = File.ReadAllBytes(documentation);
        var error = await Compile("--warn-as-error"); Assert.Equal(1, error.ExitCode);
        using (var result = JsonDocument.Parse(error.Output))
            Assert.Equal("error", Assert.Single(result.RootElement.GetProperty("diagnostics").EnumerateArray()).GetProperty("severity").GetString());
        Assert.Equal(assembly, File.ReadAllBytes(output)); Assert.Equal(pdb, File.ReadAllBytes(Path.ChangeExtension(output, ".pdb"))); Assert.Equal(xml, File.ReadAllBytes(documentation));
        var suppressed = await Compile("--warn-as-error", "--no-warn", "6001"); Assert.Equal(0, suppressed.ExitCode);
        using (var result = JsonDocument.Parse(suppressed.Output)) Assert.Empty(result.RootElement.GetProperty("diagnostics").EnumerateArray());
    }

    [Fact]
    public async Task XML_output_failure_rolls_back_other_artifacts_and_sources_cannot_be_overwritten()
    {
        using var directory = new TemporaryDirectory();
        var source = directory.File("Source.xml"); var output = directory.File("Api.dll"); var documentation = directory.File("other/docs.xml");
        File.WriteAllText(source, "// Original.\nunit Api; end unit;");
        async Task<ProcessResult> Compile(string destination) => await ProcessRunner.RunAsync([
            TestEnvironment.Metadata("FpasCliPath"), "compile", source, "--output", output, "--target", "library",
            "--reference-dir", TestEnvironment.ReferenceDirectory, "--documentation", destination, "--diagnostics", "json"]);
        var initial = await Compile(documentation); Assert.True(initial.ExitCode == 0, initial.Output + initial.Error);
        var assembly = File.ReadAllBytes(output); var pdb = File.ReadAllBytes(Path.ChangeExtension(output, ".pdb"));
        File.Delete(documentation); Directory.CreateDirectory(documentation);
        File.WriteAllText(source, "// Changed.\nunit Api; end unit;");
        var blocked = await Compile(documentation); Assert.Equal(1, blocked.ExitCode);
        Assert.Contains("FP0002", blocked.Output); Assert.True(Directory.Exists(documentation));
        Assert.Equal(assembly, File.ReadAllBytes(output)); Assert.Equal(pdb, File.ReadAllBytes(Path.ChangeExtension(output, ".pdb")));
        Assert.Empty(Directory.GetDirectories(directory.Path, ".fpas-*", SearchOption.AllDirectories));
        var collision = await Compile(source); Assert.Equal(2, collision.ExitCode);
        Assert.Equal("// Changed.\nunit Api; end unit;", File.ReadAllText(source));
    }
}
