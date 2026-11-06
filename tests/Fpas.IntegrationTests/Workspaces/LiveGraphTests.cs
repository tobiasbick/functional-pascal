using Fpas.IntegrationTests.Build;
using Fpas.Workspaces.Live;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class LiveGraphTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task Physical_text_updates_preserve_linked_owners_and_reuse_independent_compilations(bool linked)
    {
        using var directory = sdk.CreateProject(); var path = directory.File("Game/Bridge.fpas");
        const string source = "unit Demo.Bridge; public function Read(): int; begin return 42; end function; end unit;";
        File.WriteAllText(path, source); Directory.CreateDirectory(directory.File("Other"));
        var otherProject = directory.File("Other/Other.fpasproj");
        File.WriteAllText(otherProject, $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Library</OutputType><EnableDefaultFpasCompileItems>{(!linked).ToString().ToLowerInvariant()}</EnableDefaultFpasCompileItems></PropertyGroup>" +
            (linked ? "<ItemGroup><Compile Include=\"../Game/Bridge.fpas\" /></ItemGroup>" : "") + "</Project>");
        if (!linked) File.WriteAllText(directory.File("Other/Other.fpas"), "unit Other; public function Independent(): int; begin return 1; end function; end unit;");
        foreach (var project in new[] { "Game/Game.fpasproj", "Other/Other.fpasproj" })
        {
            var build = await sdk.Run(directory, "build", project, "--disable-build-servers"); Assert.True(build.ExitCode == 0, build.Output + build.Error);
        }
        var solution = directory.File("Live.slnx"); var xml = XDocument.Load(directory.File("Game.slnx"));
        xml.Root!.Elements("Project").Remove();
        xml.Root.Add(new XElement("Project", new XAttribute("Path", "Game/Game.fpasproj")), new XElement("Project", new XAttribute("Path", "Other/Other.fpasproj"))); xml.Save(solution);
        await using var workspace = new LiveWorkspace(new(solution));
        var first = await workspace.AnalyzeAsync(); Assert.True(first.Analysis.Success);
        var revised = source.Replace("Read", "Fetch", StringComparison.Ordinal); workspace.Open(path, revised, 5);
        var second = await workspace.AnalyzeAsync(); Assert.True(second.Analysis.Success);
        if (!linked)
        {
            Assert.Same(first.Analysis.Projects.Single(project => project.Project == otherProject).Result, second.Analysis.Projects.Single(project => project.Project == otherProject).Result);
            Assert.NotSame(first.Analysis.Projects.Single(project => project.Project != otherProject).Result, second.Analysis.Projects.Single(project => project.Project != otherProject).Result);
        }
        else
        {
            var offset = revised.IndexOf("Fetch", StringComparison.Ordinal); var selection = new SourceSelection(path, 1, offset + 1);
            Assert.Equal("ambiguous_project", Assert.Throws<ContextFailure>(() => second.Queries().Execute("definition", selection)).Code);
            var rename = second.Rename(selection with { Project = directory.File("Game/Game.fpasproj") }, "ReadAgain");
            Assert.Equal(2, rename.Plan.Targets.Count); Assert.Equal(5, Assert.Single(rename.Edits.Documents).Original.Version);
        }
        await workspace.VerifyAsync(second); Assert.Equal(source, File.ReadAllText(path));
    }
}
