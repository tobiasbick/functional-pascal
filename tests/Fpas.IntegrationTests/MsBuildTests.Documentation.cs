using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

public sealed partial class MsBuildTests
{
    [Theory]
    [InlineData("Debug", false)]
    [InlineData("Release", true)]
    public async Task Documentation_flows_to_CSharp_tracks_comments_outputs_options_and_cleanup(string configuration, bool custom)
    {
        using var project = sdk.CreateProject();
        var projectFile = project.File("Game/Game.fpasproj");
        var definition = XDocument.Load(projectFile); var properties = definition.Root!.Element("PropertyGroup")!;
        if (custom) properties.Add(new XElement("DocumentationFile", "api docs/Export.xml"));
        else properties.Add(new XElement("GenerateDocumentationFile", "true"));
        definition.Save(projectFile);
        var source = project.File("Game/Bridge.fpas");
        File.WriteAllText(source, File.ReadAllText(source).Replace("public function Message", "// Original summary.\n// ## Parameters\n// - `Value`: Text.\npublic function Message", StringComparison.Ordinal));
        async Task<ProcessResult> Build()
        {
            var result = await sdk.Run(project, "build", "Game.slnx", "--configuration", configuration);
            Assert.True(result.ExitCode == 0, result.Output + result.Error); return result;
        }
        await Build();
        var intermediate = project.File($"Game/obj/{configuration}/net10.0/Game.xml");
        var output = project.File($"Game/bin/{configuration}/net10.0/Game.xml");
        var consumer = project.File($"Host/bin/{configuration}/net10.0/Game.xml");
        var requested = custom ? project.File("Game/api docs/Export.xml") : output;
        var original = File.ReadAllBytes(intermediate);
        Assert.Equal(original, File.ReadAllBytes(output)); Assert.Equal(original, File.ReadAllBytes(consumer)); Assert.Equal(original, File.ReadAllBytes(requested));
        Assert.Equal("Original summary.", XDocument.Load(output).Descendants("member").Single(member => (string?)member.Attribute("name") == "M:Demo.Bridge.Message(System.String)").Element("summary")!.Value);
        Assert.DoesNotContain("FPAS: compiling", (await Build()).Output);
        File.WriteAllText(source, File.ReadAllText(source).Replace("Original summary.", "Edited summary.", StringComparison.Ordinal));
        Assert.Contains("FPAS: compiling", (await Build()).Output);
        var edited = File.ReadAllBytes(intermediate); Assert.NotEqual(original, edited); Assert.Equal(edited, File.ReadAllBytes(consumer));
        File.Delete(requested); Assert.Contains("FPAS: compiling", (await Build()).Output); Assert.Equal(edited, File.ReadAllBytes(requested));
        File.Delete(intermediate); Assert.Contains("FPAS: compiling", (await Build()).Output);
        File.WriteAllText(requested, "damaged"); await Build(); Assert.Equal(edited, File.ReadAllBytes(requested));
        var unrelated = project.File("Game/api docs/unrelated.xml"); Directory.CreateDirectory(Path.GetDirectoryName(unrelated)!); File.WriteAllText(unrelated, "keep");
        var disabled = properties.Element("GenerateDocumentationFile");
        if (disabled is null) properties.Add(new XElement("GenerateDocumentationFile", "false")); else disabled.Value = "false";
        definition.Save(projectFile); await Build();
        Assert.False(File.Exists(intermediate)); Assert.False(File.Exists(output)); Assert.False(File.Exists(requested)); Assert.False(File.Exists(consumer));
        Assert.Equal("keep", File.ReadAllText(unrelated));
        properties.Element("GenerateDocumentationFile")!.Value = "true"; definition.Save(projectFile);
        await Build(); Assert.True(File.Exists(output));
        var clean = await sdk.Run(project, "clean", "Game.slnx", "--configuration", configuration); Assert.True(clean.ExitCode == 0, clean.Output + clean.Error);
        Assert.False(File.Exists(intermediate)); Assert.False(File.Exists(output)); Assert.False(File.Exists(requested)); Assert.False(File.Exists(consumer)); Assert.Equal("keep", File.ReadAllText(unrelated));
    }

    [Fact]
    public async Task Documentation_warnings_follow_MSBuild_policy_and_unchanged_builds_retain_diagnostics()
    {
        using var project = sdk.CreateProject();
        var path = project.File("Game/Game.fpasproj"); var definition = XDocument.Load(path); var properties = definition.Root!.Element("PropertyGroup")!;
        properties.Add(new XElement("GenerateDocumentationFile", "true")); definition.Save(path);
        var source = project.File("Game/Bridge.fpas");
        File.WriteAllText(source, File.ReadAllText(source).Replace("public function Message", "// ## Parameters\n// - `Unknown`: Invalid.\npublic function Message", StringComparison.Ordinal));
        var warned = await sdk.Run(project, "build", "Game/Game.fpasproj"); Assert.True(warned.ExitCode == 0, warned.Output + warned.Error); Assert.Contains("warning FP6001", warned.Output);
        var unchanged = await sdk.Run(project, "build", "Game/Game.fpasproj", "--no-restore"); Assert.Equal(0, unchanged.ExitCode); Assert.Contains("warning FP6001", unchanged.Output); Assert.DoesNotContain("FPAS: compiling", unchanged.Output);
        properties.Add(new XElement("TreatWarningsAsErrors", "true")); definition.Save(path);
        var failed = await sdk.Run(project, "build", "Game/Game.fpasproj"); Assert.NotEqual(0, failed.ExitCode); Assert.Contains("error FP6001", failed.Output);
        properties.Add(new XElement("WarningsNotAsErrors", "FP6001")); definition.Save(path);
        var excepted = await sdk.Run(project, "build", "Game/Game.fpasproj"); Assert.Equal(0, excepted.ExitCode); Assert.Contains("warning FP6001", excepted.Output);
        properties.Add(new XElement("NoWarn", "FP6001")); definition.Save(path);
        var suppressed = await sdk.Run(project, "build", "Game/Game.fpasproj"); Assert.True(suppressed.ExitCode == 0, suppressed.Output + suppressed.Error); Assert.DoesNotContain("FP6001", suppressed.Output);
    }
}
