using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Processes.Support;
using Fpas.Tests.Support;
using System.Diagnostics;
using System.Text;

namespace Fpas.IntegrationTests.Projects;

public sealed class WindowsConsoleTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [WindowsFact]
    public async Task Run_inherits_real_console_handles_for_Unicode_IO_exit_codes_and_control_C()
    {
        using var directory = new TemporaryDirectory();
        var initialized = await ProjectCliFixture.Cli(directory, "init", "project", "App Grüße", "--name", "App",
            "--sdk-version", sdk.Version, "--sdk-feed", sdk.Feed, "--report", "json");
        Assert.True(initialized.ExitCode == 0, initialized.Output + initialized.Error);
        File.WriteAllText(directory.File("App Grüße/main.fpas"), """
            program Terminal;
            uses System.Console as Console, System.Environment as Env, System.Text.Encoding as Encoding,
              System.IO.File as File;
            begin
              Console.InputEncoding := Encoding.UTF8;
              Console.WriteLine('stdin:' + Console.IsInputRedirected.ToString());
              Console.WriteLine('stdout:' + Console.IsOutputRedirected.ToString());
              Console.WriteLine('stderr:' + Console.IsErrorRedirected.ToString());
              for Value: string in args do Console.WriteLine('arg:<' + Value + '>'); end for;
              Console.Error.WriteLine('error:Grüße');
              File.WriteAllText(Env.GetEnvironmentVariable('FPAS_CONSOLE_READY') ?? 'ready.pid', Env.ProcessId.ToString());
              Console.WriteLine('ready');
              Console.WriteLine('input:' + (Console.ReadLine() ?? 'EOF'));
              Env.ExitCode := 7;
            end;
            end program;
            """);
        var project = directory.File("App Grüße/App.fpasproj");
        var built = await ProjectCliFixture.Cli(directory, "build", project, "--report", "json");
        Assert.True(built.ExitCode == 0, built.Output + built.Error);
        string[] arguments = ["Grüße 😀", "a b", "", "quoted \"value\"", "trailing\\"];
        foreach (var cancel in new[] { false, true })
        {
            var ready = directory.File("ready.pid"); File.Delete(ready);
            using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(1));
            await using var console = WindowsConsole.Start(TestEnvironment.DotnetHost,
                [TestEnvironment.Metadata("FpasCliPath"), "run", project, "--no-build", "--", .. arguments], directory.Path,
                capture: false, environment: new Dictionary<string, string> { ["FPAS_CONSOLE_READY"] = ready });
            while (!console.Transcript.Contains("ready", StringComparison.Ordinal))
            {
                Assert.False(console.HasExited, "CLI exited before console input: " + console.Transcript);
                await Task.Delay(20, deadline.Token);
            }
            var target = int.Parse(File.ReadAllText(ready), System.Globalization.CultureInfo.InvariantCulture);
            if (cancel) await console.Interrupt();
            else
            {
                await console.Input.WriteAsync(Encoding.UTF8.GetBytes("Eingabe Grüße 😀\r"), deadline.Token);
                await console.Input.FlushAsync(deadline.Token);
            }
            await console.WaitForExit(deadline.Token);
            Assert.Equal(cancel ? 130 : 7, console.ExitCode);
            // Observe target cleanup before closing ConPTY or the harness job could terminate it.
            Assert.Throws<ArgumentException>(() => Process.GetProcessById(target));
            await console.DrainTerminal();
            var output = console.Transcript;
            if (!cancel) Assert.Contains("input:Eingabe Grüße 😀", output);
            foreach (var stream in new[] { "stdin", "stdout", "stderr" }) Assert.Contains(stream + ":False", output);
            foreach (var argument in arguments) Assert.Contains("arg:<" + argument + ">", output);
            Assert.Contains("error:Grüße", output);
        }
    }
}
