using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using Fpas.Workspaces.Documentation;
using System.Text.Json;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.ApiDocumentation;

public sealed class DocumentationScopeTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Project_selection_uses_its_actual_references_and_shared_FPAS_source_is_not_duplicated_as_metadata()
    {
        using var directory = DocumentationFixture.Create(sdk);
        Directory.CreateDirectory(directory.File("Library"));
        File.WriteAllText(directory.File("Library/Library.fpasproj"), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Library</OutputType><GenerateDocumentationFile>true</GenerateDocumentationFile></PropertyGroup></Project>
            """);
        File.WriteAllText(directory.File("Library/Math.fpas"), """
            unit Extra.Math;
            // Adds the given values.
            public function Sum(Left: int; Right: int): int; begin return Left + Right; end function;
            end unit;
            """);
        var project = XDocument.Load(directory.File("Game/Game.fpasproj"));
        project.Root!.Add(new XElement("ItemGroup", new XElement("ProjectReference", new XAttribute("Include", "../Library/Library.fpasproj")))); project.Save(directory.File("Game/Game.fpasproj"));
        var analysis = await DocumentationFixture.Analyze(directory); var query = new DocumentationQuery(analysis);
        var all = query.Search(new("Extra.Math.Sum"));
        Assert.Equal("source", Assert.Single(all.Entries).Origin);
        var narrowed = query.Search(new("Extra.Math.Sum", Project: directory.File("Game/Game.fpasproj")));
        Assert.Equal("metadata", Assert.Single(narrowed.Entries).Origin); Assert.Equal("available", narrowed.Entries[0].Documentation.Availability);
        Assert.Throws<Fpas.Workspaces.Loading.ContextFailure>(() => query.Search(new("Sum", Project: directory.File("Shared/Shared.csproj"))));
        var solution = await DocumentationFixture.Cli(directory, directory.File("Game.slnx"), "--name", "Extra.Math.Sum", "--report", "json");
        Assert.True(solution.ExitCode == 0, solution.Output + solution.Error);
        using var report = JsonDocument.Parse(solution.Output);
        Assert.Equal("source", Assert.Single(report.RootElement.GetProperty("result").GetProperty("entries").EnumerateArray()).GetProperty("origin").GetString());
    }

    [Fact]
    public async Task Fresh_packaged_CLI_uses_the_same_documentation_command_and_machine_readable_contract()
    {
        using var directory = DocumentationFixture.Create(sdk);
        await DocumentationFixture.Analyze(directory);
        var project = directory.File("Game/Game.fpasproj");
        var location = await sdk.Run(directory, "msbuild", project, "-nologo", "-getProperty:NuGetPackageRoot");
        Assert.Equal(0, location.ExitCode);
        var cli = Path.Combine(location.Output.Trim(), "fpas.net.sdk", sdk.Version.ToLowerInvariant(), "tools", "compiler", "Fpas.Cli.dll");
        var result = await ProcessRunner.RunAsync([cli, "doc", project, "--name", "Demo.Bridge.Message", "--no-restore", "--report", "json"], workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(result.ExitCode == 0, result.Output + result.Error); Assert.Empty(result.Error);
        using var report = JsonDocument.Parse(result.Output);
        Assert.Equal("fpas", Assert.Single(report.RootElement.GetProperty("result").GetProperty("entries").EnumerateArray()).GetProperty("signatureFormat").GetString());
    }
}
