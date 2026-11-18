using System.Reflection.PortableExecutable;
using System.Xml.Linq;
using Fpas.IntegrationTests.Processes.Support;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Projects;

public sealed partial class PublishingTests
{
    /// <summary>Checks each Windows launcher header and executes the published FPAS entry point.</summary>
    [WindowsFact]
    public async Task WinExe_publishes_a_runnable_GUI_subsystem_in_each_executable_mode()
    {
        using var directory = new TemporaryDirectory();
        var project = await ProjectCliFixture.Init(directory, sdk);
        var xml = XDocument.Load(project);
        xml.Root!.Descendants("OutputType").Single().Value = "WinExe";
        xml.Save(project);
        File.WriteAllText(directory.File("App/main.fpas"), """
            program App;
            uses System.IO.File as File;
            begin
              File.WriteAllText(args[0], 'started');
            end;
            end program;
            """);

        var build = await Cli(directory, "build", project);
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var check = await Cli(directory, "check", project);
        Assert.True(check.ExitCode == 0, check.Output + check.Error);

        foreach (string mode in new[] { "apphost", "self-contained", "single-file", "aot" })
        {
            string output = directory.File(mode);
            var published = await Cli(directory, "publish", project, "--mode", mode, "--output", output);
            Assert.True(published.ExitCode == 0, mode + ": " + published.Output + published.Error);
            string executable = Path.Combine(output, "App.exe");
            using (var image = new PEReader(File.OpenRead(executable)))
                Assert.Equal(Subsystem.WindowsGui, image.PEHeaders.PEHeader!.Subsystem);

            string receipt = directory.File(mode + ".txt");
            var run = await ProcessRunner.RunExecutableAsync(executable, [receipt], workingDirectory: directory.Path);
            Assert.True(run.ExitCode == 0, mode + ": " + run.Output + run.Error);
            Assert.Equal("started", File.ReadAllText(receipt));
        }
    }
}
