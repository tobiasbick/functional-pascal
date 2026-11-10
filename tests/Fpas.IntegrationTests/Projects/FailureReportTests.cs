using Fpas.IntegrationTests.Debugging;
using Fpas.Tests.Support;
using System.Diagnostics;
using System.Text;
using System.Text.Json;
using Fpas.Processes.Terminal;

namespace Fpas.IntegrationTests.Projects;

/// <summary>Acceptance for automatic capture of the original failure during ordinary CLI runs.</summary>
public sealed class FailureReportTests
{
    /// <summary>Capture real exception state once while preserving input, output and the natural crash exit.</summary>
    /// <param name="json">Whether the ordinary run result uses JSON output.</param>
    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task Original_failure_is_saved_without_replay_and_preserves_natural_exit_and_streams(bool json)
    {
        using var program = new Debuggee("run-failure-λ space");
        await program.Compile("""
            program FailureCapture;
            uses System.Console as Console, System.InvalidOperationException as Failure,
              System.IO.File as File, System.Text.Encoding as Encoding;
            function Crash(Input: string): int;
            begin
              const Local: int := Input.Length + 34;
              File.AppendAllText('runs.txt', 'x');
              Console.WriteLine(Input);
              Console.WriteLine(Local);
              raise new Failure('original failure');
            end function;
            begin
              Console.InputEncoding := Encoding.UTF8;
              discard Crash(Console.ReadLine() ?? 'EOF');
            end;
            end program;
            """);
        var baseline = await Run(program, "Grüße 😀\n");
        Assert.NotEqual(0, baseline.ExitCode);
        File.Delete(program.FilePath("runs.txt"));
        var path = program.FilePath(Path.Combine("reports", "failure.json"));
        var result = await Run(program, "Grüße 😀\n", ["--failure-report", path, .. json ? new[] { "--report", "json" } : []]);
        Assert.True(result.ExitCode == baseline.ExitCode, result.Output + result.Error);
        Assert.Equal("x", File.ReadAllText(program.FilePath("runs.txt")));
        var output = result.Output;
        if (json)
        {
            Assert.Equal("", result.Error);
            using var run = JsonDocument.Parse(result.Output);
            Assert.Equal(path, run.RootElement.GetProperty("failureReport").GetString());
            Assert.Equal(result.ExitCode, run.RootElement.GetProperty("program").GetProperty("exitCode").GetInt32());
            output = run.RootElement.GetProperty("program").GetProperty("output").GetString()!;
            Assert.Contains("original failure", run.RootElement.GetProperty("program").GetProperty("error").GetString());
        }
        else Assert.Contains("Failure report: " + path, result.Error);
        Assert.Equal("Grüße 😀" + Environment.NewLine + "42" + Environment.NewLine, output);
        Assert.True(new FileInfo(path).Length < 8 * 1024 * 1024);
        using var report = JsonDocument.Parse(File.ReadAllText(path));
        var root = report.RootElement;
        Assert.Equal("exception", root.GetProperty("outcome").GetString());
        Assert.Equal("natural", root.GetProperty("termination").GetString());
        Assert.Equal(result.ExitCode, root.GetProperty("targetExitCode").GetInt32());
        Assert.Equal("System.InvalidOperationException", root.GetProperty("exception").GetProperty("type").GetString());
        Assert.Equal("original failure", root.GetProperty("exception").GetProperty("message").GetProperty("text").GetString());
        Assert.NotEmpty(root.GetProperty("exception").GetProperty("throwStack").EnumerateArray());
        Assert.NotEmpty(root.GetProperty("frames").EnumerateArray());
        Assert.Contains(root.GetProperty("bindings").EnumerateArray(), item => item.GetProperty("value").GetProperty("name").GetString() == "Local"
            && item.GetProperty("value").GetProperty("scalar").GetString() == "42");
        Assert.Contains(root.GetProperty("bindings").EnumerateArray(), item => item.GetProperty("value").GetProperty("name").GetString() == "Input"
            && item.GetProperty("value").GetProperty("scalar").GetString() == "Grüße 😀");
        Assert.NotEqual(JsonValueKind.Null, root.GetProperty("toolchain").ValueKind);
        Assert.Empty(Directory.GetFiles(Path.GetDirectoryName(path)!, "*.tmp"));
    }

