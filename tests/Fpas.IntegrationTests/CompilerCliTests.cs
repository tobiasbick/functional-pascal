using System.Text.Json;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

public sealed class CompilerCliTests
{
    private static string Cli => TestEnvironment.Metadata("FpasCliPath");
    private static string Hello => Path.Combine(AppContext.BaseDirectory, "Fixtures", "hello.fpas");

    [Fact]
    public async Task The_checked_in_hello_program_compiles_and_runs_on_dotnet()
    {
        using var directory = new TemporaryDirectory();
        var output = directory.File("Hello.dll");
        var compiled = await CompileAsync(Hello, output);
        Assert.Equal(0, compiled.ExitCode);
        Assert.Equal("", compiled.Error);
        using var report = JsonDocument.Parse(compiled.Output);
        Assert.True(report.RootElement.GetProperty("success").GetBoolean());
        Assert.Equal(output, report.RootElement.GetProperty("output").GetString());
        Assert.Empty(report.RootElement.GetProperty("diagnostics").EnumerateArray());
        using var config = JsonDocument.Parse(File.ReadAllText(Path.ChangeExtension(output, ".runtimeconfig.json")));
        Assert.Equal("Microsoft.NETCore.App", config.RootElement.GetProperty("runtimeOptions").GetProperty("framework").GetProperty("name").GetString());

        var program = await ProcessRunner.RunAsync([output]);
        Assert.Equal(0, program.ExitCode);
        Assert.Equal("Hello from FPAS" + Environment.NewLine, program.Output);
        Assert.Equal("", program.Error);
    }

    [Fact]
    public async Task Stdin_nested_metadata_calls_quotes_unicode_and_int_overloads_work()
    {
        using var directory = new TemporaryDirectory();
        var output = directory.File("Strings.dll");
        var source = """
            // The comment deliberately contains 'quotes' and unused tokens.
            program Strings;
            uses System.Console as Console, System.String as Text;
            begin
              const Message: string := Text.Concat('It''s ', Text.Concat('FPAS ', '😀'));
              const Number: int := 12_345;
              Console.WriteLine(Message);
              Console.WriteLine(Number);
            end;
            end program;
            """;
        var compiled = await CompileAsync("-", output, source);
        Assert.Equal(0, compiled.ExitCode);
        var executed = await ProcessRunner.RunAsync([output]);
        Assert.Equal(0, executed.ExitCode);
        Assert.Equal("It's FPAS 😀" + Environment.NewLine + "12345" + Environment.NewLine, executed.Output);
    }

    [Fact]
    public async Task Compilation_errors_preserve_existing_output_and_return_structured_diagnostics()
    {
        using var directory = new TemporaryDirectory();
        var source = directory.File("Bad.fpas");
        File.WriteAllText(source, "program Bad; begin const Message := 'x'; end; end program;");
        var output = directory.File("Bad.dll");
        var configuration = Path.ChangeExtension(output, ".runtimeconfig.json");
        File.WriteAllText(output, "existing assembly");
        File.WriteAllText(configuration, "existing configuration");
        var result = await CompileAsync(source, output);
        Assert.Equal(1, result.ExitCode);
        using var report = JsonDocument.Parse(result.Output);
        Assert.False(report.RootElement.GetProperty("success").GetBoolean());
        Assert.Equal("FP2002", Assert.Single(report.RootElement.GetProperty("diagnostics").EnumerateArray()).GetProperty("code").GetString());
        Assert.Equal("existing assembly", File.ReadAllText(output));
        Assert.Equal("existing configuration", File.ReadAllText(configuration));
    }

    [Fact]
    public async Task Publishing_failures_restore_existing_output_files()
    {
        using var directory = new TemporaryDirectory();
        var output = directory.File("Blocked.dll");
        Directory.CreateDirectory(output);
        var configuration = Path.ChangeExtension(output, ".runtimeconfig.json");
        File.WriteAllText(configuration, "previous configuration");
        var result = await CompileAsync(Hello, output);
        Assert.Equal(1, result.ExitCode);
        using var report = JsonDocument.Parse(result.Output);
        Assert.Equal("FP0002", Assert.Single(report.RootElement.GetProperty("diagnostics").EnumerateArray()).GetProperty("code").GetString());
        Assert.True(Directory.Exists(output));
        Assert.Equal("previous configuration", File.ReadAllText(configuration));
        Assert.Empty(Directory.GetDirectories(directory.Path, ".fpas-*"));
    }

