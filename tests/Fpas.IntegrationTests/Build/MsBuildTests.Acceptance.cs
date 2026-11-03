using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Parallel_configurations_and_projects_isolate_outputs_and_match_clean_rebuilds()
    {
        using var project = sdk.CreateProject();
        Directory.CreateDirectory(project.File("Peer"));
        File.WriteAllText(project.File("Peer/Peer.fpasproj"), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup><OutputType>Library</OutputType><GenerateDocumentationFile>true</GenerateDocumentationFile></PropertyGroup>
            </Project>
            """);
        File.WriteAllText(project.File("Peer/Peer.fpas"), """
            unit Demo.Peer;
            // Returns a value from the independent project.
            public function Value(): int;
            begin return 17; end function;
            end unit;
            """);
        var solution = XDocument.Load(project.File("Game.slnx"));
        solution.Root!.Add(new XElement("Project", new XAttribute("Path", "Peer/Peer.fpasproj")));
        solution.Save(project.File("Game.slnx"));
        var host = XDocument.Load(project.File("Host/Host.csproj"));
        host.Root!.Add(new XElement("ItemGroup", new XElement("ProjectReference", new XAttribute("Include", "../Peer/Peer.fpasproj"))));
        host.Save(project.File("Host/Host.csproj"));
        File.AppendAllText(project.File("Host/Program.cs"), "\nConsole.WriteLine(Demo.Peer.Value());\n");
        var restore = await sdk.Run(project, "restore", "Game.slnx");
        Assert.True(restore.ExitCode == 0, restore.Output + restore.Error);
        var builds = await Task.WhenAll(new[] { "Debug", "Release" }.Select(configuration =>
            sdk.Run(project, "build", "Game.slnx", "--no-restore", "--configuration", configuration, "-maxcpucount")));
        Assert.All(builds, result => Assert.True(result.ExitCode == 0, result.Output + result.Error));
        foreach (var configuration in new[] { "Debug", "Release" })
        {
            var run = await sdk.Run(project, $"Host/bin/{configuration}/net10.0/Host.dll");
            Assert.Equal(0, run.ExitCode);
            Assert.Equal("C#:FPAS" + Environment.NewLine + "17" + Environment.NewLine, run.Output);
            foreach (var name in new[] { "Game", "Peer" })
            {
                Assert.True(File.Exists(project.File($"{name}/obj/{configuration}/net10.0/fpas.compile.json")));
                Assert.Equal(File.ReadAllBytes(project.File($"{name}/bin/{configuration}/net10.0/{name}.dll")),
                    File.ReadAllBytes(project.File($"Host/bin/{configuration}/net10.0/{name}.dll")));
            }
            Assert.True(File.Exists(project.File($"Host/bin/{configuration}/net10.0/Peer.xml")));
        }
        var assembly = File.ReadAllBytes(project.File("Peer/bin/Debug/net10.0/Peer.dll"));
        var symbols = File.ReadAllBytes(project.File("Peer/bin/Debug/net10.0/Peer.pdb"));
        var documentation = File.ReadAllBytes(project.File("Peer/bin/Debug/net10.0/Peer.xml"));
        var rebuild = await sdk.Run(project, "msbuild", "Game.slnx", "-t:Rebuild", "-p:Configuration=Debug", "-maxcpucount");
        Assert.True(rebuild.ExitCode == 0, rebuild.Output + rebuild.Error);
        Assert.Equal(assembly, File.ReadAllBytes(project.File("Peer/bin/Debug/net10.0/Peer.dll")));
        Assert.Equal(symbols, File.ReadAllBytes(project.File("Peer/bin/Debug/net10.0/Peer.pdb")));
        Assert.Equal(documentation, File.ReadAllBytes(project.File("Peer/bin/Debug/net10.0/Peer.xml")));
        Assert.True(File.Exists(project.File("Peer/bin/Release/net10.0/Peer.dll")));
        var negotiation = await sdk.Run(project, "msbuild", "Peer/Peer.fpasproj", "-getTargetResult:GetTargetFrameworks,GetTargetPath", "-verbosity:quiet");
        Assert.True(negotiation.ExitCode == 0, negotiation.Output + negotiation.Error);
        Assert.Contains("net10.0", negotiation.Output); Assert.Contains("Peer.dll", negotiation.Output);
    }

    [Fact]
    public async Task Compiler_assets_and_reference_list_changes_invalidate_incremental_state()
    {
        using var project = sdk.CreateProject();
        Directory.CreateDirectory(project.File("compiler"));
        foreach (var file in Directory.GetFiles(Path.GetDirectoryName(TestEnvironment.Metadata("FpasCliPath"))!))
            File.Copy(file, project.File("compiler/" + Path.GetFileName(file)));
        var compilerOption = "-p:FpasCompilerPath=" + project.File("compiler/Fpas.Cli.dll");
        async Task<ProcessResult> Build()
        {
            var result = await sdk.Run(project, "build", "Game.slnx", compilerOption);
            Assert.True(result.ExitCode == 0, result.Output + result.Error); return result;
        }
        await Build();
        Assert.DoesNotContain("FPAS: compiling", (await Build()).Output);
        var assembly = File.ReadAllBytes(project.File("Game/bin/Debug/net10.0/Game.dll"));
        File.AppendAllText(project.File("compiler/Fpas.Cli.runtimeconfig.json"), "\n");
        Assert.Contains("FPAS: compiling", (await Build()).Output);
        Assert.Equal(assembly, File.ReadAllBytes(project.File("Game/bin/Debug/net10.0/Game.dll")));
        Assert.DoesNotContain("FPAS: compiling", (await Build()).Output);
        Directory.CreateDirectory(project.File("Extra"));
        File.WriteAllText(project.File("Extra/Extra.csproj"), "<Project Sdk=\"Microsoft.NET.Sdk\" />");
        File.WriteAllText(project.File("Extra/Extra.cs"), "namespace Extra; public sealed class Value { }");
        var extra = await sdk.Run(project, "build", "Extra/Extra.csproj");
        Assert.True(extra.ExitCode == 0, extra.Output + extra.Error);
        var path = project.File("Game/Game.fpasproj");
        var definition = XDocument.Load(path);
        var item = new XElement("Reference", new XAttribute("Include", "Extra"), new XElement("HintPath", "../Extra/bin/Debug/net10.0/Extra.dll"));
        definition.Root!.Add(new XElement("ItemGroup", item)); definition.Save(path);
        Assert.Contains("FPAS: compiling", (await Build()).Output);
        item.Remove(); definition.Save(path);
        Assert.Contains("FPAS: compiling", (await Build()).Output);
        Assert.Equal(assembly, File.ReadAllBytes(project.File("Game/bin/Debug/net10.0/Game.dll")));
        Assert.DoesNotContain("FPAS: compiling", (await Build()).Output);
    }
}
