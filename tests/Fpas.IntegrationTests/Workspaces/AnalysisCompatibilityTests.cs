using Fpas.Compiler;
using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using Fpas.Workspaces.Loading;
using System.IO.Compression;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class AnalysisCompatibilityTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Missing_analysis_targets_in_an_old_SDK_produce_an_actionable_error()
    {
        using var project = sdk.CreateProject();
        var legacy = sdk.Version + "-legacy";
        var package = Path.Combine(sdk.Feed, "Fpas.NET.Sdk." + legacy + ".nupkg");
        File.Copy(Path.Combine(sdk.Feed, "Fpas.NET.Sdk." + sdk.Version + ".nupkg"), package);
        using (var zip = ZipFile.Open(package, ZipArchiveMode.Update))
        {
            var specification = zip.Entries.Single(entry => entry.FullName.EndsWith(".nuspec", StringComparison.Ordinal));
            XDocument manifest;
            using (var stream = specification.Open()) manifest = XDocument.Load(stream);
            manifest.Descendants().Single(element => element.Name.LocalName == "version").Value = legacy;
            using (var stream = specification.Open()) { stream.SetLength(0); manifest.Save(stream); }
            var targets = zip.GetEntry("Sdk/Sdk.targets")!;
            XDocument xml;
            using (var stream = targets.Open()) xml = XDocument.Load(stream);
            xml.Descendants("Target").Where(element => (string?)element.Attribute("Name") is "FpasDescribeAnalysisContext" or "FpasExportAnalysisContext").Remove();
            using (var stream = targets.Open()) { stream.SetLength(0); xml.Save(stream); }
        }
        var path = project.File("Game/Game.fpasproj");
        File.WriteAllText(path, File.ReadAllText(path).Replace(sdk.Version, legacy, StringComparison.Ordinal));
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        var context = await WorkspaceLoader.LoadAsync(new(path), deadline.Token);
        Assert.Empty(context.Projects);
        Assert.Contains(context.Issues, issue => issue.Code == "context_failed" && issue.Message.Contains("matching FPAS SDK", StringComparison.Ordinal));
    }

    [Fact]
    public async Task Mixed_language_graph_and_separate_executables_have_independent_compilation_contexts()
    {
        using var project = sdk.CreateProject();
        var propsPath = project.File("Directory.Build.props");
        var props = XDocument.Load(propsPath);
        props.Root!.Element("PropertyGroup")!.Add(new XElement("RestoreAdditionalProjectSources", "$(MSBuildSDKsPath)/../FSharp/library-packs"), new XElement("NuGetAudit", "false"));
        props.Save(propsPath);
        Directory.CreateDirectory(project.File("FSharp"));
        File.WriteAllText(project.File("FSharp/Shared.fsproj"), """
            <Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><AssemblyName>FSharpDependency</AssemblyName></PropertyGroup><ItemGroup><Compile Include="Api.fs" /></ItemGroup></Project>
            """);
        File.WriteAllText(project.File("FSharp/Api.fs"), "namespace Foreign\ntype Api = static member Value = 42\n");
        var gamePath = project.File("Game/Game.fpasproj");
        var game = XDocument.Load(gamePath);
        game.Root!.Add(new XElement("ItemGroup", new XElement("ProjectReference", new XAttribute("Include", "../FSharp/Shared.fsproj"))));
        game.Save(gamePath);
        File.WriteAllText(project.File("Game/Bridge.fpas"), "unit Demo.Bridge; uses Foreign.Api as Api; public function Answer(): int; begin return Api.Value; end function; end unit;");
        var solutionPath = project.File("Game.slnx");
        var solution = XDocument.Load(solutionPath);
        foreach (var name in new[] { "First", "Second" })
        {
            Directory.CreateDirectory(project.File(name));
            File.WriteAllText(project.File(name + "/Program.fpas"), "program " + name + "; begin end; end program;");
            File.WriteAllText(project.File(name + "/" + name + ".fpasproj"), $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Exe</OutputType></PropertyGroup></Project>");
            solution.Root!.Add(new XElement("Project", new XAttribute("Path", name + "/" + name + ".fpasproj")));
        }
        solution.Save(solutionPath);
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        var context = await WorkspaceLoader.LoadAsync(new(solutionPath), deadline.Token);
        Assert.Empty(context.Issues);
        Assert.Equal(3, context.Projects.Count);
        foreach (var input in context.Projects) Assert.True(FpasCompiler.Analyze(input.Request, deadline.Token).Success);
        Assert.Contains(context.DependencyOutputs, output => Path.GetFileName(output.Path) == "Shared.dll");
        Assert.Contains(context.Projects.Single(input => input.Request.AssemblyName == "Game").Request.References, reference => Path.GetFileName(reference) == "FSharp.Core.dll");
        Assert.False(File.Exists(project.File("First/bin/Debug/net10.0/First.dll")));
        Assert.False(File.Exists(project.File("Second/bin/Debug/net10.0/Second.dll")));
    }
}