    [Fact]
    public async Task Invalid_UTF8_is_an_input_failure_without_output()
    {
        using var directory = new TemporaryDirectory();
        var source = directory.File("Invalid.fpas");
        File.WriteAllBytes(source, [0xff, 0xfe, 0x00]);
        var output = directory.File("Invalid.dll");
        var result = await CompileAsync(source, output);
        Assert.Equal(1, result.ExitCode);
        Assert.False(File.Exists(output));
        using var report = JsonDocument.Parse(result.Output);
        Assert.Equal("FP0002", Assert.Single(report.RootElement.GetProperty("diagnostics").EnumerateArray()).GetProperty("code").GetString());
    }

    [Theory]
    [InlineData("--help")]
    [InlineData("compile", "--help")]
    public async Task Help_is_noninteractive_and_includes_examples(params string[] arguments)
    {
        var result = await ProcessRunner.RunAsync(new[] { Cli }.Concat(arguments));
        Assert.Equal(0, result.ExitCode);
        Assert.Contains("Example", result.Output);
        Assert.Contains("--output", result.Output);
        Assert.Equal("", result.Error);
    }

    [Fact]
    public async Task Invalid_arguments_have_a_distinct_exit_code_and_JSON_result()
    {
        var result = await ProcessRunner.RunAsync([Cli, "compile", "--diagnostics", "json"]);
        Assert.Equal(2, result.ExitCode);
        using var report = JsonDocument.Parse(result.Output);
        Assert.False(report.RootElement.GetProperty("success").GetBoolean());
        Assert.Equal("FP0001", Assert.Single(report.RootElement.GetProperty("diagnostics").EnumerateArray()).GetProperty("code").GetString());
    }

    private static Task<ProcessResult> CompileAsync(string source, string output, string? input = null)
        => ProcessRunner.RunAsync([Cli, "compile", source, "--output", output,
            "--reference-dir", TestEnvironment.ReferenceDirectory, "--diagnostics", "json"], input);

    [Fact]
    public async Task Input_cannot_be_overwritten_by_the_runtime_configuration()
    {
        using var directory = new TemporaryDirectory();
        var output = directory.File("Hello.dll");
        var source = Path.ChangeExtension(output, ".runtimeconfig.json");
        var text = File.ReadAllText(Hello);
        File.WriteAllText(source, text);
        var result = await CompileAsync(source, output);
        Assert.Equal(2, result.ExitCode);
        Assert.Equal(text, File.ReadAllText(source));
        Assert.False(File.Exists(output));
    }

    [Fact]
    public async Task Output_requires_an_assembly_name()
    {
        using var directory = new TemporaryDirectory();
        var result = await CompileAsync(Hello, directory.File(".dll"));
        Assert.Equal(2, result.ExitCode);
        Assert.Empty(Directory.GetFiles(directory.Path));
    }

    [Fact]
    public async Task Named_routines_support_forward_calls_and_typed_parameters()
    {
        using var directory = new TemporaryDirectory();
        var output = directory.File("Routines.dll");
        var source = """
            program Routines;
            uses System.Console as Console;
            function Greet(Value: string): string;
            begin
              return Echo(Value);
            end function;
            function Echo(Value: string): string;
            begin
              return Value;
            end function;
            begin
              Console.WriteLine(Greet('Forward call'));
            end;
            end program;
            """;
        var compiled = await CompileAsync("-", output, source);
        Assert.True(compiled.ExitCode == 0, compiled.Output + compiled.Error);
        var result = await ProcessRunner.RunAsync([output]);
        Assert.Equal(0, result.ExitCode);
        Assert.Equal("Forward call" + Environment.NewLine, result.Output);
    }
}
