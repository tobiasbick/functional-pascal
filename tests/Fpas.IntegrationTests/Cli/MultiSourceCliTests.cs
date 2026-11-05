using System.Text.Json;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Cli;

public sealed class MultiSourceCliTests
{
    [Fact]
    public async Task Stdin_program_and_mutually_recursive_units_work_in_any_input_order()
    {
        using var directory = new TemporaryDirectory();
        var even = directory.File("last-name.fpas");
        var odd = directory.File("first-name.fpas");
        File.WriteAllText(even, """
            unit Numbers.Even;
            uses Numbers.Odd as Other, Numbers.Even as Own;
            private function Zero(N: int): bool;
            begin return N = 0; end function;
            public function Test(N: int): bool;
            begin if Own.Zero(N) then return true; else return Other.Test(N - 1); end if; end function;
            end unit;
            """);
        File.WriteAllText(odd, """
            unit Numbers.Odd;
            uses Numbers.Even as Other;
            public function Test(N: int): bool;
            begin if N = 0 then return false; else return Other.Test(N - 1); end if; end function;
            end unit;
            """);
        const string program = "program App; uses Numbers.Even as E, Numbers.Odd as O, System.Console as Console; begin Console.WriteLine(E.Test(10)); Console.WriteLine(O.Test(10)); end; end program;";
        foreach (var sources in new[] { new[] { even, "-", odd }, new[] { odd, even, "-" } })
        {
            var compiled = await Compile(directory, sources, program);
            Assert.True(compiled.ExitCode == 0, compiled.Output + compiled.Error);
            var executed = await ProcessRunner.RunAsync([directory.File("Multiple.dll")]);
            Assert.True(executed.ExitCode == 0, executed.Output + executed.Error);
            Assert.Equal("True" + Environment.NewLine + "False" + Environment.NewLine, executed.Output);
        }
    }

    [Fact]
    public async Task Errors_in_secondary_sources_preserve_previous_outputs_and_report_the_correct_path()
    {
        using var directory = new TemporaryDirectory();
        var entry = directory.File("main.fpas");
        var helper = directory.File("helper.fpas");
        File.WriteAllText(entry, "program App; begin end; end program;");
        File.WriteAllText(helper, "unit Helper;\npublic function Value(): int;\nbegin\nreturn Missing;\nend function; end unit;");
        var output = directory.File("Multiple.dll");
        File.WriteAllText(output, "previous assembly");
        var result = await Compile(directory, [entry, helper]);
        Assert.Equal(1, result.ExitCode);
        using var json = JsonDocument.Parse(result.Output);
        var diagnostic = Assert.Single(json.RootElement.GetProperty("diagnostics").EnumerateArray());
        Assert.Equal("FP3001", diagnostic.GetProperty("code").GetString());
        Assert.Equal(helper, diagnostic.GetProperty("location").GetProperty("path").GetString());
        Assert.Equal(4, diagnostic.GetProperty("location").GetProperty("line").GetInt32());
        Assert.Equal("previous assembly", File.ReadAllText(output));
    }

    [Fact]
    public async Task Invalid_UTF8_in_a_secondary_file_reports_that_file()
    {
        using var directory = new TemporaryDirectory();
        var entry = directory.File("main.fpas");
        var helper = directory.File("bad.fpas");
        File.WriteAllText(entry, "program App; begin end; end program;");
        File.WriteAllBytes(helper, [0xff, 0xfe]);
        var result = await Compile(directory, [entry, helper]);
        Assert.Equal(1, result.ExitCode);
        using var json = JsonDocument.Parse(result.Output);
        var diagnostic = Assert.Single(json.RootElement.GetProperty("diagnostics").EnumerateArray());
        Assert.Equal("FP0002", diagnostic.GetProperty("code").GetString());
        Assert.Equal(JsonValueKind.Null, diagnostic.GetProperty("location").ValueKind);
        Assert.Equal(helper, diagnostic.GetProperty("inputPath").GetString());
    }

    [Theory]
    [InlineData("-", "-")]
    [InlineData("same.fpas", "./same.fpas")]
    public async Task Duplicate_sources_and_duplicate_stdin_are_argument_errors(string first, string second)
    {
        using var directory = new TemporaryDirectory();
        var result = await Compile(directory, [first, second]);
        Assert.Equal(2, result.ExitCode);
        using var json = JsonDocument.Parse(result.Output);
        Assert.Equal("FP0001", Assert.Single(json.RootElement.GetProperty("diagnostics").EnumerateArray()).GetProperty("code").GetString());
    }

    [Fact]
    public async Task No_selected_source_may_be_overwritten_by_outputs()
    {
        using var directory = new TemporaryDirectory();
        var entry = directory.File("main.fpas");
        var helper = directory.File("Multiple.dll");
        File.WriteAllText(entry, "program App; begin end; end program;");
        File.WriteAllText(helper, "unit Helper; end unit;");
        var result = await Compile(directory, [entry, helper]);
        Assert.Equal(2, result.ExitCode);
        Assert.Equal("unit Helper; end unit;", File.ReadAllText(helper));
    }

    [Theory]
    [InlineData("[null]")]
    [InlineData("{bad json}")]
    [InlineData("null")]
    [InlineData("[]")]
    public async Task Malformed_response_files_are_argument_errors(string contents)
    {
        using var directory = new TemporaryDirectory();
        var response = directory.File("arguments.json");
        File.WriteAllText(response, contents);
        var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", "--response-file", response]);
        Assert.Equal(2, result.ExitCode);
        Assert.Contains("FP0001", result.Error);
    }

    private static Task<ProcessResult> Compile(TemporaryDirectory directory, IEnumerable<string> sources, string? input = null)
        => ProcessRunner.RunAsync(new[] { TestEnvironment.Metadata("FpasCliPath"), "compile" }.Concat(sources)
            .Concat(["--output", directory.File("Multiple.dll"), "--reference-dir", TestEnvironment.ReferenceDirectory, "--diagnostics", "json"]), input);
}
