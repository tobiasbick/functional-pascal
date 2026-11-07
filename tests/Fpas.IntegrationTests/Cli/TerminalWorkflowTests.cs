using System.Diagnostics;
using System.Text.Json;
using System.Text.Json.Nodes;
using Fpas.IntegrationTests.Debugging;
using Fpas.Tests.Support;
using Json.Schema;

namespace Fpas.IntegrationTests.Cli;

public sealed class TerminalWorkflowTests
{
    [Fact]
    public async Task Actual_terminal_input_Unicode_resize_and_exit_are_observed_in_order()
    {
        using var target = new Debuggee("terminal-workflow");
        await target.Compile("""
            program TerminalProbe;
            uses System as Sys, System.Console as Console;
            begin
              Console.InputEncoding := Sys.Text.Encoding.UTF8;
              Console.OutputEncoding := Sys.Text.Encoding.UTF8;
              Console.WriteLine('redirected=' + Console.IsInputRedirected.ToString());
              const Key: Sys.ConsoleKeyInfo := Console.ReadKey(true);
              Console.WriteLine('key=' + Key.KeyChar.ToString());
              Console.WriteLine('width=' + Console.WindowWidth.ToString());
            end;
            end program;
            """);
        var script = Script(target, new object[] { new { action = "expect", text = "redirected=False" },
            new { action = "resize", columns = 100, rows = 40 }, new { action = "send", text = "λ" },
            new { action = "expect", text = "key=λ" }, new { action = "expect", text = "width=100" }, new { action = "exit", exitCode = 0 } });
        var exported = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "schema", "terminal-script"]);
        Assert.Equal(0, exported.ExitCode);
        var schema = JsonSchema.FromText(exported.Output, new BuildOptions { SchemaRegistry = new SchemaRegistry() });
        var input = JsonNode.Parse(File.ReadAllText(script))!;
        Assert.True(schema.Evaluate(JsonSerializer.SerializeToElement(input)).IsValid);
        input["steps"]![1]!["columns"] = 0;
        Assert.False(schema.Evaluate(JsonSerializer.SerializeToElement(input)).IsValid);
        var result = await Run(target, script); Assert.True(result.ExitCode == 0, result.Output + result.Error);
        using var report = JsonDocument.Parse(result.Output);
        Assert.True(report.RootElement.GetProperty("success").GetBoolean());
        Assert.Equal(6, report.RootElement.GetProperty("terminal").GetProperty("completedSteps").GetInt32());
    }

    [Fact]
    public async Task Terminal_output_limit_fails_even_when_the_target_exits_successfully()
    {
        using var target = new Debuggee("terminal-output-limit");
        await target.Compile("""
            program TerminalOutput;
            uses System.Console as Console;
            begin
              Console.WriteLine('This output exceeds one byte.');
            end;
            end program;
            """);
        var script = target.FilePath("terminal.json");
        File.WriteAllText(script, JsonSerializer.Serialize(new { executable = Fpas.Workspaces.Processes.BuildProcess.DotnetHost(),
            arguments = new[] { target.Assembly }, directory = target.DirectoryPath, outputBytes = 1,
            steps = new[] { new { action = "exit", exitCode = 0 } } }));
        var result = await Run(target, script);
        Assert.Equal(1, result.ExitCode);
        using var report = JsonDocument.Parse(result.Output);
        Assert.Equal("output_limit", report.RootElement.GetProperty("status").GetString());
        Assert.True(report.RootElement.GetProperty("terminal").GetProperty("truncated").GetBoolean());
    }

    [Fact]
    public async Task Terminal_timeout_reaps_the_real_target_before_returning_a_failure()
    {
        using var target = new Debuggee("terminal-timeout");
        await target.Compile("""
            program WaitingTerminal;
            uses System as Sys;
            begin
              const Args: array of string := Sys.Environment.GetCommandLineArgs();
              if Args.Length = 1 then
                const Info: Sys.Diagnostics.ProcessStartInfo := new Sys.Diagnostics.ProcessStartInfo();
                Info.FileName := Sys.Environment.ProcessPath ?? '';
                Info.ArgumentList.Add(Args[0]); Info.ArgumentList.Add('child');
                Info.UseShellExecute := false;
                discard Sys.Diagnostics.Process.Start(Info);
                while not Sys.IO.File.Exists('child.pid') do Sys.Threading.Thread.Sleep(10); end while;
              else
                Sys.IO.File.WriteAllText('child.pid', Sys.Environment.ProcessId.ToString());
              end if;
              Sys.Console.WriteLine('ready');
              while true do Sys.Threading.Thread.Sleep(50); end while;
            end;
            end program;
            """);
        var script = Script(target, new object[] { new { action = "expect", text = "ready" }, new { action = "exit", exitCode = 0 } }, seconds: 5);
        var result = await Run(target, script); Assert.Equal(1, result.ExitCode);
        using var report = JsonDocument.Parse(result.Output); Assert.Equal("timeout", report.RootElement.GetProperty("status").GetString());
        var pid = report.RootElement.GetProperty("terminal").GetProperty("processId").GetInt32();
        try { using var child = Process.GetProcessById(pid); Assert.True(child.HasExited); } catch (ArgumentException) { }
        Assert.True(File.Exists(target.FilePath("child.pid")));
        var descendant = int.Parse(File.ReadAllText(target.FilePath("child.pid")), System.Globalization.CultureInfo.InvariantCulture);
        try { using var child = Process.GetProcessById(descendant); Assert.True(child.HasExited); } catch (ArgumentException) { }
    }

    private static string Script(Debuggee target, object[] steps, int seconds = 20)
    {
        var file = target.FilePath("terminal.json");
        File.WriteAllText(file, JsonSerializer.Serialize(new { schemaVersion = 1, executable = Fpas.Workspaces.Processes.BuildProcess.DotnetHost(), arguments = new[] { target.Assembly }, directory = target.DirectoryPath, timeoutSeconds = seconds, steps }));
        return file;
    }
    private static Task<ProcessResult> Run(Debuggee target, string script) => ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "terminal", "test", script], workingDirectory: target.DirectoryPath);
}
