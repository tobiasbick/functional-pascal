using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.Gui.Cui.Tests;

/// <summary>Exercises the FPAS MVU owner loop with real CUI presentation and native interactions.</summary>
public sealed class GuiHostTests
{
    /// <summary>Builds the public examples and checks frame, input, validation, lifetime and queue contracts.</summary>
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
                reference.Attribute("Include")!.Value.EndsWith("Fpas.Gui.Cui.csproj", StringComparison.Ordinal)))
                reference.SetAttributeValue("Include", TestEnvironment.Metadata("FpasCuiProject"));
            foreach (var reference in project.Descendants("ProjectReference").Where(reference =>
                reference.Attribute("Include")!.Value.EndsWith("Fpas.Processes.csproj", StringComparison.Ordinal)))
                reference.SetAttributeValue("Include", TestEnvironment.Metadata("FpasProcessesProject"));
            project.Save(path);
        }
        var environment = new Dictionary<string, string?> { ["NUGET_PACKAGES"] = directory.File("packages"), ["LD_LIBRARY_PATH"] = null };
        foreach (string project in new[] { "src/Fpas.Mvu/Fpas.Mvu.fpasproj", "src/Fpas.Ui/Fpas.Ui.fpasproj",
            "src/Fpas.Gui/Fpas.Gui.fpasproj", "src/Fpas.Tui/Fpas.Tui.fpasproj",
            "examples/gui/counter/GuiCounter.fpasproj", "examples/gui/gallery/GuiWidgetGallery.fpasproj" })
        {
            var build = await ProcessRunner.RunAsync(["build", directory.File(project), "-c", configuration,
                "-p:BuildProjectReferences=false", "-p:RestoreRecursive=false", "-warnaserror"],
                workingDirectory: directory.Path, environment: environment, timeLimit: TimeSpan.FromMinutes(2));
            Assert.True(build.ExitCode == 0, project + ": " + build.Output + build.Error);
        }
        File.Copy(directory.File("HostContract.fpas"), directory.File("examples/gui/counter/main.fpas"), overwrite: true);
        var compiled = await ProcessRunner.RunAsync(["build", directory.File("examples/gui/counter/GuiCounter.fpasproj"), "-c", configuration,
            "-p:BuildProjectReferences=false", "-p:RestoreRecursive=false", "-warnaserror"],
            workingDirectory: directory.Path, environment: environment, timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(compiled.ExitCode == 0, compiled.Output + compiled.Error);
        string capture = directory.File("host.png");
        var run = await ProcessRunner.RunAsync([directory.File($"examples/gui/counter/bin/{configuration}/net10.0/GuiCounter.dll"), renderer, capture],
            workingDirectory: directory.Path, environment: environment, timeLimit: TimeSpan.FromSeconds(30));
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("GUI host verified: " + renderer + Environment.NewLine, run.Output);
        PngAssertions.HasRenderedContent(capture, 640, 480);
        File.Copy(directory.File("GalleryContract.fpas"), directory.File("examples/gui/gallery/main.fpas"), overwrite: true);
        var galleryBuild = await ProcessRunner.RunAsync(["build", directory.File("examples/gui/gallery/GuiWidgetGallery.fpasproj"), "-c", configuration,
            "-p:BuildProjectReferences=false", "-p:RestoreRecursive=false", "-warnaserror"],
            workingDirectory: directory.Path, environment: environment, timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(galleryBuild.ExitCode == 0, galleryBuild.Output + galleryBuild.Error);
        string galleryCapture = directory.File("gallery.png");
        var gallery = await ProcessRunner.RunAsync([directory.File($"examples/gui/gallery/bin/{configuration}/net10.0/GuiWidgetGallery.dll"), renderer, galleryCapture],
            workingDirectory: directory.Path, environment: environment, timeLimit: TimeSpan.FromSeconds(45));
        Assert.True(gallery.ExitCode == 0, gallery.Output + gallery.Error);
        Assert.Equal("GUI gallery verified: " + renderer + Environment.NewLine, gallery.Output);
        PngAssertions.HasRenderedContent(galleryCapture, 1000, 760);

        if (renderer != "software") return;
        File.WriteAllText(directory.File("Portable.fpasproj"), $"""
            <Project Sdk="Fpas.NET.Sdk/{version}">
              <PropertyGroup>
                <TargetFramework>net10.0</TargetFramework>
                <OutputType>Exe</OutputType>
                <EnableDefaultFpasCompileItems>false</EnableDefaultFpasCompileItems>
              </PropertyGroup>
              <ItemGroup>
                <Compile Remove="**/*.fpas" />
                <Compile Include="PortableContract.fpas" />
                <ProjectReference Include="src/Fpas.Tui/Fpas.Tui.fpasproj" />
                <ProjectReference Include="src/Fpas.Gui/Fpas.Gui.fpasproj" />
              </ItemGroup>
            </Project>
            """);
        var portableBuild = await ProcessRunner.RunAsync(["build", directory.File("Portable.fpasproj"), "-c", configuration,
            "-p:BuildProjectReferences=false", "-p:RestoreRecursive=false", "-warnaserror"],
            workingDirectory: directory.Path, environment: environment, timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(portableBuild.ExitCode == 0, portableBuild.Output + portableBuild.Error);
        var portable = await ProcessRunner.RunAsync([directory.File($"bin/{configuration}/net10.0/Portable.dll")],
            workingDirectory: directory.Path, environment: environment, timeLimit: TimeSpan.FromSeconds(30));
        Assert.True(portable.ExitCode == 0, portable.Output + portable.Error);
        Assert.Equal("portable TUI and GUI contracts verified" + Environment.NewLine, portable.Output);
    }
}
