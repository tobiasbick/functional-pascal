using System.Xml.Linq;
using System.Text.RegularExpressions;
using Fpas.Tests.Support;

namespace Fpas.Ui.Gui.Cui.Tests;

/// <summary>Builds an isolated SDK/feed and consumes the complete GUI through package references.</summary>
internal sealed class GuiPackageFixture : IDisposable
{
    private readonly TemporaryDirectory directory = new(prefix: "fpas-gui-host-");
    private readonly string configuration = TestEnvironment.Metadata("FpasBuildConfiguration");
    internal string Root => directory.Path;
    internal string Version { get; } = "0.1.0-gui-host." + Guid.NewGuid().ToString("N");
    internal string Rid { get; } = OperatingSystem.IsWindows() ? "win-x64" : "linux-x64";
    internal string Project => directory.File("consumer/GalleryProbe.fpasproj");
    internal string Compiler => directory.File($"packages/fpas.net.sdk/{Version}/tools/compiler/Fpas.Cli.dll");
    internal string Binding => directory.File($"packages/fpas.ui.gui.cui/{Version}");
    internal Dictionary<string, string?> Environment { get; }
    internal string Artifacts { get; }

    internal GuiPackageFixture()
    {
        Environment = new() { ["NUGET_PACKAGES"] = directory.File("packages"),
            ["LD_LIBRARY_PATH"] = null, ["DYLD_LIBRARY_PATH"] = null };
        string? retained = System.Environment.GetEnvironmentVariable("FPAS_GUI_TEST_ARTIFACTS");
        Artifacts = retained is null ? directory.File("evidence")
            : Path.Combine(retained, Path.GetFileName(Root));
        Directory.CreateDirectory(Artifacts);
    }

    internal string File(string relative) => directory.File(relative);

    internal async Task Prepare()
    {
        // Artifact directories may live inside the checkout; consumers must not inherit its build policy.
        foreach (string name in new[] { "Directory.Build.props", "Directory.Build.targets" })
            System.IO.File.WriteAllText(File(name), "<Project />");
        System.IO.File.WriteAllText(File("Directory.Packages.props"),
            "<Project><PropertyGroup><ManagePackageVersionsCentrally>false</ManagePackageVersionsCentrally></PropertyGroup></Project>");
        Directory.CreateDirectory(File("feed"));
        string sdk = TestEnvironment.Metadata("FpasSdkProject");
        await Check(["msbuild", sdk, "-t:BuildFpasGuiDistributionAssets", "-p:Configuration=" + configuration,
            "-p:FpasCuiDistributionRid=" + Rid], "native", TimeSpan.FromMinutes(5));
        await Check(["pack", sdk, "--no-build", "-c", configuration, "--output", File("feed"),
            "-p:PackageVersion=" + Version, "-p:FpasCuiDistributionRid=" + Rid], "sdk", TimeSpan.FromMinutes(2));
        new XDocument(new XElement("configuration", new XElement("packageSources", new XElement("clear"),
            new XElement("add", new XAttribute("key", "gui-release"), new XAttribute("value", File("feed"))),
            new XElement("add", new XAttribute("key", "nuget.org"), new XAttribute("value", "https://api.nuget.org/v3/index.json")))))
            .Save(File("NuGet.Config"));

        string inputs = Path.Combine(AppContext.BaseDirectory, "Fixtures", "host");
        foreach (string name in new[] { "Fpas.Ui.Mvu", "Fpas.Ui", "Fpas.Ui.Gui" })
        {
            string source = Path.Combine(inputs, "src", name);
            foreach (string file in Directory.GetFiles(source, "*", SearchOption.AllDirectories))
            {
                string target = File("src/" + name + "/" + Path.GetRelativePath(source, file));
                Directory.CreateDirectory(Path.GetDirectoryName(target)!);
                System.IO.File.Copy(file, target);
            }
            string project = File($"src/{name}/{name}.fpasproj");
            var xml = XDocument.Load(project);
            xml.Root!.SetAttributeValue("Sdk", "Fpas.NET.Sdk/" + Version);
            foreach (var reference in xml.Descendants("ProjectReference").Where(reference =>
                reference.Attribute("Include")!.Value.EndsWith("Fpas.Ui.Gui.Cui.csproj", StringComparison.Ordinal)))
                reference.SetAttributeValue("Include", TestEnvironment.Metadata("FpasCuiProject"));
            xml.Save(project);
            await Check(["build", project, "-c", configuration, "-p:BuildProjectReferences=false",
                "-p:RestoreRecursive=false", "-warnaserror"], name + "-build", TimeSpan.FromMinutes(2));
            await Check(["pack", project, "--no-build", "-c", configuration, "--output", File("feed"),
                "-p:PackageVersion=" + Version, "-warnaserror"], name + "-pack", TimeSpan.FromMinutes(2));
        }

        Directory.CreateDirectory(File("consumer"));
        string gallerySources = Path.Combine(inputs, "examples", "ui", "gui", "gallery");
        foreach (string file in Directory.GetFiles(gallerySources, "*.fpas", SearchOption.AllDirectories))
        {
            string relative = Path.GetRelativePath(gallerySources, file);
            if (relative == "main.fpas") continue;
            string target = File("consumer/" + relative);
            Directory.CreateDirectory(Path.GetDirectoryName(target)!);
            System.IO.File.Copy(file, target);
        }
        System.IO.File.Copy(Path.Combine(inputs, "PublishedGallery.fpas"), File("consumer/main.fpas"));
        foreach (string file in Directory.GetFiles(inputs, "*.fpas"))
            if (Path.GetFileName(file) != "PublishedGallery.fpas")
                System.IO.File.Copy(file, File("consumer/" + Path.GetFileName(file)));
        string handbook = System.IO.File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "Fixtures", "GuiHandbook.md"));
        var sample = Regex.Match(handbook, @"```xml\s*\n(?<project>.*?)```", RegexOptions.Singleline);
        Assert.True(sample.Success, "The GUI handbook must contain its complete project.");
        var consumer = XDocument.Parse(sample.Groups["project"].Value);
        consumer.Root!.SetAttributeValue("Sdk", "Fpas.NET.Sdk/" + Version);
        Assert.Single(consumer.Descendants("PackageReference"), item => item.Attribute("Include")?.Value == "Fpas.Ui.Gui")
            .SetAttributeValue("Version", "[" + Version + "]");
        consumer.Save(Project);
        await Check(["build", Project, "-c", configuration, "-warnaserror"], "consumer", TimeSpan.FromMinutes(3));
        Assert.True(System.IO.File.Exists(Binding + "/native/" + Rid + "/static/" +
            (OperatingSystem.IsWindows() ? "fpas_cui.lib" : "libfpas_cui.a")));
    }

    internal async Task Check(string[] arguments, string log, TimeSpan? deadline = null)
    {
        var result = await ProcessRunner.RunAsync(arguments, workingDirectory: Root, environment: Environment,
            timeLimit: deadline ?? TimeSpan.FromSeconds(30));
        System.IO.File.WriteAllText(Path.Combine(Artifacts, log + ".log"), result.Output + result.Error);
        Assert.True(result.ExitCode == 0, log + ": " + result.Output + result.Error);
    }

    public void Dispose() => directory.Dispose();
}
