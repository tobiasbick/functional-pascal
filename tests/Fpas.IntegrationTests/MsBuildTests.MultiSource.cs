using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Checked_in_multi_file_example_builds_through_default_source_globs()
    {
        using var project = sdk.CreateProject();
        var fixture = Path.Combine(AppContext.BaseDirectory, "Fixtures", "units");
        foreach (var source in Directory.GetFiles(fixture, "*", SearchOption.AllDirectories))
        {
            var target = project.File(Path.Combine("Units", Path.GetRelativePath(fixture, source)));
            Directory.CreateDirectory(Path.GetDirectoryName(target)!);
            File.WriteAllText(target, File.ReadAllText(source).Replace("0.1.0-dev", sdk.Version, StringComparison.Ordinal));
        }
        var result = await sdk.Run(project, "build", "Units/Units.fpasproj");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        var executed = await sdk.Run(project, "Units/bin/Debug/net10.0/Units.dll");
        Assert.True(executed.ExitCode == 0, executed.Output + executed.Error);
        Assert.Equal("55" + Environment.NewLine + "The expected sum" + Environment.NewLine, executed.Output);
    }

    [Fact]
    public async Task Multi_unit_library_calls_CSharp_and_is_consumed_by_a_CSharp_host()
    {
        using var project = sdk.CreateProject();
        AddLibraryUnits(project);
        var result = await sdk.Run(project, "build", "Game.slnx");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        var executed = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.True(executed.ExitCode == 0, executed.Output + executed.Error);
        Assert.Equal("C#:FPAS" + Environment.NewLine, executed.Output);
        using var pe = new PEReader(File.OpenRead(project.File("Game/bin/Debug/net10.0/Game.dll")));
        var reader = pe.GetMetadataReader();
        foreach (var name in new[] { "Bridge", "Internal", "Empty" })
            Assert.Contains(reader.TypeDefinitions, handle => reader.GetString(reader.GetTypeDefinition(handle).Name) == name);
    }

    [Fact]
    public async Task Another_Fpas_project_consumes_a_multi_unit_library_through_CLR_metadata()
    {
        using var project = sdk.CreateProject();
        AddLibraryUnits(project);
        Directory.CreateDirectory(project.File("Middle"));
        File.WriteAllText(project.File("Middle/Middle.fpasproj"), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup><OutputType>Library</OutputType></PropertyGroup>
              <ItemGroup><ProjectReference Include="../Game/Game.fpasproj" /></ItemGroup>
            </Project>
            """);
        File.WriteAllText(project.File("Middle/Api.fpas"), """
            unit Demo.Middle;
            uses Demo.Internal as API;
            public function Message(Value: string): string;
            begin return API.Decorate(Value); end function;
            end unit;
            """);
        var host = project.File("Host/Host.csproj");
        File.WriteAllText(host, File.ReadAllText(host).Replace("../Game/Game.fpasproj", "../Middle/Middle.fpasproj", StringComparison.Ordinal));
        File.WriteAllText(project.File("Host/Program.cs"), "Console.WriteLine(Demo.Middle.Message(\"FPAS\"));");
        var build = await sdk.Run(project, "build", "Host/Host.csproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var executed = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.True(executed.ExitCode == 0, executed.Output + executed.Error);
        Assert.Equal("C#:FPAS" + Environment.NewLine, executed.Output);
        Assert.True(File.Exists(project.File("Host/bin/Debug/net10.0/Game.dll")));
        Assert.True(File.Exists(project.File("Host/bin/Debug/net10.0/Middle.dll")));
    }

    [Fact]
    public async Task Incremental_builds_track_secondary_edits_added_and_removed_units()
    {
        using var project = sdk.CreateProject();
        AddLibraryUnits(project);
        var result = await sdk.Run(project, "build", "Game.slnx");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        var unchanged = await sdk.Run(project, "build", "Game.slnx", "--no-restore");
        Assert.True(unchanged.ExitCode == 0, unchanged.Output + unchanged.Error);
        Assert.DoesNotContain("FPAS: compiling", unchanged.Output);
        var helper = project.File("Game/Support.fpas");
        File.WriteAllText(helper, File.ReadAllText(helper).Replace("Shared.Decorate(Value)", "Shared.Decorate(Value + '!')", StringComparison.Ordinal));
        var edited = await sdk.Run(project, "build", "Game.slnx", "--no-restore");
        Assert.True(edited.ExitCode == 0, edited.Output + edited.Error);
        Assert.Contains("FPAS: compiling", edited.Output);
        var executed = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.Equal(0, executed.ExitCode);
        Assert.Equal("C#:FPAS!" + Environment.NewLine, executed.Output);
        var added = project.File("Game/New.fpas");
        File.WriteAllText(added, "unit Demo.New; end unit;");
        var addedBuild = await sdk.Run(project, "build", "Game.slnx", "--no-restore");
        Assert.True(addedBuild.ExitCode == 0, addedBuild.Output + addedBuild.Error);
        Assert.Contains("FPAS: compiling", addedBuild.Output);
        File.Delete(added);
        var removed = await sdk.Run(project, "build", "Game.slnx", "--no-restore");
        Assert.True(removed.ExitCode == 0, removed.Output + removed.Error);
        Assert.Contains("FPAS: compiling", removed.Output);
        File.Delete(helper);
        var missing = await sdk.Run(project, "build", "Game.slnx", "--no-restore");
        Assert.NotEqual(0, missing.ExitCode);
        Assert.Contains("FP3001", missing.Output + missing.Error);
        Assert.Contains("Bridge.fpas", missing.Output + missing.Error);
    }

    private static void AddLibraryUnits(TemporaryDirectory project)
    {
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            uses Demo.Internal as Implementation;
            public function Message(Value: string): string;
            begin return Implementation.Decorate(Value); end function;
            end unit;
            """);
        File.WriteAllText(project.File("Game/Support.fpas"), """
            unit Demo.Internal;
            uses Demo.SharedApi as Shared;
            public function Decorate(Value: string): string;
            begin return Shared.Decorate(Value); end function;
            end unit;
            """);
        File.WriteAllText(project.File("Game/Empty.fpas"), "unit Demo.Empty; end unit;");
    }
}
