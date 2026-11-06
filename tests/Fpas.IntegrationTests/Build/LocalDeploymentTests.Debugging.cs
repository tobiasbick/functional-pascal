using System.Text.Json;
using System.Text.Json.Nodes;
using Fpas.IntegrationTests.Debugging;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Build;

public sealed partial class LocalDeploymentTests
{
    private static async Task CheckDebugger(string checkout, string consumer, string launcher, string version, string packages,
        IReadOnlyDictionary<string, string?> environment)
    {
        var project = Path.Combine(Path.GetDirectoryName(consumer)!, "debug-consumer"); Directory.CreateDirectory(project);
        File.Copy(Path.Combine(consumer, "NuGet.Config"), Path.Combine(project, "NuGet.Config"));
        var projectFile = Path.Combine(project, "DebugProbe.fpasproj");
        var source = Path.Combine(project, "Program.fpas");
        File.WriteAllText(projectFile, $"<Project Sdk=\"Fpas.NET.Sdk/{version}\"><PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType></PropertyGroup></Project>");
        File.WriteAllText(source, """
            program DebugProbe;
            uses System.Console as Console;
            begin
              const Value: long := 9007199254740993L;
              Console.WriteLine(Value);
              Console.WriteLine('after');
            end;
            end program;
            """);
        var built = await ProcessRunner.RunExecutableAsync(launcher, ["build", projectFile, "--report", "json"],
            workingDirectory: consumer, environment: environment, timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(built.ExitCode == 0, built.Output + built.Error);
        var assembly = Path.Combine(project, "bin", "Debug", "net10.0", "DebugProbe.dll");
        Assert.True(File.Exists(assembly)); Assert.True(File.Exists(Path.ChangeExtension(assembly, ".pdb")));
        File.WriteAllText(projectFile, "<invalid project to prove debugging never builds");

        // Only the delivered files, target/PDB/sources and installed .NET remain reachable.
        var hidden = new[] { Path.Combine(checkout, "src"), Path.Combine(checkout, "sdk"), packages };
        var moved = new List<string>();
        try
        {
            foreach (var directory in hidden) { Directory.Move(directory, directory + ".offline"); moved.Add(directory); }
            var batch = await ProcessRunner.RunExecutableAsync(launcher, ["debug", assembly, "--break", source + ":5", "--print", "Value"],
                workingDirectory: Path.GetTempPath(), environment: environment);
            Assert.True(batch.ExitCode == 0, batch.Output + batch.Error); Assert.Empty(batch.Error);
            using var report = JsonDocument.Parse(batch.Output);
            Assert.Equal("breakpoint", report.RootElement.GetProperty("outcome").GetString());
            Assert.Equal("9007199254740993", report.RootElement.GetProperty("expressions")[0].GetProperty("value").GetProperty("scalar").GetString());

            await using var dap = new DapProbe(launcher);
            await dap.Success("initialize"); await dap.Success("launch", new() { ["program"] = assembly, ["cwd"] = project });
            await dap.Event("initialized");
            await dap.Success("setBreakpoints", new() { ["source"] = new JsonObject { ["path"] = source }, ["breakpoints"] = new JsonArray(new JsonObject { ["line"] = 5 }) });
            await dap.Success("configurationDone"); var stopped = await dap.Event("stopped");
            var thread = stopped.GetProperty("threadId").GetInt32();
            var stack = await dap.Success("stackTrace", new() { ["threadId"] = thread, ["levels"] = 1 });
            Assert.Equal(5, stack.GetProperty("stackFrames")[0].GetProperty("line").GetInt32());
            var value = await dap.Success("evaluate", new() { ["expression"] = "Value" }); Assert.Equal("9007199254740993", value.GetProperty("result").GetString());
            await dap.Success("next", new() { ["threadId"] = thread }); await dap.Event("stopped");
            await dap.Success("terminate"); await dap.End();
        }
        finally { foreach (var directory in moved.AsEnumerable().Reverse()) Directory.Move(directory + ".offline", directory); }
        Assert.Equal("<invalid project to prove debugging never builds", File.ReadAllText(projectFile));
    }
}
