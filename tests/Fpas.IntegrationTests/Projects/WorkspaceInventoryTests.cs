// [Documentation](../../../docs/pascal/program-structure/project-commands.md#inspect-the-workspace)
using Fpas.Tests.Support;
using System.Text.Json;

namespace Fpas.IntegrationTests.Projects;

public sealed class WorkspaceInventoryTests
{
    [Fact]
    public async Task Reports_mapped_roles_and_references_without_running_targets_or_restoring()
    {
        using var directory = new TemporaryDirectory();
        File.WriteAllText(directory.File("App.csproj"), """
            <Project Sdk="Microsoft.NET.Sdk" InitialTargets="Marker">
              <PropertyGroup>
                <TargetFrameworks>net10.0;net9.0</TargetFrameworks>
                <OutputType>Exe</OutputType>
              </PropertyGroup>
              <ItemGroup Condition="'$(Configuration)' == 'Debug' and '$(TargetFramework)' == 'net10.0'">
                <ProjectReference Include="Library.fsproj" />
              </ItemGroup>
              <Target Name="Marker"><Touch Files="target.marker" AlwaysCreate="true" /></Target>
            </Project>
            """);
        File.WriteAllText(directory.File("Library.fsproj"), """
            <Project Sdk="Microsoft.NET.Sdk">
              <PropertyGroup><TargetFramework>net10.0</TargetFramework><IsTestProject>true</IsTestProject></PropertyGroup>
            </Project>
            """);
        File.WriteAllText(directory.File("Example.slnx"), """
            <Solution>
              <Configurations><BuildType Name="Release" /><Platform Name="Any CPU" /></Configurations>
              <Project Path="App.csproj"><BuildType Solution="Release|*" Project="Debug" /></Project>
              <Project Path="Library.fsproj"><Build Solution="Release|*" Project="false" /></Project>
              <Project Path="Missing.csproj" />
            </Solution>
            """);
        var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "workspace", "Example.slnx", "--configuration", "Release"],
            workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(1));
        Assert.Equal(1, result.ExitCode);
        Assert.Empty(result.Error);
        using var json = JsonDocument.Parse(result.Output);
        var report = json.RootElement;
        Assert.Equal(1, report.GetProperty("schemaVersion").GetInt32());
        Assert.Equal("partial", report.GetProperty("status").GetString());
        var projects = report.GetProperty("projects").EnumerateArray().ToDictionary(project => project.GetProperty("project").GetString()!);
        Assert.Equal(3, projects.Count);
        var app = projects[directory.File("App.csproj")];
        Assert.Equal("Debug", app.GetProperty("configuration").GetString());
        Assert.True(app.GetProperty("buildEnabled").GetBoolean());
        var frameworks = app.GetProperty("frameworks").EnumerateArray().ToDictionary(framework => framework.GetProperty("framework").GetString()!);
        Assert.Equal("executable", frameworks["net10.0"].GetProperty("role").GetString());
        Assert.Equal(directory.File("Library.fsproj"), Assert.Single(frameworks["net10.0"].GetProperty("references").EnumerateArray()).GetString());
        Assert.Empty(frameworks["net9.0"].GetProperty("references").EnumerateArray());
        var library = projects[directory.File("Library.fsproj")];
        Assert.False(library.GetProperty("buildEnabled").GetBoolean());
        Assert.Equal("test", Assert.Single(library.GetProperty("frameworks").EnumerateArray()).GetProperty("role").GetString());
        var issue = projects[directory.File("Missing.csproj")].GetProperty("issue");
        Assert.Equal(directory.File("Missing.csproj"), issue.GetProperty("inputPath").GetString());
        Assert.False(File.Exists(directory.File("target.marker")));
        Assert.False(Directory.Exists(directory.File("obj")));
    }
    [Fact]
    public async Task File_filter_matches_conditional_and_linked_compile_items()
    {
        using var directory = new TemporaryDirectory();
        foreach (var folder in new[] { "App", "Tools", "Other" }) Directory.CreateDirectory(directory.File(folder));
        File.WriteAllText(directory.File("Shared.cs"), "public sealed class Shared {}");
        File.WriteAllText(directory.File("App/App.csproj"), """
            <Project Sdk="Microsoft.NET.Sdk">
              <PropertyGroup><TargetFrameworks>net10.0;net9.0</TargetFrameworks></PropertyGroup>
              <ItemGroup Condition="'$(TargetFramework)' == 'net10.0' and '$(Configuration)' == 'Debug'">
                <Compile Include="../Shared.cs" Link="Shared.cs" />
              </ItemGroup>
            </Project>
            """);
        File.WriteAllText(directory.File("Tools/Tools.csproj"), """
            <Project Sdk="Microsoft.NET.Sdk">
              <PropertyGroup><TargetFramework>net10.0</TargetFramework></PropertyGroup>
              <ItemGroup><Compile Include="../Shared.cs" Link="Linked/Shared.cs" /></ItemGroup>
            </Project>
            """);
        File.WriteAllText(directory.File("Other/Other.csproj"), """
            <Project Sdk="Microsoft.NET.Sdk">
              <PropertyGroup><TargetFramework>net10.0</TargetFramework></PropertyGroup>
              <ItemGroup><Compile Include="../Shared.cs" /><Compile Remove="../Shared.cs" /></ItemGroup>
            </Project>
            """);
        File.WriteAllText(directory.File("Files.slnx"), """
            <Solution>
              <Configurations><BuildType Name="Release" /><Platform Name="Any CPU" /></Configurations>
              <Project Path="App/App.csproj"><BuildType Solution="Release|*" Project="Debug" /></Project>
              <Project Path="Tools/Tools.csproj"><Build Solution="Release|*" Project="false" /></Project>
              <Project Path="Other/Other.csproj" />
            </Solution>
            """);
        var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "workspace", "Files.slnx", "--configuration", "Release", "--file", "./Shared.cs"],
            workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(1));
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        using var json = JsonDocument.Parse(result.Output);
        var report = json.RootElement;
        Assert.Equal(directory.File("Shared.cs"), report.GetProperty("file").GetString());
        var owners = report.GetProperty("projects").EnumerateArray().ToArray();
        Assert.Equal(new[] { directory.File("App/App.csproj"), directory.File("Tools/Tools.csproj") }, owners.Select(owner => owner.GetProperty("project").GetString()));
        Assert.Equal("Debug", owners[0].GetProperty("configuration").GetString());
        Assert.Equal("net10.0", Assert.Single(owners[0].GetProperty("frameworks").EnumerateArray()).GetProperty("framework").GetString());
        Assert.False(owners[1].GetProperty("buildEnabled").GetBoolean());
        Assert.False(Directory.Exists(directory.File("App/obj")));
    }

    [Fact]
    public async Task Affected_sources_include_linked_owners_and_transitive_tests_and_report_incomplete_graphs()
    {
        using var directory = new TemporaryDirectory();
        File.WriteAllText(directory.File("Shared.cs"), "public sealed class Shared {}");
        WriteProject("Root", "<Compile Include=\"../Shared.cs\" Link=\"Shared.cs\" />");
        WriteProject("Linked", "<Compile Include=\"../Shared.cs\" Link=\"Shared.cs\" />");
        WriteProject("App", "<ProjectReference Include=\"../Root/Root.csproj\" />");
        WriteProject("Tests", "<ProjectReference Include=\"../App/App.csproj\" />", test: true);
        WriteProject("Unrelated", "", test: true);
        File.WriteAllText(directory.File("All.slnx"), """
            <Solution><Project Path="Root/Root.csproj" /><Project Path="Linked/Linked.csproj" />
              <Project Path="App/App.csproj" /><Project Path="Tests/Tests.csproj" /><Project Path="Unrelated/Unrelated.csproj" /></Solution>
            """);
        using (var document = await Read(0))
        {
            var root = document.RootElement;
            var impact = root.GetProperty("impact");
            Assert.True(impact.GetProperty("complete").GetBoolean());
            var sources = impact.GetProperty("sources").EnumerateArray().ToDictionary(item => item.GetProperty("file").GetString()!);
            Assert.Equal(new[] { directory.File("Linked/Linked.csproj"), directory.File("Root/Root.csproj") }, Paths(sources[directory.File("Shared.cs")], "owners"));
            Assert.Empty(Paths(sources[directory.File("Unknown.cs")], "owners"));
            Assert.Equal(new[] { directory.File("App/App.csproj"), directory.File("Tests/Tests.csproj") }, Paths(impact, "dependents"));
            Assert.Equal(directory.File("Tests/Tests.csproj"), Assert.Single(Paths(impact, "tests")));
            Assert.Equal(4, root.GetProperty("projects").GetArrayLength());
        }
        WriteProject("Unrelated", "<ProjectReference Include=\"../External/External.csproj\" />", test: true);
        File.Delete(directory.File("Linked/Linked.csproj"));
        using var partial = await Read(1);
        Assert.Equal("partial", partial.RootElement.GetProperty("status").GetString());
        Assert.False(partial.RootElement.GetProperty("impact").GetProperty("complete").GetBoolean());
        Assert.Equal(directory.File("External/External.csproj"), Assert.Single(Paths(partial.RootElement.GetProperty("impact"), "externalReferences")));
        Assert.Contains(partial.RootElement.GetProperty("projects").EnumerateArray(), item => item.TryGetProperty("issue", out _));
        Assert.False(Directory.Exists(directory.File("App/obj")));

        void WriteProject(string name, string items, bool test = false)
        {
            Directory.CreateDirectory(directory.File(name));
            File.WriteAllText(directory.File($"{name}/{name}.csproj"), $"""
                <Project Sdk="Microsoft.NET.Sdk">
                  <PropertyGroup><TargetFramework>net10.0</TargetFramework><IsTestProject>{test.ToString().ToLowerInvariant()}</IsTestProject></PropertyGroup>
                  <ItemGroup>{items}</ItemGroup>
                </Project>
                """);
        }
        async Task<JsonDocument> Read(int exit)
        {
            var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "workspace", "All.slnx", "--affected", "Shared.cs", "--affected", "Unknown.cs"],
                workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(1));
            Assert.True(result.ExitCode == exit, result.Output + result.Error);
            return JsonDocument.Parse(result.Output);
        }
        static IEnumerable<string?> Paths(JsonElement parent, string property) => parent.GetProperty(property).EnumerateArray().Select(value => value.GetString());
    }
}
