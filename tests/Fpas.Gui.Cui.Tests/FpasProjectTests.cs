using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.Gui.Cui.Tests;

/// <summary>Builds a real FPAS project and runs its emitted CLR/native GUI integration contract.</summary>
public sealed class FpasProjectTests
{
    /// <summary>Checks project-reference delivery, both renderers, FPAS editing and scope cleanup.</summary>
    [GuiFact]
    [Trait("Category", "NativeGui")]
    public async Task FpasProjectBuildsAndRunsWithBothRenderers()
    {
        using var directory = new TemporaryDirectory();
        string version = "0.1.0-gui." + Guid.NewGuid().ToString("N");
        string configuration = TestEnvironment.Metadata("FpasBuildConfiguration");
        string feed = Directory.CreateDirectory(directory.File("feed")).FullName;
        var pack = await ProcessRunner.RunAsync(["pack", TestEnvironment.Metadata("FpasSdkProject"),
            "--no-build", "--configuration", configuration, "--output", feed, "-p:PackageVersion=" + version],
            timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(pack.ExitCode == 0, pack.Output + pack.Error);

        string projectDirectory = Directory.CreateDirectory(directory.File("project")).FullName;
        string fixture = Path.Combine(AppContext.BaseDirectory, "Fixtures", "gui");
        File.Copy(Path.Combine(fixture, "main.fpas"), Path.Combine(projectDirectory, "main.fpas"));
        var project = XDocument.Load(Path.Combine(fixture, "GuiInterop.fpasproj"));
        project.Root!.SetAttributeValue("Sdk", "Fpas.NET.Sdk/" + version);
        project.Descendants("ProjectReference").Single().SetAttributeValue("Include", TestEnvironment.Metadata("FpasCuiProject"));
        project.Save(Path.Combine(projectDirectory, "GuiInterop.fpasproj"));
        new XDocument(new XElement("configuration", new XElement("packageSources", new XElement("clear"),
            new XElement("add", new XAttribute("key", "gui-sdk"), new XAttribute("value", feed)))))
            .Save(directory.File("NuGet.Config"));
        var environment = new Dictionary<string, string?>
        {
            ["NUGET_PACKAGES"] = directory.File("packages"),
            ["LD_LIBRARY_PATH"] = null,
            ["DYLD_LIBRARY_PATH"] = null
        };
        // Consume the solution-built binding without rebuilding it or rewriting its restore assets.
        var build = await ProcessRunner.RunAsync(["build", "GuiInterop.fpasproj", "--configuration", configuration,
            "-p:BuildProjectReferences=false", "-p:RestoreRecursive=false", "-warnaserror"], workingDirectory: projectDirectory,
            environment: environment, timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        string output = Path.Combine(projectDirectory, "bin", configuration, "net10.0");
        Assert.True(File.Exists(Path.Combine(output, OperatingSystem.IsWindows() ? "fpas_cui.dll" : "libfpas_cui.so")));
        Assert.Equal(6, Directory.GetFiles(Path.Combine(output, "licenses", "cui"), "LICENSE-*.txt").Length);
        string workingDirectory = Directory.CreateDirectory(directory.File("run elsewhere")).FullName;
        foreach (string renderer in new[] { "software", "opengl" })
        {
            string capture = Path.Combine(workingDirectory, renderer + ".png");
            var run = await ProcessRunner.RunAsync([Path.Combine(output, "GuiInterop.dll"), renderer, capture],
                workingDirectory: workingDirectory, environment: environment, timeLimit: TimeSpan.FromSeconds(30));
            Assert.True(run.ExitCode == 0, renderer + ": " + run.Output + run.Error);
            Assert.Equal("FPAS GUI verified: " + renderer + "; button; checkbox; UTF-8; normal/exception disposal" + Environment.NewLine,
                run.Output);
            PngAssertions.HasRenderedContent(capture, 640, 480);
        }
    }
}
