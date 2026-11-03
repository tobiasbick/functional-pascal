using System.Text.Json;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Cli;

public sealed class ArgumentParsingTests
{
    private static string Cli => TestEnvironment.Metadata("FpasCliPath");

    [Theory]
    [InlineData("--unknown")]
    [InlineData("--debug", "invalid")]
    [InlineData("--target", "EXE")]
    [InlineData("--diagnostics", "text")]
    [InlineData("--output")]
    [InlineData("--reference")]
    [InlineData("--output", "Other.dll")]
    [InlineData("--documentation", "one.xml", "--documentation", "two.xml")]
    public async Task Parser_errors_produce_one_JSON_result_before_reading_sources(params string[] arguments)
    {
        using var directory = new TemporaryDirectory();
        var result = await ProcessRunner.RunAsync([Cli, "compile", "missing.fpas", "--output", directory.File("App.dll"),
            "--reference", "missing.dll", "--diagnostics=json", .. arguments]);
        Assert.Equal(2, result.ExitCode);
        Assert.Equal("", result.Error);
        using var report = JsonDocument.Parse(result.Output);
        Assert.False(report.RootElement.GetProperty("success").GetBoolean());
        var diagnostic = Assert.Single(report.RootElement.GetProperty("diagnostics").EnumerateArray());
        Assert.Equal("FP0001", diagnostic.GetProperty("code").GetString());
        Assert.Contains("compile --help", diagnostic.GetProperty("message").GetString());
        Assert.Empty(Directory.GetFiles(directory.Path));
    }

    [Fact]
    public async Task Repeated_references_and_literal_sources_after_double_dash_compile_and_run()
    {
        using var directory = new TemporaryDirectory();
        var sourceDirectory = directory.File("source files");
        Directory.CreateDirectory(sourceDirectory);
        var emptyReferences = directory.File("empty references");
        Directory.CreateDirectory(emptyReferences);
        File.WriteAllText(Path.Combine(sourceDirectory, "--main.fpas"), """
            program App;
            uses Helper as H, System.Console as Console;
            begin Console.WriteLine(H.Value()); end;
            end program;
            """);
        File.WriteAllText(Path.Combine(sourceDirectory, "@helper.fpas"), """
            unit Helper;
            public function Value(): int; begin return 42; end function;
            end unit;
            """);
        var output = directory.File("output files/App.dll");
        var compiled = await ProcessRunner.RunAsync([Cli, "compile", "--output=" + output,
            "--reference", Path.Combine(TestEnvironment.ReferenceDirectory, "System.Runtime.dll"),
            "--reference", Path.Combine(TestEnvironment.ReferenceDirectory, "System.Console.dll"),
            "--reference-dir", emptyReferences, "--reference-dir", TestEnvironment.ReferenceDirectory,
            "--diagnostics=json", "--", "--main.fpas", "@helper.fpas"], workingDirectory: sourceDirectory);
        Assert.True(compiled.ExitCode == 0, compiled.Output + compiled.Error);
        Assert.Equal("", compiled.Error);
        using var report = JsonDocument.Parse(compiled.Output);
        Assert.True(report.RootElement.GetProperty("success").GetBoolean());
        var executed = await ProcessRunner.RunAsync([output]);
        Assert.Equal(0, executed.ExitCode);
        Assert.Equal("42" + Environment.NewLine, executed.Output);
        Assert.Equal("", executed.Error);
    }

    [Fact]
    public async Task JSON_response_arguments_support_stdin_and_structured_parse_errors()
    {
        using var directory = new TemporaryDirectory();
        var response = directory.File("compile arguments.json");
        var output = directory.File("output files/App.dll");
        string[] arguments = ["-", "--output", output, "--reference-dir", TestEnvironment.ReferenceDirectory, "--diagnostics=json"];
        File.WriteAllText(response, JsonSerializer.Serialize(arguments));
        var compiled = await ProcessRunner.RunAsync([Cli, "compile", "--response-file=" + response], "program App; begin end; end program;");
        Assert.True(compiled.ExitCode == 0, compiled.Output + compiled.Error);
        Assert.Equal("", compiled.Error);
        using (var report = JsonDocument.Parse(compiled.Output))
            Assert.True(report.RootElement.GetProperty("success").GetBoolean());

        File.WriteAllText(response, JsonSerializer.Serialize(arguments.Concat(["--target", "invalid"])));
        var invalid = await ProcessRunner.RunAsync([Cli, "compile", "--response-file", response]);
        Assert.Equal(2, invalid.ExitCode);
        Assert.Equal("", invalid.Error);
        using var failure = JsonDocument.Parse(invalid.Output);
        Assert.Equal("FP0001", Assert.Single(failure.RootElement.GetProperty("diagnostics").EnumerateArray()).GetProperty("code").GetString());

        File.WriteAllText(response, JsonSerializer.Serialize(arguments.Concat(["--response-file", "missing.json"])));
        var nested = await ProcessRunner.RunAsync([Cli, "compile", "--response-file", response]);
        Assert.Equal(2, nested.ExitCode);
        Assert.Equal("", nested.Error);
        using var nesting = JsonDocument.Parse(nested.Output);
        var diagnostic = Assert.Single(nesting.RootElement.GetProperty("diagnostics").EnumerateArray());
        Assert.Equal("FP0001", diagnostic.GetProperty("code").GetString());
        Assert.Contains("nesting is unsupported", diagnostic.GetProperty("message").GetString());
    }

    [Fact]
    public async Task Response_files_cannot_mix_with_inline_compile_arguments()
    {
        var result = await ProcessRunner.RunAsync([Cli, "compile", "--response-file", "missing.json", "--diagnostics=json"]);
        Assert.Equal(2, result.ExitCode);
        Assert.Equal("", result.Error);
        using var report = JsonDocument.Parse(result.Output);
        var diagnostic = Assert.Single(report.RootElement.GetProperty("diagnostics").EnumerateArray());
        Assert.Equal("FP0001", diagnostic.GetProperty("code").GetString());
        Assert.Contains("replaces all compile arguments", diagnostic.GetProperty("message").GetString());
    }
}
