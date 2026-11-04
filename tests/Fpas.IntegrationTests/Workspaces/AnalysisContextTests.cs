using Fpas.Compiler;
using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using Fpas.Workspaces.Loading;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class AnalysisContextTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Evaluated_project_context_prepares_references_without_publishing_the_root()
    {
        using var project = sdk.CreateProject();
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        var context = await WorkspaceLoader.LoadAsync(new(project.File("Game/Game.fpasproj")), deadline.Token);
        Assert.Empty(context.Issues);
        var input = Assert.Single(context.Projects);
        Assert.Equal("Game", input.Request.AssemblyName);
        Assert.Contains(input.Request.References, path => Path.GetFileName(path) == "Shared.dll");
        Assert.Contains(input.Request.References, path => Path.GetFileName(path) == "Fpas.Core.dll");
        Assert.Contains(context.DependencyOutputs, output => Path.GetFileName(output.Path) == "Shared.dll");
        Assert.True(File.Exists(project.File("Shared/bin/Debug/net10.0/Shared.dll")));
        Assert.False(File.Exists(project.File("Game/bin/Debug/net10.0/Game.dll")));
        Assert.False(File.Exists(project.File("Game/obj/Debug/net10.0/fpas.compile.json")));
        Assert.True(FpasCompiler.Analyze(input.Request, deadline.Token).Success);
        await context.VerifyInputsAsync(deadline.Token);
        File.AppendAllText(project.File("Game/Bridge.fpas"), "\n// changed after context capture\n");
        Assert.Equal("inputs_changed", (await Assert.ThrowsAsync<ContextFailure>(() => context.VerifyInputsAsync())).Code);
    }

    [Fact]
    public async Task Solution_context_discovers_FPAs_through_foreign_project_edges_and_honors_configuration()
    {
        using var project = sdk.CreateProject();
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        var context = await WorkspaceLoader.LoadAsync(new(project.File("Game.slnx"), "Release"), deadline.Token);
        Assert.Empty(context.Issues);
        var input = Assert.Single(context.Projects);
        Assert.Equal("Release", input.Configuration);
        Assert.Equal(project.File("Game/Game.fpasproj"), input.Path);
        Assert.True(FpasCompiler.Analyze(input.Request, deadline.Token).Success);
        Assert.False(File.Exists(project.File("Game/bin/Release/net10.0/Game.dll")));
    }

    [Fact]
    public async Task Conditional_items_and_warning_policy_are_exported_from_MSBuild()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Extra.fpas"), "unit Extra; public function Read(): int; begin return 42; end function; end unit;");
        File.WriteAllText(project.File("Game/Game.fpasproj"), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup>
                <OutputType>Library</OutputType>
                <GenerateDocumentationFile>true</GenerateDocumentationFile>
                <TreatWarningsAsErrors>true</TreatWarningsAsErrors>
                <NoWarn>FP6001</NoWarn>
              </PropertyGroup>
              <ItemGroup>
                <ProjectReference Include="../Shared/Shared.csproj" />
                <Compile Remove="Extra.fpas" Condition="'$(Configuration)' == 'Debug'" />
              </ItemGroup>
            </Project>
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        var debug = await WorkspaceLoader.LoadAsync(new(project.File("Game/Game.fpasproj")), deadline.Token);
        Assert.Empty(debug.Issues);
        var input = Assert.Single(debug.Projects);
        Assert.Single(input.Request.Sources); Assert.True(input.Request.ValidateDocumentation);
        Assert.True(input.Request.DiagnosticOptions!.TreatWarningsAsErrors);
        Assert.Contains("FP6001", input.Request.DiagnosticOptions.NoWarn!);
        var release = await WorkspaceLoader.LoadAsync(new(project.File("Game/Game.fpasproj"), "Release", NoRestore: true), deadline.Token);
        Assert.Empty(release.Issues); Assert.Equal(2, Assert.Single(release.Projects).Request.Sources.Count);
        Assert.NotEqual(debug.SnapshotId, release.SnapshotId);
    }

    [Fact]
    public async Task New_globbed_source_invalidates_the_captured_context()
    {
        using var project = sdk.CreateProject();
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        var context = await WorkspaceLoader.LoadAsync(new(project.File("Game/Game.fpasproj")), deadline.Token);
        Assert.Empty(context.Issues);
        File.WriteAllText(project.File("Game/Added.fpas"), "unit Added; end unit;");
        var failure = await Assert.ThrowsAsync<ContextFailure>(() => context.VerifyInputsAsync(deadline.Token));
        Assert.Equal("inputs_changed", failure.Code);
    }

    [Fact]
    public async Task Solution_mapping_can_check_Debug_FPAs_against_a_Release_dependency()
    {
        using var project = sdk.CreateProject();
        var path = project.File("Game.slnx");
        var solution = XDocument.Load(path);
        solution.Descendants("Project").Single(element => (string?)element.Attribute("Path") == "Game/Game.fpasproj")
            .Add(new XElement("BuildType", new XAttribute("Solution", "Release|*"), new XAttribute("Project", "Debug")));
        solution.Save(path);
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        var context = await WorkspaceLoader.LoadAsync(new(path, "Release"), deadline.Token);
        Assert.Empty(context.Issues);
        var input = Assert.Single(context.Projects);
        Assert.Equal("Debug", input.Configuration);
        Assert.Contains(input.Request.References, reference => reference == project.File("Shared/bin/Release/net10.0/Shared.dll"));
        Assert.True(FpasCompiler.Analyze(input.Request, deadline.Token).Success);
    }
}
