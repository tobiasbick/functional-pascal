// [Documentation](../../../docs/pascal/program-structure/cli.md#check-one-source-file)
using Fpas.IntegrationTests.Documentation;
using Fpas.Tests.Support;
using System.Text.Json;

namespace Fpas.IntegrationTests.Cli;

public sealed class PipedSourceTests
{
    [Fact]
    public async Task Check_reads_a_complete_program_without_writing_an_output()
    {
        using var directory = Scratch();
        var checkedSource = await Cli(directory, "program Probe; begin end; end program;", "check", "-");
        Assert.True(checkedSource.ExitCode == 0, checkedSource.Output + checkedSource.Error);
        using var report = JsonDocument.Parse(checkedSource.Output);
        Assert.Equal("<stdin>", report.RootElement.GetProperty("context").GetString());
        Assert.True(report.RootElement.GetProperty("coverage").GetProperty("complete").GetBoolean());
        Assert.Empty(Directory.EnumerateFileSystemEntries(directory.Path));
    }

    [Theory]
    [InlineData("check")]
    [InlineData("run")]
    public async Task Piped_compiler_errors_name_the_source_and_its_exact_position(string command)
    {
        using var directory = Scratch();
        string[] options = command == "check" ? ["--diagnostic-file", "<stdin>"] : [];
        var failure = await Cli(directory, "program Probe;\nbegin\n  Missing();\nend;\nend program;", [command, "-", .. options]);
        Assert.Equal(1, failure.ExitCode);
        Assert.Empty(failure.Error);
        using var report = JsonDocument.Parse(failure.Output);
        var body = report.RootElement;
        Assert.Equal("<stdin>", body.GetProperty("context").GetString());
        Assert.False(body.TryGetProperty("program", out _));
        var diagnostics = command == "check" ? body.GetProperty("diagnostics") : body.GetProperty("build").GetProperty("diagnostics");
        var diagnostic = Assert.Single(diagnostics.EnumerateArray());
        Assert.Equal("FP3001", diagnostic.GetProperty("code").GetString());
        var location = diagnostic.GetProperty("location");
        Assert.Equal("<stdin>", location.GetProperty("path").GetString());
        Assert.Equal(3, location.GetProperty("line").GetInt32());
        Assert.Equal(3, location.GetProperty("column").GetInt32());
        Assert.Empty(Directory.EnumerateFileSystemEntries(directory.Path));
    }

    [Fact]
    public async Task Run_reads_separate_program_input_resolves_core_and_removes_its_managed_output()
    {
        using var directory = Scratch();
        var cwd = directory.File("working dir");
        Directory.CreateDirectory(cwd);
        File.WriteAllText(Path.Combine(cwd, "data.txt"), "Grüße\n");
        const string source = """
            program Probe;

            uses System.Console as Console; uses System.Text.Encoding as Encoding;
              uses System.Reflection.Assembly as Assembly; uses Fpas.Core as Core;

            begin
              Console.InputEncoding := Encoding.UTF8;
              Console.OutputEncoding := Encoding.UTF8;
              const Marker: Core.RecordAttribute := new Core.RecordAttribute(42);
              Console.WriteLine(Marker.Version);
              Console.WriteLine(Console.ReadLine() ?? 'EOF');
              Console.WriteLine(Console.ReadLine() ?? 'EOF');
              Console.WriteLine(args[0]);
              Console.WriteLine(Assembly.GetExecutingAssembly().Location);
            end;

            end program;
            """;
        var run = await Cli(directory, source, "run", "-", "--cwd", cwd, "--stdin", "data.txt", "--", "one argument");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Empty(run.Error);
        using var report = JsonDocument.Parse(run.Output);
        var body = report.RootElement;
        Assert.Equal("<stdin>", body.GetProperty("context").GetString());
        Assert.False(body.TryGetProperty("target", out _));
        var output = body.GetProperty("program").GetProperty("output").GetString()!;
        var lines = output.Split(Environment.NewLine, StringSplitOptions.RemoveEmptyEntries);
        Assert.Equal(new[] { "42", "Grüße", "EOF", "one argument" }, lines[..4]);
        Assert.EndsWith(".dll", lines[4]);
        Assert.False(Directory.Exists(Path.GetDirectoryName(lines[4])));
        Assert.Equal(cwd, Assert.Single(Directory.EnumerateFileSystemEntries(directory.Path)));
    }

    [Fact]
    public async Task Run_rejects_using_stdin_for_both_source_and_program_data()
    {
        using var directory = Scratch();
        var failure = await Cli(directory, null, "run", "-", "--stdin", "-");
        Assert.Equal(2, failure.ExitCode);
        using var report = JsonDocument.Parse(failure.Output);
        var error = report.RootElement.GetProperty("error");
        Assert.Equal("invalid_argument", error.GetProperty("code").GetString());
        Assert.Contains("--stdin FILE", error.GetProperty("message").GetString());
        Assert.Empty(Directory.EnumerateFileSystemEntries(directory.Path));
    }

    private static TemporaryDirectory Scratch() => new(Path.Combine(DocumentationLinks.Repository, ".temp-data"));

    private static Task<ProcessResult> Cli(TemporaryDirectory directory, string? source, params string[] arguments)
        => ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), arguments[0], "--report", "json", .. arguments.Skip(1)],
            input: source, workingDirectory: directory.Path);
}
