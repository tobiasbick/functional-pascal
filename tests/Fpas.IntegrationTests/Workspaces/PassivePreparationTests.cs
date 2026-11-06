using Fpas.IntegrationTests.Build;
using Fpas.Workspaces.Loading;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class PassivePreparationTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Theory]
    [InlineData("InitialTargets")]
    [InlineData("ResolveReferences")]
    [InlineData("ImportedCompile")]
    public async Task Unsupported_custom_preparation_is_rejected_before_any_target_executes(string kind)
    {
        using var directory = sdk.CreateProject(); var path = directory.File("Game/Game.fpasproj");
        var build = await sdk.Run(directory, "build", path, "--disable-build-servers"); Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var xml = XDocument.Load(path); var target = new XElement("Target", new XAttribute("Name", "Surprise"),
            new XElement("Touch", new XAttribute("Files", "surprise.marker"), new XAttribute("AlwaysCreate", "true")));
        if (kind == "InitialTargets") { xml.Root!.SetAttributeValue("InitialTargets", "Surprise"); xml.Root.Add(target); }
        else if (kind == "ResolveReferences") { target.SetAttributeValue("BeforeTargets", "ResolveReferences"); xml.Root!.Add(target); }
        else
        {
            target.SetAttributeValue("BeforeTargets", "FpasPrepareCompilation");
            target.Add(new XElement("ItemGroup", new XElement("Compile", new XAttribute("Include", "Generated.fpas"))));
            new XDocument(new XElement("Project", target)).Save(directory.File("Game/extra.targets"));
            xml.Root!.Add(new XElement("Import", new XAttribute("Project", "extra.targets")));
        }
        xml.Save(path);
        var before = Directory.GetFiles(directory.Path, "*", SearchOption.AllDirectories).ToDictionary(file => file, File.ReadAllBytes);
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(1));
        var context = await WorkspaceLoader.LoadAsync(new(path, Preparation: ContextPreparation.SourceSnapshots), deadline.Token);
        Assert.Empty(context.Projects); Assert.Contains(context.Issues, issue => issue.Code == "preparation_required");
        Assert.False(File.Exists(directory.File("Game/surprise.marker")));
        Assert.Equal(before.Keys.Order(), Directory.GetFiles(directory.Path, "*", SearchOption.AllDirectories).Order());
        foreach (var (file, bytes) in before) Assert.Equal(bytes, File.ReadAllBytes(file));
    }

    [Fact]
    public async Task A_missing_SDK_in_an_imported_dynamic_reference_is_not_downloaded()
    {
        using var directory = sdk.CreateProject(); var path = directory.File("Game/Game.fpasproj");
        var build = await sdk.Run(directory, "build", path, "--disable-build-servers"); Assert.Equal(0, build.ExitCode);
        new XDocument(new XElement("Project", new XElement("PropertyGroup", new XElement("ExtraSdk", "Missing.Sdk")),
            new XElement("Import", new XAttribute("Project", "Sdk.props"), new XAttribute("Sdk", "$(ExtraSdk)"), new XAttribute("Version", "99.0.0"))))
            .Save(directory.File("Game/extra.props"));
        var xml = XDocument.Load(path); xml.Root!.Add(new XElement("Import", new XAttribute("Project", "extra.props"))); xml.Save(path);
        var before = Directory.GetFiles(directory.Path, "*", SearchOption.AllDirectories).Order().ToArray();
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(1));
        var context = await WorkspaceLoader.LoadAsync(new(path, Preparation: ContextPreparation.SourceSnapshots), deadline.Token);
        Assert.Empty(context.Projects); Assert.Contains(context.Issues, issue => issue.Code == "preparation_required");
        Assert.Equal(before, Directory.GetFiles(directory.Path, "*", SearchOption.AllDirectories).Order().ToArray());
    }
}
