using System.Text.Json;
using Fpas.IntegrationTests.Debugging;
using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Tui;

public sealed class TerminalModeTests
{
    [Theory]
    [InlineData("normal")]
    [InlineData("failure")]
    public async Task Interactive_TUI_restores_native_modes_before_returning(string scenario)
    {
        using var target = new Debuggee("tui-terminal-modes");
        await TuiFixture.BuildLibrary(target);
        var cli = TestEnvironment.Metadata("FpasCliPath");
        File.Copy(Path.Combine(Path.GetDirectoryName(cli)!, "Fpas.Core.dll"), target.FilePath("Fpas.Core.dll"));
        File.Copy(Path.Combine(TuiFixture.Fixtures, "TerminalModes.cs"), target.FilePath("TerminalModes.cs"));
        var project = target.FilePath("Probe.csproj");
        File.WriteAllText(project, """
            <Project Sdk="Microsoft.NET.Sdk">
              <PropertyGroup>
                <TargetFramework>net10.0</TargetFramework>
                <OutputType>Exe</OutputType>
                <ImplicitUsings>enable</ImplicitUsings>
                <Nullable>enable</Nullable>
              </PropertyGroup>
              <ItemGroup>
                <Reference Include="Fpas.Tui" HintPath="Fpas.Tui.dll" />
                <Reference Include="Fpas.Mvu" HintPath="Fpas.Mvu.dll" />
                <Reference Include="Fpas.Ui" HintPath="Fpas.Ui.dll" />
                <Reference Include="Fpas.Processes" HintPath="Fpas.Processes.dll" />
                <Reference Include="Fpas.Core" HintPath="Fpas.Core.dll" />
              </ItemGroup>
            </Project>
            """);
        var dotnet = BuildProcess.DotnetHost();
        var build = await ProcessRunner.RunExecutableAsync(dotnet, ["build", project, "--nologo", "-warnaserror"],
            workingDirectory: target.DirectoryPath);
        Assert.True(build.ExitCode == 0, build.Output + build.Error);

        var script = target.FilePath("terminal.json");
        File.WriteAllText(script, JsonSerializer.Serialize(new
        {
            executable = dotnet,
            arguments = new[] { target.FilePath("bin/Debug/net10.0/Probe.dll"), scenario },
            directory = target.DirectoryPath,
            timeoutSeconds = 15,
            steps = new object[]
            {
                new { action = "expect", text = "terminal-ready" },
                new { action = "send", text = "\r" },
                new { action = "exit", exitCode = 0 }
            }
        }));
        var run = await ProcessRunner.RunAsync([cli, "terminal", "test", script], workingDirectory: target.DirectoryPath);
        Assert.True(run.ExitCode == 0, run.Output + run.Error);

        using var report = JsonDocument.Parse(File.ReadAllText(target.FilePath("terminal-modes.json")));
        var result = report.RootElement;
        var before = result.GetProperty("Before");
        var active = result.GetProperty("Active");
        var after = result.GetProperty("After");
        Assert.NotEqual(before.GetProperty("Input").GetString(), active.GetProperty("Input").GetString());
        Assert.Equal(before.GetProperty("Input").GetString(), after.GetProperty("Input").GetString());
        if (OperatingSystem.IsWindows())
        {
            Assert.NotEqual(before.GetProperty("Output").GetUInt32(), active.GetProperty("Output").GetUInt32());
            Assert.Equal(before.GetProperty("Output").GetUInt32(), after.GetProperty("Output").GetUInt32());
        }
        var error = result.GetProperty("Error").GetString();
        if (scenario == "failure") Assert.StartsWith("TUI presentation failed:", error);
        else Assert.Null(error);
    }
}
