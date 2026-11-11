using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Exercises the FPAS MVU owner loop with real CUI presentation and native interactions.</summary>
public sealed class GuiHostTests
{
    /// <summary>Builds the public gallery and verifies its visible pages and controls.</summary>
    /// <param name="renderer">The renderer that must be selected by the FPAS host.</param>
    [GuiTheory]
    [InlineData("software")]
    [InlineData("opengl")]
    public async Task FpasHostOwnsTypedTransitionsAndNativeFrames(string renderer)
    {
        using var directory = new TemporaryDirectory();
        string version = "0.1.0-host." + Guid.NewGuid().ToString("N");
        string configuration = TestEnvironment.Metadata("FpasBuildConfiguration");
        string feed = Directory.CreateDirectory(directory.File("feed")).FullName;
        var pack = await ProcessRunner.RunAsync(["pack", TestEnvironment.Metadata("FpasSdkProject"), "--no-build",
            "-c", configuration, "--output", feed, "-p:PackageVersion=" + version], timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(pack.ExitCode == 0, pack.Output + pack.Error);
        string fixture = Path.Combine(AppContext.BaseDirectory, "Fixtures", "host");
        foreach (string source in Directory.GetFiles(fixture, "*", SearchOption.AllDirectories))
        {
            string target = directory.File(Path.GetRelativePath(fixture, source));
            Directory.CreateDirectory(Path.GetDirectoryName(target)!);
            File.Copy(source, target);
        }
        new XDocument(new XElement("configuration", new XElement("packageSources", new XElement("clear"),
            new XElement("add", new XAttribute("key", "host-sdk"), new XAttribute("value", feed)))))
            .Save(directory.File("NuGet.Config"));
        foreach (string path in Directory.GetFiles(directory.Path, "*.fpasproj", SearchOption.AllDirectories))
        {
            var project = XDocument.Load(path);
            project.Root!.SetAttributeValue("Sdk", "Fpas.NET.Sdk/" + version);
            foreach (var reference in project.Descendants("ProjectReference").Where(reference =>
                reference.Attribute("Include")!.Value.EndsWith("Fpas.Ui.Gui.Cui.csproj", StringComparison.Ordinal)))
                reference.SetAttributeValue("Include", TestEnvironment.Metadata("FpasCuiProject"));
            foreach (var reference in project.Descendants("ProjectReference").Where(reference =>
                reference.Attribute("Include")!.Value.EndsWith("Fpas.Processes.csproj", StringComparison.Ordinal)))
                reference.SetAttributeValue("Include", TestEnvironment.Metadata("FpasProcessesProject"));
            project.Save(path);
        }
        var environment = new Dictionary<string, string?> { ["NUGET_PACKAGES"] = directory.File("packages"), ["LD_LIBRARY_PATH"] = null };
        foreach (string project in new[] { "src/Fpas.Ui.Mvu/Fpas.Ui.Mvu.fpasproj", "src/Fpas.Ui/Fpas.Ui.fpasproj",
            "src/Fpas.Ui.Gui/Fpas.Ui.Gui.fpasproj",
            "examples/ui/gui/gallery/GuiWidgetGallery.fpasproj" })
        {
            var build = await ProcessRunner.RunAsync(["build", directory.File(project), "-c", configuration,
                "-p:BuildProjectReferences=false", "-p:RestoreRecursive=false", "-warnaserror"],
                workingDirectory: directory.Path, environment: environment, timeLimit: TimeSpan.FromMinutes(2));
            Assert.True(build.ExitCode == 0, project + ": " + build.Output + build.Error);
        }
        CheckDocumentation(directory.File($"src/Fpas.Ui.Mvu/bin/{configuration}/net10.0/Fpas.Ui.Mvu.xml"),
            "M:Fpas.Ui.Mvu.Application.Focus``1(", ["Id"], ["TMsg"]);
        CheckDocumentation(directory.File($"src/Fpas.Ui/bin/{configuration}/net10.0/Fpas.Ui.xml"),
            "M:Fpas.Ui.Elements.CachedSubview``2(", ["Id", "Dependencies", "Build", "Equal"], ["TDeps", "TMsg"]);
        CheckDocumentation(directory.File($"src/Fpas.Ui.Gui/bin/{configuration}/net10.0/Fpas.Ui.Gui.xml"),
            "M:Fpas.Ui.Gui.Application.Start``2(", ["InitialModel", "Update", "View", "Settings", "ThemeFor"], ["TModel", "TMsg"]);
        File.Copy(directory.File("PublishedGallery.fpas"), directory.File("examples/ui/gui/gallery/main.fpas"), overwrite: true);
        foreach (string helper in Directory.GetFiles(directory.Path, "*.fpas"))
            if (Path.GetFileName(helper) != "PublishedGallery.fpas")
                File.Copy(helper, directory.File("examples/ui/gui/gallery/" + Path.GetFileName(helper)));
        var galleryBuild = await ProcessRunner.RunAsync(["build", directory.File("examples/ui/gui/gallery/GuiWidgetGallery.fpasproj"), "-c", configuration,
            "-p:BuildProjectReferences=false", "-p:RestoreRecursive=false", "-warnaserror"],
            workingDirectory: directory.Path, environment: environment, timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(galleryBuild.ExitCode == 0, galleryBuild.Output + galleryBuild.Error);
        string galleryCapture = directory.File("gallery.png");
        var gallery = await ProcessRunner.RunAsync([directory.File($"examples/ui/gui/gallery/bin/{configuration}/net10.0/GuiWidgetGallery.dll"), renderer, galleryCapture],
            workingDirectory: directory.Path, environment: environment, timeLimit: TimeSpan.FromSeconds(45));
        Assert.True(gallery.ExitCode == 0, gallery.Output + gallery.Error);
        Assert.Equal("GUI gallery verified: " + renderer + Environment.NewLine, gallery.Output);
        PngAssertions.HasRenderedContent(galleryCapture, 1000, 760);
        string? artifacts = Environment.GetEnvironmentVariable("FPAS_GUI_TEST_ARTIFACTS");
        foreach (var (page, width, height) in new[]
        {
            ("actions", 1000, 760), ("selection", 1100, 1360),
            ("navigation", 1000, 900), ("navigation-compact", 360, 900), ("toasts", 1000, 760),
            ("tooltips", 1000, 900), ("popover", 1000, 900), ("confirm-popover", 1000, 900),
            ("text-fields", 1000, 760), ("text-affixes", 1000, 760), ("edit-profile", 1000, 900)
        })
        {
            string pageCapture = galleryCapture + "." + page + ".png";
            PngAssertions.HasRenderedContent(pageCapture, width, height);
            if (!string.IsNullOrWhiteSpace(artifacts))
            {
                string target = Directory.CreateDirectory(Path.Combine(artifacts, "host-gallery", renderer)).FullName;
                File.Copy(pageCapture, Path.Combine(target, page + ".png"), overwrite: true);
            }
        }
    }

    private static void CheckDocumentation(string path, string methodPrefix, string[] parameters, string[] typeParameters)
    {
        var member = Assert.Single(XDocument.Load(path).Descendants("member"), item =>
            ((string?)item.Attribute("name"))?.StartsWith(methodPrefix, StringComparison.Ordinal) == true);
        Assert.Equal(parameters, member.Elements("param").Select(item => (string?)item.Attribute("name")));
        Assert.Equal(typeParameters, member.Elements("typeparam").Select(item => (string?)item.Attribute("name")));
        foreach (var item in member.Elements().Where(item => item.Name.LocalName is "summary" or "param" or "typeparam" or "returns"))
            Assert.False(string.IsNullOrWhiteSpace(item.Value), path + ": " + item.Name);
        Assert.NotNull(member.Element("summary"));
        Assert.NotNull(member.Element("returns"));
        Assert.False(member.Element("summary")!.Value.StartsWith('/'));
    }
}
