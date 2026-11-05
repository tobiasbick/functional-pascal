using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Projects;
using Fpas.Tests.Support;
using System.Text.Json;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Testing;

public sealed class TestSolutionTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Mixed_solutions_skip_non_tests_honor_mappings_and_allow_explicit_selection()
    {
        using var directory = new TemporaryDirectory();
        var tests = await TestProjectFixture.Create(directory, sdk, "xunit");
        var testProject = XDocument.Load(tests);
        testProject.Root!.Add(new XElement("PropertyGroup", new XAttribute("Condition", "'$(BuildingSolutionFile)' == 'true'"),
            new XElement("OutputPath", "$(SolutionDir)artifacts/$(Configuration)/")));
        testProject.Save(tests);
        var app = await ProjectCliFixture.Init(directory, sdk, "App");
        var created = await ProjectCliFixture.Cli(directory, "init", "solution", "Solution", "--project", tests, "--project", app);
        Assert.True(created.ExitCode == 0, created.Output + created.Error);
        var path = directory.File("Solution/Solution.slnx");
        var xml = XDocument.Load(path); xml.Root!.Elements("Project").Single(element => ((string)element.Attribute("Path")!).Contains("Tests", StringComparison.Ordinal))
            .Add(new XElement("BuildType", new XAttribute("Solution", "Release|*"), new XAttribute("Project", "Debug")));
        xml.Save(path);
        var run = await ProjectCliFixture.Cli(directory, "test", path, "--configuration", "Release", "--report", "json");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        using (var report = JsonDocument.Parse(run.Output))
        {
            var selectedProject = Assert.Single(report.RootElement.GetProperty("projects").EnumerateArray()).GetProperty("project");
            Assert.Equal("Debug", selectedProject.GetProperty("configuration").GetString());
            Assert.Contains("artifacts", selectedProject.GetProperty("assembly").GetString());
        }
        var selected = await ProjectCliFixture.Cli(directory, "test", path, "--configuration", "Release", "--project", tests, "--list", "--no-build", "--report", "json");
        Assert.True(selected.ExitCode == 0, selected.Output + selected.Error);
        var outside = await ProjectCliFixture.Cli(directory, "test", path, "--project", directory.File("outside.fpasproj"), "--no-build", "--report", "json");
        Assert.Equal(2, outside.ExitCode);
    }

    [Fact]
    public async Task A_CSharp_test_project_and_an_FPAS_test_project_share_one_native_solution_run()
    {
        using var directory = new TemporaryDirectory();
        var tests = await TestProjectFixture.Create(directory, sdk, "xunit");
        var foreign = directory.File("Foreign/Foreign.csproj"); Directory.CreateDirectory(Path.GetDirectoryName(foreign)!);
        var xml = XDocument.Load(tests); xml.Root!.SetAttributeValue("Sdk", "Microsoft.NET.Sdk"); xml.Save(foreign);
        File.Copy(directory.File("Tests/NuGet.Config"), directory.File("Foreign/NuGet.Config"));
        File.WriteAllText(directory.File("Foreign/Tests.cs"), "public class Tests { [Xunit.Fact] public void Native() => Xunit.Assert.Equal(4, 2 + 2); }");
        var created = await ProjectCliFixture.Cli(directory, "init", "solution", "Mixed", "--project", tests, "--project", foreign);
        Assert.Equal(0, created.ExitCode);
        var run = await ProjectCliFixture.Cli(directory, "test", directory.File("Mixed/Mixed.slnx"), "--report", "json");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        using var report = JsonDocument.Parse(run.Output);
        var projects = report.RootElement.GetProperty("projects").EnumerateArray().ToArray();
        Assert.Equal(2, projects.Length);
        Assert.All(projects, project => Assert.Equal(1, project.GetProperty("result").GetProperty("counts").GetProperty("passed").GetInt32()));
    }
}
