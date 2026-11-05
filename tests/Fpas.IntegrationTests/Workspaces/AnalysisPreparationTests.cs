using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using Fpas.Workspaces.Loading;
using System.Diagnostics;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class AnalysisPreparationTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Shared_preparation_generates_sources_and_exports_linked_files_and_custom_names()
    {
        using var project = sdk.CreateProject();
        Directory.CreateDirectory(project.File("external inputs"));
        File.WriteAllText(project.File("external inputs/Grüße.fpas"), "unit Linked; end unit;");
        File.WriteAllText(project.File("Game/Game.fpasproj"), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup><OutputType>Library</OutputType><AssemblyName>CustomGame</AssemblyName></PropertyGroup>
              <ItemGroup>
                <ProjectReference Include="../Shared/Shared.csproj" />
                <Compile Include="../external inputs/Grüße.fpas" />
                <Compile Include="$(IntermediateOutputPath)Generated.fpas" />
              </ItemGroup>
              <Target Name="GenerateSource" BeforeTargets="FpasPrepareCompilation">
                <WriteLinesToFile File="$(IntermediateOutputPath)Generated.fpas" Lines="unit Generated%3B end unit%3B" Overwrite="true" WriteOnlyWhenDifferent="true" />
              </Target>
            </Project>
            """);
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        var context = await WorkspaceLoader.LoadAsync(new(project.File("Game/Game.fpasproj")), deadline.Token);
        Assert.Empty(context.Issues);
        var input = Assert.Single(context.Projects);
        Assert.Equal("CustomGame", input.Request.AssemblyName);
        Assert.Contains(input.Request.Sources, source => source.Path == project.File("external inputs/Grüße.fpas"));
        Assert.Contains(input.Request.Sources, source => source.Path.EndsWith("Generated.fpas", StringComparison.Ordinal));
        Assert.True(Fpas.Compiler.FpasCompiler.Analyze(input.Request, deadline.Token).Success);
        Assert.False(File.Exists(project.File("Game/bin/Debug/net10.0/CustomGame.dll")));
        await context.VerifyInputsAsync(deadline.Token);
    }

    [Fact]
    public async Task No_restore_and_preparation_failures_are_explicit_instead_of_empty_success()
    {
        using var project = sdk.CreateProject();
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        var context = await WorkspaceLoader.LoadAsync(new(project.File("Game/Game.fpasproj"), NoRestore: true), deadline.Token);
        Assert.False(context.RestorePerformed);
        Assert.Empty(context.Projects);
        Assert.Contains(context.Issues, issue => issue.Code == "context_failed" && issue.Message.Contains("NETSDK1004", StringComparison.Ordinal));
        Assert.False(File.Exists(project.File("Game/obj/project.assets.json")));
    }

    [Fact]
    public async Task Explicit_transitive_preparation_dependencies_run_for_both_analysis_and_build()
    {
        using var project = sdk.CreateProject();
        var path = project.File("Game/Game.fpasproj");
        File.WriteAllText(path, File.ReadAllText(path).Replace("</Project>", """
              <PropertyGroup><FpasPrepareCompilationDependsOn>SharedPreparation</FpasPrepareCompilationDependsOn></PropertyGroup>
              <ItemGroup><Compile Include="$(IntermediateOutputPath)Generated.fpas" /></ItemGroup>
              <Target Name="SharedPreparation" DependsOnTargets="GenerateSource" />
              <Target Name="GenerateSource" BeforeTargets="CoreCompile">
                <MakeDir Directories="$(IntermediateOutputPath)" />
                <WriteLinesToFile File="$(IntermediateOutputPath)Generated.fpas" Lines="unit Generated%3B end unit%3B" Overwrite="true" WriteOnlyWhenDifferent="true" />
              </Target>
            </Project>
            """, StringComparison.Ordinal));
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        var context = await WorkspaceLoader.LoadAsync(new(path), deadline.Token);
        Assert.Empty(context.Issues);
        Assert.Contains(Assert.Single(context.Projects).Request.Sources, source => source.Path.EndsWith("Generated.fpas", StringComparison.Ordinal));
        var build = await sdk.Run(project, "build", "Game/Game.fpasproj", "--no-restore");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
    }

    [Theory]
    [InlineData("<Target Name=\"Unsupported\" BeforeTargets=\"CoreCompile\" />", "FpasPrepareCompilation")]
    [InlineData("<Target Name=\"Unsupported\" BeforeTargets=\"corecompile\" />", "FpasPrepareCompilation")]
    [InlineData("<PropertyGroup><CustomHook>CoreCompile</CustomHook></PropertyGroup><Target Name=\"Unsupported\" BeforeTargets=\"$(CustomHook)\" />", "FpasPrepareCompilation")]
    [InlineData("<ItemGroup><ProjectReference Update=\"../Shared/Shared.csproj\" AdditionalProperties=\"Custom=true\" /></ItemGroup>", "AdditionalProperties")]
    public async Task Unsupported_preparation_is_diagnosed(string content, string reason)
    {
        using var project = sdk.CreateProject();
        var path = project.File("Game/Game.fpasproj");
        File.WriteAllText(path, File.ReadAllText(path).Replace("</Project>", content + "</Project>", StringComparison.Ordinal));
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        var context = await WorkspaceLoader.LoadAsync(new(path), deadline.Token);
        Assert.Empty(context.Projects);
        Assert.Contains(context.Issues, issue => issue.Message.Contains(reason, StringComparison.Ordinal));
    }

    [Fact]
    public async Task Cancellation_terminates_and_joins_a_running_MSBuild_task()
    {
        using var project = sdk.CreateProject();
        using var helper = new TemporaryDirectory();
        var source = helper.File("Wait.fpas"); var assembly = helper.File("Wait.dll");
        File.WriteAllText(source, """
            program Wait;
            uses System as S, System.IO.File as File, System.Threading.Thread as Thread;
            begin
              const Ready: string := S.Environment.GetCommandLineArgs()[1];
              File.WriteAllText(Ready + '.tmp', S.Environment.ProcessId.ToString());
              File.Move(Ready + '.tmp', Ready);
              while true do Thread.Sleep(20); end while;
            end;
            end program;
            """);
        var compiled = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", source,
            "--output", assembly, "--reference-dir", TestEnvironment.ReferenceDirectory]);
        Assert.True(compiled.ExitCode == 0, compiled.Output + compiled.Error);
        var ready = helper.File("ready.pid");
        var path = project.File("Game/Game.fpasproj");
        var xml = System.Xml.Linq.XDocument.Load(path);
        xml.Root!.Add(new System.Xml.Linq.XElement("Target", new System.Xml.Linq.XAttribute("Name", "WaitForCancellation"),
            new System.Xml.Linq.XAttribute("BeforeTargets", "FpasPrepareCompilation"),
            new System.Xml.Linq.XElement("Exec", new System.Xml.Linq.XAttribute("Command", $"\"{TestEnvironment.DotnetHost}\" \"{assembly}\" \"{ready}\""))));
        xml.Save(path);
        using var cancellation = new CancellationTokenSource();
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        using var combined = CancellationTokenSource.CreateLinkedTokenSource(cancellation.Token, deadline.Token);
        var load = WorkspaceLoader.LoadAsync(new(path), combined.Token);
        try
        {
            while (!File.Exists(ready))
            {
                if (load.IsCompleted) Assert.Fail("Preparation ended before its owned task became ready: " + string.Join('\n', (await load).Issues));
                await Task.Delay(20, deadline.Token);
            }
            var pid = int.Parse(await File.ReadAllTextAsync(ready, deadline.Token), System.Globalization.CultureInfo.InvariantCulture);
            using var child = Process.GetProcessById(pid);
            cancellation.Cancel();
            await Assert.ThrowsAnyAsync<OperationCanceledException>(() => load);
            Assert.True(child.HasExited);
        }
        finally
        {
            cancellation.Cancel();
            try { await load; } catch (OperationCanceledException) { }
        }
    }
}
