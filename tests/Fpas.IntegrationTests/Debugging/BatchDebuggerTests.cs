using System.Text.Json;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Debugging;

public sealed partial class BatchDebuggerTests
{
    [Fact]
    public async Task Breakpoint_capture_is_detached_passive_and_does_not_resume_past_the_requested_statement()
    {
        using var program = new Debuggee("batch-capture-λ space");
        await program.Compile("""
            program BatchCapture;
            uses System.Console as Console, System.IO.File as File;
            begin
              const Answer: long := 9007199254740993L;
              const Numbers: int[] := [3, 7];
              File.WriteAllText('before.txt', 'before');
              Console.WriteLine(Answer);
              File.WriteAllText('after.txt', 'after');
            end;
            end program;
            """);
        var result = await Run(program, "--break", "Program.fpas:7", "--break", "Program.fpas:7", "--print", "Answer", "--print", "Numbers[1] + 1");
        using var json = Report(result, 0); var root = json.RootElement;
        Assert.Equal("breakpoint", root.GetProperty("outcome").GetString());
        Assert.Equal("debugger_requested", root.GetProperty("termination").GetString());
        Assert.True(File.Exists(program.FilePath("before.txt"))); Assert.False(File.Exists(program.FilePath("after.txt")));
        Assert.Single(root.GetProperty("breakpoints").EnumerateArray());
        var values = root.GetProperty("expressions").EnumerateArray().ToArray();
        Assert.Equal("9007199254740993", values[0].GetProperty("value").GetProperty("scalar").GetString());
        Assert.Equal("8", values[1].GetProperty("value").GetProperty("scalar").GetString());
        Assert.False(ContainsHandle(root)); Assert.NotEmpty(root.GetProperty("frames").EnumerateArray());
    }

    [Theory]
    [InlineData(0)]
    [InlineData(17)]
    public async Task Natural_exit_keeps_target_code_and_separates_escaped_stdout_and_stderr(int exit)
    {
        using var program = new Debuggee("batch-exit");
        await program.Compile($$"""
            program BatchExit;
            uses System.Console as Console, System.Environment as Environment, System.Text.Encoding as Encoding;
            begin
              Console.OutputEncoding := Encoding.UTF8;
              Console.WriteLine('λ😀'); Console.Error.WriteLine('error');
              Environment.ExitCode := {{exit}};
            end;
            end program;
            """);
        using var json = Report(await Run(program), exit == 0 ? 0 : 3);
        var root = json.RootElement; Assert.Equal("completed", root.GetProperty("outcome").GetString());
        Assert.Equal(exit, root.GetProperty("targetExitCode").GetInt32()); Assert.Equal("natural", root.GetProperty("termination").GetString());
        var output = root.GetProperty("output").GetProperty("items").EnumerateArray().ToArray();
        Assert.Contains(output, item => item.GetProperty("stream").GetString() == "stdout" && item.GetProperty("text").GetString()!.Contains("λ😀", StringComparison.Ordinal));
        Assert.Contains(output, item => item.GetProperty("stream").GetString() == "stderr" && item.GetProperty("text").GetString()!.Contains("error", StringComparison.Ordinal));
        Assert.Empty(root.GetProperty("bindings").EnumerateArray());
    }

    [Fact]
    public async Task Independent_expression_failure_retains_successful_observations_at_the_same_stop()
    {
        using var program = new Debuggee("batch-expression-failure");
        await program.Compile("program Observations; uses System.Console as Console; begin const Value: int := 42; Console.WriteLine(Value); end; end program;");
        using var json = Report(await Run(program, "--break", "Program.fpas:1:96", "--print", "Value", "--print", "Missing"), 3);
        var values = json.RootElement.GetProperty("expressions").EnumerateArray().ToArray();
        Assert.Equal("42", values[0].GetProperty("value").GetProperty("scalar").GetString()); Assert.True(values[1].TryGetProperty("error", out _));
    }

    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task Deadline_and_exact_output_limit_end_owned_execution(bool output)
    {
        using var program = new Debuggee("batch-limit");
        await program.Compile(output ? "program OutputLimit; uses System.Console as Console; begin Console.Write('1234567'); end; end program;"
            : "program Deadline; uses System.Threading.Thread as Thread; begin while true do Thread.Sleep(100); end while; end; end program;");
        var elapsed = System.Diagnostics.Stopwatch.StartNew();
        using var json = Report(await Run(program, output ? ["--output-limit", "7"] : ["--timeout", "2"]), 4);
        Assert.True(elapsed.Elapsed < TimeSpan.FromSeconds(5), "The bounded invocation overran cleanup grace.");
        Assert.Equal("limit", json.RootElement.GetProperty("outcome").GetString());
        if (output) Assert.Equal(7, json.RootElement.GetProperty("output").GetProperty("totalBytes").GetInt64());
    }

    [Theory]
    [InlineData("--protocol", "unknown")]
    [InlineData("--timeout", "0")]
    [InlineData("--print", "Call()")]
    public async Task Invalid_requests_return_one_json_report_without_executing_target_statements(string option, string value)
    {
        using var program = new Debuggee("batch-invalid");
        await program.Compile("program Invalid; uses System.IO.File as File; begin File.WriteAllText('ran.txt', 'ran'); end; end program;");
        using var json = Report(await Run(program, option, value), 2);
        Assert.Equal("error", json.RootElement.GetProperty("outcome").GetString()); Assert.False(File.Exists(program.FilePath("ran.txt")));
    }

    private static Task<ProcessResult> Run(Debuggee program, params string[] arguments)
        => ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "debug", program.Assembly, "--cwd", program.DirectoryPath, .. arguments],
            workingDirectory: Path.GetTempPath(), timeLimit: TimeSpan.FromSeconds(40));

    private static JsonDocument Report(ProcessResult result, int expected)
    {
        Assert.Equal("", result.Error); Assert.EndsWith("\n", result.Output);
        var report = JsonDocument.Parse(result.Output);
        Assert.True(result.ExitCode == expected, result.Output);
        Assert.Equal(expected, report.RootElement.GetProperty("exitCode").GetInt32());
        Assert.Equal(1, report.RootElement.GetProperty("schemaVersion").GetInt32()); return report;
    }

    private static bool ContainsHandle(JsonElement value) => value.ValueKind switch
    {
        JsonValueKind.Object => value.EnumerateObject().Any(item => item.Name is "sessionId" or "stopId" or "frameId" or "reference" or "moduleId" || ContainsHandle(item.Value)),
        JsonValueKind.Array => value.EnumerateArray().Any(ContainsHandle), _ => false
    };
}