    /// <summary>Nonzero return values and handled exceptions do not produce crash reports.</summary>
    /// <param name="exit">Natural program exit code.</param>
    /// <param name="json">Whether stdout is reserved for the project report.</param>
    [Theory]
    [InlineData(0, false)]
    [InlineData(17, false)]
    [InlineData(0, true)]
    [InlineData(17, true)]
    public async Task Completed_runs_do_not_create_reports(int exit, bool json)
    {
        using var program = new Debuggee("run-completed");
        await program.Compile($$"""
            program Completed;
            uses System.Console as Console, System.Environment as Env, System.InvalidOperationException as Failure;
            begin
              try
                raise new Failure('handled');
              catch Error: Failure do
                Console.WriteLine('handled');
              end try;
              Env.ExitCode := {{exit}};
            end;
            end program;
            """);
        var path = program.FilePath("failure.json");
        var result = await Run(program, null, ["--failure-report", path, .. json ? new[] { "--report", "json" } : []]);
        Assert.True(result.ExitCode == exit, result.Output + result.Error);
        Assert.False(File.Exists(path));
        Assert.Empty(Directory.GetFiles(program.DirectoryPath, "*.tmp"));
        if (!json) Assert.Equal("handled" + Environment.NewLine, result.Output);
        else
        {
            using var report = JsonDocument.Parse(result.Output);
            Assert.False(report.RootElement.TryGetProperty("failureReport", out _));
            Assert.Equal("handled" + Environment.NewLine, report.RootElement.GetProperty("program").GetProperty("output").GetString());
        }
    }

