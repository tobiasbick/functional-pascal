using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Projects;
using Fpas.Tests.Support;
using System.Text.Json;

namespace Fpas.IntegrationTests.Formatting;

public sealed class FormatProjectTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Evaluated_Compile_items_honor_configuration_links_exclusions_and_solution_members()
    {
        using var directory = new TemporaryDirectory();
        var project = await ProjectCliFixture.Init(directory, sdk);
        var source = directory.File("App/main.fpas");
        File.WriteAllText(source, "program App;begin end;end program;");
        File.WriteAllText(directory.File("App/ignored.fpas"), "intentionally invalid excluded source");
        File.WriteAllText(directory.File("App/release.fpas"), "unit Extra;end unit;");
        File.WriteAllText(directory.File("linked.fpas"), "unit Linked;end unit;");
        File.WriteAllText(project, File.ReadAllText(project).Replace("</Project>", """
              <ItemGroup>
                <Compile Remove="ignored.fpas;release.fpas" />
                <Compile Include="release.fpas" Condition="'$(Configuration)' == 'Release'" />
                <Compile Include="../linked.fpas" />
              </ItemGroup>
            </Project>
            """, StringComparison.Ordinal));
        var direct = await FormatCliTests.Run(directory, project, "--check", "--report", "json");
        Assert.Equal(2, direct.ExitCode); Assert.Empty(direct.Error);
        using (var report = JsonDocument.Parse(direct.Output))
            Assert.Equal(new[] { source, directory.File("linked.fpas") }, report.RootElement.GetProperty("files").EnumerateArray().Select(file => file.GetProperty("path").GetString()));
        File.WriteAllText(directory.File("Host.csproj"), "This C# host must not be evaluated by the FPAS formatter.");
        var solution = await ProjectCliFixture.Cli(directory, "init", "solution", "Solution", "--project", project, "--project", directory.File("Host.csproj"));
        Assert.True(solution.ExitCode == 0, solution.Output + solution.Error);
        var release = await FormatCliTests.Run(directory, directory.File("Solution/Solution.slnx"), "--configuration", "Release", "--check", "--report", "json");
        Assert.Equal(2, release.ExitCode); Assert.Empty(release.Error);
        using (var report = JsonDocument.Parse(release.Output)) Assert.Equal(3, report.RootElement.GetProperty("files").GetArrayLength());
        Assert.False(File.Exists(directory.File("App/obj/project.assets.json")));
        Assert.False(Directory.Exists(directory.File("App/bin")));
        Assert.Equal("intentionally invalid excluded source", File.ReadAllText(directory.File("App/ignored.fpas")));
    }
}
