using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using System.Text.Json;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Projects;

public sealed class BuildRunTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Build_and_run_a_mixed_solution_selects_the_CSharp_host_and_honors_configuration()
    {
        using var project = sdk.CreateProject();
        var initialized = await ProjectCliFixture.Cli(project, "init", "solution", "Solution", "--name", "Demo", "--project", "Game/Game.fpasproj", "--project", "Host/Host.csproj", "--report", "json");
        Assert.Equal(0, initialized.ExitCode);
        var build = await ProjectCliFixture.Cli(project, "build", "Solution/Demo.slnx", "--configuration", "Release", "--report", "json");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var result = await ProjectCliFixture.Cli(project, "run", "Solution/Demo.slnx", "--configuration", "Release", "--no-build", "--report", "json");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.Equal(project.File("Host/Host.csproj"), json.RootElement.GetProperty("target").GetProperty("project").GetString());
        Assert.Contains("Release", json.RootElement.GetProperty("target").GetProperty("assembly").GetString());
        Assert.Equal("C#:FPAS" + Environment.NewLine, json.RootElement.GetProperty("program").GetProperty("output").GetString());
    }

    [Fact]
    public async Task Ambiguity_requires_an_included_executable_and_does_not_build_before_selection()
    {
        using var directory = new TemporaryDirectory();
        var first = await ProjectCliFixture.Init(directory, sdk, "First");
        var second = await ProjectCliFixture.Init(directory, sdk, "Second");
        var init = await ProjectCliFixture.Cli(directory, "init", "solution", ".", "--name", "Demo", "--project", first, "--project", second);
        Assert.Equal(0, init.ExitCode);
        var ambiguous = await ProjectCliFixture.Cli(directory, "run", "Demo.slnx", "--report", "json");
        Assert.Equal(2, ambiguous.ExitCode);
        using var json = JsonDocument.Parse(ambiguous.Output);
        Assert.Equal("ambiguous_project", json.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.Equal(2, json.RootElement.GetProperty("error").GetProperty("candidates").GetArrayLength());
        Assert.False(Directory.Exists(directory.File("First/bin")));
        var selected = await ProjectCliFixture.Cli(directory, "run", "Demo.slnx", "--project", first, "--report", "json");
        Assert.True(selected.ExitCode == 0, selected.Output + selected.Error);
        var outside = await ProjectCliFixture.Cli(directory, "run", "Demo.slnx", "--project", "Other.fpasproj", "--report", "json");
        Assert.Equal(2, outside.ExitCode);
    }

    [Fact]
    public async Task Program_arguments_stdin_working_directory_stderr_and_exit_code_survive_both_stream_modes()
    {
        using var directory = new TemporaryDirectory();
        var project = await ProjectCliFixture.Init(directory, sdk);
        File.WriteAllText(directory.File("App/main.fpas"), """
            program IO;
            uses System.Console as Console, System.Environment as Env, System.IO.File as File;
            begin
              Console.WriteLine(args[0]);
              Console.WriteLine(Console.ReadLine() ?? 'EOF');
              Console.WriteLine(File.ReadAllText('data.txt'));
              Console.Error.WriteLine('diagnostic');
              Env.ExitCode := 7;
            end;
            end program;
            """);
        var build = await ProjectCliFixture.Cli(directory, "build", project);
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        Directory.CreateDirectory(directory.File("working Grüße")); File.WriteAllText(directory.File("working Grüße/data.txt"), "file value");
        foreach (var json in new[] { false, true })
        {
            var options = new List<string> { TestEnvironment.Metadata("FpasCliPath"), "run", project, "--no-build", "--working-directory", directory.File("working Grüße") };
            if (json) options.AddRange(["--report", "json"]);
            options.AddRange(["--", "a b; $(no-shell)"]);
            var result = await ProcessRunner.RunAsync(options, "input value\n", directory.Path, timeLimit: TimeSpan.FromMinutes(2));
            Assert.Equal(7, result.ExitCode);
            var expected = "a b; $(no-shell)" + Environment.NewLine + "input value" + Environment.NewLine + "file value" + Environment.NewLine;
            if (!json) { Assert.Equal(expected, result.Output); Assert.Equal("diagnostic" + Environment.NewLine, result.Error); }
            else
            {
                Assert.Equal("", result.Error); using var report = JsonDocument.Parse(result.Output);
                Assert.Equal(7, report.RootElement.GetProperty("program").GetProperty("exitCode").GetInt32());
                Assert.Equal(expected, report.RootElement.GetProperty("program").GetProperty("output").GetString());
                Assert.Equal("diagnostic" + Environment.NewLine, report.RootElement.GetProperty("program").GetProperty("error").GetString());
            }
        }
    }

    [Fact]
    public async Task Build_failure_cannot_run_previous_outputs_and_no_restore_or_no_build_fail_explicitly()
    {
        using var directory = new TemporaryDirectory();
        var project = await ProjectCliFixture.Init(directory, sdk);
        var noRestore = await ProjectCliFixture.Cli(directory, "build", project, "--no-restore", "--report", "json");
        Assert.Equal(1, noRestore.ExitCode);
        var noBuild = await ProjectCliFixture.Cli(directory, "run", project, "--no-build", "--report", "json");
        Assert.Equal(2, noBuild.ExitCode);
        var built = await ProjectCliFixture.Cli(directory, "build", project, "--report", "json"); Assert.Equal(0, built.ExitCode);
        File.WriteAllText(directory.File("App/main.fpas"), "program App; begin const Broken := 1; end; end program;");
        var run = await ProjectCliFixture.Cli(directory, "run", project, "--report", "json");
        Assert.Equal(1, run.ExitCode); using var result = JsonDocument.Parse(run.Output);
        Assert.Equal("build_failed", result.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.False(result.RootElement.TryGetProperty("program", out _));
        Assert.True(File.Exists(directory.File("App/bin/Debug/net10.0/App.dll")));
    }

    [Fact]
    public async Task Solution_start_selection_uses_the_mapped_project_configuration()
    {
        using var directory = sdk.CreateProject();
        var solution = XDocument.Load(directory.File("Game.slnx"));
        solution.Descendants("Project").Single(element => (string?)element.Attribute("Path") == "Host/Host.csproj")
            .Add(new XElement("BuildType", new XAttribute("Solution", "Release|*"), new XAttribute("Project", "Debug")));
        solution.Save(directory.File("Game.slnx"));
        var result = await ProjectCliFixture.Cli(directory, "run", "Game.slnx", "--configuration", "Release", "--report", "json");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.Equal("Debug", json.RootElement.GetProperty("target").GetProperty("configuration").GetString());
        Assert.Equal(directory.File("Host/bin/Debug/net10.0/Host.dll"), json.RootElement.GetProperty("target").GetProperty("assembly").GetString());
    }

    [Fact]
    public async Task A_mixed_solution_can_select_and_run_an_FSharp_host()
    {
        using var directory = sdk.CreateProject();
        var props = XDocument.Load(directory.File("Directory.Build.props"));
        props.Root!.Element("PropertyGroup")!.Add(new XElement("RestoreAdditionalProjectSources", "$(MSBuildSDKsPath)/../FSharp/library-packs"), new XElement("NuGetAudit", "false"));
        props.Save(directory.File("Directory.Build.props"));
        Directory.CreateDirectory(directory.File("FSharp"));
        File.WriteAllText(directory.File("FSharp/Host.fsproj"), """
            <Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType></PropertyGroup>
              <ItemGroup><Compile Include="Program.fs" /><ProjectReference Include="../Game/Game.fpasproj" /></ItemGroup></Project>
            """);
        File.WriteAllText(directory.File("FSharp/Program.fs"), "printfn \"%s\" (Demo.Bridge.Message(\"F#\"))\n");
        var solution = XDocument.Load(directory.File("Game.slnx"));
        solution.Descendants("Project").Single(element => (string?)element.Attribute("Path") == "Host/Host.csproj").SetAttributeValue("Path", "FSharp/Host.fsproj");
        solution.Save(directory.File("Game.slnx"));
        var result = await ProjectCliFixture.Cli(directory, "run", "Game.slnx", "--report", "json");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.Equal(directory.File("FSharp/Host.fsproj"), json.RootElement.GetProperty("target").GetProperty("project").GetString());
        Assert.Equal("C#:F#" + Environment.NewLine, json.RootElement.GetProperty("program").GetProperty("output").GetString());
    }
}