    /// <summary>Invalid report storage and unavailable symbols fail before target side effects.</summary>
    /// <param name="existing">Whether an existing report or missing symbols cause rejection.</param>
    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task Invalid_capture_setup_does_not_execute_target(bool existing)
    {
        using var program = new Debuggee("run-invalid-capture");
        await program.Compile("program Invalid; uses System.IO.File as File; begin File.WriteAllText('ran.txt', 'ran'); end; end program;");
        var path = program.FilePath("failure.json");
        if (existing) File.WriteAllText(path, "preserve"); else File.Delete(Path.ChangeExtension(program.Assembly, ".pdb"));
        var result = await Run(program, null, "--failure-report", path, "--report", "json");
        Assert.True(result.ExitCode != 0, result.Output);
        using var report = JsonDocument.Parse(result.Output);
        Assert.Equal(existing ? "invalid_argument" : "operation_failed", report.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.False(File.Exists(program.FilePath("ran.txt")));
        if (existing) Assert.Equal("preserve", File.ReadAllText(path)); else Assert.False(File.Exists(path));
        Assert.Empty(Directory.GetFiles(program.DirectoryPath, "*.tmp"));
    }

    /// <summary>A run deadline terminates its owned target without manufacturing an exception report.</summary>
    [Fact]
    public async Task Deadline_cleans_up_the_original_process()
    {
        using var program = new Debuggee("run-failure-timeout");
        await program.Compile("""
            program Waiting;
            uses System.IO.File as File, System.Environment as Env, System.Threading.Thread as Thread;
            begin
              File.WriteAllText('target.pid', Env.ProcessId.ToString());
              while true do Thread.Sleep(20); end while;
            end;
            end program;
            """);
        var path = program.FilePath("failure.json");
        var result = await Run(program, null, "--failure-report", path, "--timeout", "3", "--report", "json");
        Assert.True(result.ExitCode == 1, result.Output + result.Error);
        using var report = JsonDocument.Parse(result.Output);
        Assert.Equal("timeout", report.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.False(File.Exists(path));
        var id = int.Parse(File.ReadAllText(program.FilePath("target.pid")), System.Globalization.CultureInfo.InvariantCulture);
        Assert.Throws<ArgumentException>(() => Process.GetProcessById(id));
    }

    /// <summary>The debugger worker preserves terminal handles and accepts input sent after a live prompt.</summary>
    [Fact]
    public async Task Interactive_console_input_remains_inherited()
    {
        using var program = new Debuggee("run-failure-terminal");
        await program.Compile("""
            program Interactive;
            uses System.Console as Console, System.Environment as Env, System.Text.Encoding as Encoding;
            begin
              Console.InputEncoding := Encoding.UTF8;
              Console.WriteLine('input:' + Console.IsInputRedirected.ToString());
              Console.WriteLine('output:' + Console.IsOutputRedirected.ToString());
              Console.WriteLine('error:' + Console.IsErrorRedirected.ToString());
              Console.WriteLine('ready');
              Console.WriteLine('received:' + (Console.ReadLine() ?? 'EOF'));
              Env.ExitCode := 17;
            end;
            end program;
            """);
        var path = program.FilePath("failure.json");
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var terminal = PseudoTerminal.Start(TestEnvironment.DotnetHost,
            [TestEnvironment.Metadata("FpasCliPath"), "run", program.Assembly, "--failure-report", path], program.DirectoryPath);
        using var reader = new StreamReader(terminal.Output, Encoding.UTF8);
        var text = new StringBuilder();
        while (!text.ToString().Contains("ready", StringComparison.Ordinal))
        {
            var line = await reader.ReadLineAsync(deadline.Token);
            Assert.NotNull(line);
            text.AppendLine(line);
        }
        await terminal.Input.WriteAsync(Encoding.UTF8.GetBytes("Grüße 😀" + (OperatingSystem.IsWindows() ? "\r" : "\n")), deadline.Token);
        await terminal.Input.FlushAsync(deadline.Token);
        while (!text.ToString().Contains("received:Grüße 😀", StringComparison.Ordinal))
        {
            var line = await reader.ReadLineAsync(deadline.Token);
            Assert.NotNull(line);
            text.AppendLine(line);
        }
        Assert.Equal(17, await terminal.Completion.WaitAsync(deadline.Token));
        foreach (var stream in new[] { "input", "output", "error" }) Assert.Contains(stream + ":False", text.ToString());
        Assert.Contains("received:Grüße 😀", text.ToString());
        Assert.False(File.Exists(path));
    }

    /// <summary>Captured output is bounded without preventing the later original-failure snapshot.</summary>
    [Fact]
    public async Task Output_budget_does_not_stop_failure_capture()
    {
        using var program = new Debuggee("run-failure-output-limit");
        await program.Compile("""
            program LoudFailure;
            uses System as S, System.Console as Console, System.InvalidOperationException as Failure;
            begin
              Console.Write(new S.String(#120, 1200000));
              raise new Failure('failure after output');
            end;
            end program;
            """);
        var path = program.FilePath("failure.json");
        var result = await Run(program, null, "--failure-report", path, "--report", "json");
        Assert.True(result.ExitCode == 1, result.Output + result.Error);
        using var run = JsonDocument.Parse(result.Output);
        Assert.Equal("output_limit", run.RootElement.GetProperty("status").GetString());
        Assert.True(run.RootElement.GetProperty("program").GetProperty("truncated").GetBoolean());
        using var report = JsonDocument.Parse(File.ReadAllText(path));
        Assert.Equal("failure after output", report.RootElement.GetProperty("exception").GetProperty("message").GetProperty("text").GetString());
        Assert.True(report.RootElement.GetProperty("output").GetProperty("truncated").GetBoolean());
        Assert.Equal("natural", report.RootElement.GetProperty("termination").GetString());
    }

    private static Task<ProcessResult> Run(Debuggee program, string? input, params string[] options)
        => ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "run", program.Assembly, .. options],
            input, program.DirectoryPath, timeLimit: TimeSpan.FromSeconds(40));
}
