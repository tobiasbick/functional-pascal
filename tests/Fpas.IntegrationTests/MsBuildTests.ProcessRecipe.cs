using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

public sealed partial class MsBuildTests
{
    [Theory]
    [InlineData("Debug")]
    [InlineData("Release")]
    public async Task Documented_process_recipe_captures_bounds_and_owns_real_children(string configuration)
    {
        using var project = sdk.CreateProject();
        var document = Path.Combine(AppContext.BaseDirectory, "Fixtures", "specification", "libraries", "processes.md");
        var unit = MarkdownExamples.Read(document).Single(example => example.Library);
        File.WriteAllText(project.File("Game/Bridge.fpas"), unit.Source);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using System.ComponentModel;
            using System.Diagnostics;
            using System.Text;
            using System.Text.Json;
            using Capture = Recipes.ProcessCapture;

            Console.OutputEncoding = Console.InputEncoding = Encoding.UTF8;
            if (args.Length > 0)
            {
                switch (args[0])
                {
                    case "echo":
                        if (Console.In.ReadToEnd().Length != 0) throw new Exception("Expected stdin EOF");
                        Console.WriteLine(JsonSerializer.Serialize(args[1..]));
                        Console.WriteLine(Environment.GetEnvironmentVariable("FPAS_RECIPE_VALUE"));
                        Console.WriteLine(Environment.CurrentDirectory);
                        Console.Error.Write("error: café");
                        return 7;
                    case "flood":
                        for (var i = 0; i < 256; i++) Console.Write(new string('o', 8192));
                        for (var i = 0; i < 256; i++) Console.Error.Write(new string('e', 8192));
                        return 0;
                    case "wait":
                        File.WriteAllText(args[1] + ".tmp", Environment.ProcessId.ToString());
                        File.Move(args[1] + ".tmp", args[1]);
                        await Task.Delay(Timeout.Infinite);
                        return 0;
                    default: throw new Exception("Unknown worker mode");
                }
            }

            var host = Environment.ProcessPath ?? throw new Exception("Missing dotnet host");
            var worker = Path.Combine(AppContext.BaseDirectory, "Host.dll");
            var originalDirectory = Environment.CurrentDirectory;
            var originalEnvironment = Environment.GetEnvironmentVariable("FPAS_RECIPE_VALUE");
            var childDirectory = Path.Combine(originalDirectory, "child directory");
            Directory.CreateDirectory(childDirectory);
            ProcessStartInfo Options(params string[] values)
            {
                var options = new ProcessStartInfo(host) { WorkingDirectory = childDirectory };
                options.ArgumentList.Add(worker);
                foreach (var value in values) options.ArgumentList.Add(value);
                options.Environment["FPAS_RECIPE_VALUE"] = "child only";
                return options;
            }
            void Check(bool condition, string message)
            {
                if (!condition) throw new Exception(message);
            }
            void Exited(string ready)
            {
                var id = int.Parse(File.ReadAllText(ready));
                try
                {
                    using var child = Process.GetProcessById(id);
                    Check(child.HasExited, "Recipe returned while its child was alive");
                }
                catch (ArgumentException) { }
            }

            string[] literal = ["", "space here", "double\"quote", "back\\slash", "; $() &", "café"];
            var echo = await Capture.Run(Options(["echo", .. literal]), 10000, 4096, CancellationToken.None);
            Check(echo.ExitCode == 7, "Nonzero exit status was lost");
            var lines = echo.Output.Value.Replace("\r\n", "\n").TrimEnd('\n').Split('\n');
            Check(JsonSerializer.Deserialize<string[]>(lines[0])!.SequenceEqual(literal), "Arguments changed");
            Check(lines[1] == "child only" && lines[2] == childDirectory, "Environment or cwd changed");
            Check(echo.Error.Value == "error: café", "stderr or UTF-8 was lost");
            Check(!echo.Output.Truncated && !echo.Error.Truncated, "Short capture was truncated");
            Check(Environment.CurrentDirectory == originalDirectory &&
                Environment.GetEnvironmentVariable("FPAS_RECIPE_VALUE") == originalEnvironment, "Parent state changed");

            foreach (var limit in new[] { 0, 37 })
            {
                var flood = await Capture.Run(Options("flood"), 10000, limit, CancellationToken.None);
                Check(flood.ExitCode == 0 && flood.Output.Value == new string('o', limit) &&
                    flood.Error.Value == new string('e', limit), "Capture prefix/limit changed");
                Check(flood.Output.Truncated && flood.Error.Truncated, "Truncation was not reported");
            }

            var timeoutReady = Path.Combine(childDirectory, "timeout.ready");
            try
            {
                await Capture.Run(Options("wait", timeoutReady), 3000, 64, CancellationToken.None);
                throw new Exception("Timeout was ignored");
            }
            catch (TimeoutException) { Exited(timeoutReady); }

            using var cancellation = new CancellationTokenSource();
            var cancellationReady = Path.Combine(childDirectory, "cancellation.ready");
            var pending = Capture.Run(Options("wait", cancellationReady), 15000, 64, cancellation.Token);
            try
            {
                using var startup = new CancellationTokenSource(10000);
                while (!File.Exists(cancellationReady)) await Task.Delay(10, startup.Token);
                cancellation.Cancel();
                try
                {
                    await pending;
                    throw new Exception("Cancellation was ignored");
                }
                catch (OperationCanceledException failure)
                {
                    Check(failure.CancellationToken == cancellation.Token, "Caller token was lost");
                    Exited(cancellationReady);
                }
            }
            finally
            {
                cancellation.Cancel();
                try { await pending; }
                catch (OperationCanceledException) { }
            }

            var missing = new ProcessStartInfo(Path.Combine(childDirectory, "missing-executable"));
            try
            {
                await Capture.Run(missing, 1000, 64, cancellation.Token);
                throw new Exception("Pre-cancellation was ignored");
            }
            catch (OperationCanceledException) { }
            try
            {
                await Capture.Run(missing, 1000, 64, CancellationToken.None);
                throw new Exception("Launch failure was ignored");
            }
            catch (Win32Exception) { }
            Console.WriteLine("Process recipe verified");
            return 0;
            """);
        var build = await sdk.Run(project, "build", "Game.slnx", "--configuration", configuration, "-warnaserror");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await ProcessRunner.RunAsync([project.File($"Host/bin/{configuration}/net10.0/Host.dll")],
            workingDirectory: project.Path, timeLimit: TimeSpan.FromSeconds(45));
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("Process recipe verified" + Environment.NewLine, run.Output);
        Assert.Empty(run.Error);
    }
}
