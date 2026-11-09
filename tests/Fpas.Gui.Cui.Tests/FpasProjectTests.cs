using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.Gui.Cui.Tests;

/// <summary>Builds a real FPAS project and runs its emitted CLR/native GUI integration contract.</summary>
public sealed partial class FpasProjectTests
{
    /// <summary>Checks project-reference delivery, both renderers, FPAS editing and scope cleanup.</summary>
    [GuiFact]
    [Trait("Category", "NativeGui")]
    public async Task FpasProjectBuildsAndRunsWithBothRenderers()
    {
        using var directory = new TemporaryDirectory();
        var (projectDirectory, _, environment) = await PrepareGuiProject(directory);
        string configuration = TestEnvironment.Metadata("FpasBuildConfiguration");
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
            CheckGuiRun(run, renderer, capture);
        }
    }

    private static async Task<(string Project, string Sdk, Dictionary<string, string?> Environment)> PrepareGuiProject(
        TemporaryDirectory directory, string? packageRid = null)
    {
        string version = "0.1.0-gui." + Guid.NewGuid().ToString("N");
        string configuration = TestEnvironment.Metadata("FpasBuildConfiguration");
        string sdkProject = TestEnvironment.Metadata("FpasSdkProject");
        string feed = Directory.CreateDirectory(directory.File("feed")).FullName;
        string[] distribution = packageRid is null ? [] : ["-p:FpasCuiDistributionRid=" + packageRid];
        if (packageRid is not null)
        {
            var native = await ProcessRunner.RunAsync(["msbuild", sdkProject, "-t:BuildFpasGuiDistributionAssets",
                "-p:Configuration=" + configuration, .. distribution], timeLimit: TimeSpan.FromMinutes(5));
            Assert.True(native.ExitCode == 0, native.Output + native.Error);
        }
        var pack = await ProcessRunner.RunAsync(["pack", sdkProject, "--no-build", "--configuration", configuration,
            "--output", feed, "-p:PackageVersion=" + version, "-warnaserror", .. distribution],
            timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(pack.ExitCode == 0, pack.Output + pack.Error);

        string projectDirectory = Directory.CreateDirectory(directory.File("project")).FullName;
        string fixture = Path.Combine(AppContext.BaseDirectory, "Fixtures", "gui");
        File.Copy(Path.Combine(fixture, "main.fpas"), Path.Combine(projectDirectory, "main.fpas"));
        var project = XDocument.Load(Path.Combine(fixture, packageRid is null ? "GuiInterop.fpasproj" : "GuiPackaged.fpasproj"));
        project.Root!.SetAttributeValue("Sdk", "Fpas.NET.Sdk/" + version);
        string packages = directory.File("packages");
        string sdk = Path.Combine(packages, "fpas.net.sdk", version);
        var sources = new XElement("packageSources", new XElement("clear"),
            new XElement("add", new XAttribute("key", "gui-sdk"), new XAttribute("value", feed)));
        if (packageRid is null)
        {
            project.Descendants("ProjectReference").Single().SetAttributeValue("Include", TestEnvironment.Metadata("FpasCuiProject"));
        }
        else
        {
            string binding = Directory.CreateDirectory(directory.File("binding")).FullName;
            new XDocument(new XElement("Project", new XAttribute("Sdk", "Microsoft.NET.Sdk"),
                new XElement("PropertyGroup", new XElement("TargetFramework", "net10.0")),
                new XElement("ItemGroup", new XElement("PackageReference", new XAttribute("Include", "Fpas.Gui.Cui"),
                    new XAttribute("Version", "[" + version + "]"))))).Save(Path.Combine(binding, "Binding.csproj"));
            sources.Add(new XElement("add", new XAttribute("key", "nuget.org"),
                new XAttribute("value", "https://api.nuget.org/v3/index.json")));
        }
        project.Save(Path.Combine(projectDirectory, "GuiInterop.fpasproj"));
        new XDocument(new XElement("configuration", sources)).Save(directory.File("NuGet.Config"));
        var environment = new Dictionary<string, string?>
        {
            ["NUGET_PACKAGES"] = packages,
            ["LD_LIBRARY_PATH"] = null,
            ["DYLD_LIBRARY_PATH"] = null
        };
        return (projectDirectory, sdk, environment);
    }

    private static void CheckGuiRun(ProcessResult run, string renderer, string capture)
    {
        Assert.True(run.ExitCode == 0, renderer + ": " + run.Output + run.Error);
        Assert.Equal("FPAS GUI verified: " + renderer + "; input; layouts; modal; UTF-8; image; normal/exception disposal" + Environment.NewLine,
            run.Output);
        Assert.Contains(0xff0000ffu, PngAssertions.ReadPixels(capture, 640, 480));
    }
}
