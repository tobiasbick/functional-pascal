using Fpas.Tests.Support;
using System.Text.Json;

namespace Fpas.IntegrationTests.Diagnostics;

public sealed class DiagnosticCliTests
{
    /// <summary>Publish foreign-syntax guidance as JSON while preserving invalid sources and prior outputs.</summary>
    /// <param name="text">The rejected source text.</param>
    /// <param name="help">The expected diagnostic guidance fragment.</param>
    [Theory]
    [InlineData("program Example; begin discard 1 /= 2; end; end program;", "'<>'")]
    [InlineData("program Example; begin { comment } end; end program;", "'//' line comments")]
    [InlineData("program Example; begin (* comment *) end; end program;", "'//' line comments")]
    [InlineData("program Example; begin end.", "'end program;'")]
    [InlineData("procedure Main is begin null; end Main;", "'program Name;'")]
    [InlineData("program Example; function Answer(): int; begin Result := 1; end function; begin discard Answer(); end; end program;", "'return Value;'")]
    public async Task Foreign_habit_guidance_reaches_JSON_without_accepting_the_source_or_replacing_outputs(string text, string help)
    {
        using var directory = new TemporaryDirectory();
        var source = directory.File("Example.fpas"); var output = directory.File("Example.dll");
        File.WriteAllText(source, text); File.WriteAllText(output, "retained assembly");
        var result = await Compile(source, output, "--diagnostics", "json");
        Assert.Equal(1, result.ExitCode); Assert.Empty(result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.False(json.RootElement.GetProperty("success").GetBoolean());
        Assert.Equal(JsonValueKind.Null, json.RootElement.GetProperty("output").ValueKind);
        var diagnostic = Assert.Single(json.RootElement.GetProperty("diagnostics").EnumerateArray());
        Assert.Contains(help, diagnostic.GetProperty("help").GetString());
        Assert.Equal(text, File.ReadAllText(source)); Assert.Equal("retained assembly", File.ReadAllText(output));
    }

    [Fact]
    public async Task Json_exposes_a_complete_correction_and_never_publishes_an_invalid_program()
    {
        using var directory = new TemporaryDirectory();
        var source = directory.File("Example.fpas"); var output = directory.File("Example.dll");
        File.WriteAllText(source, "program Example; begin const Value: integer := 1; end; end program;");
        File.WriteAllText(output, "retained assembly");
        var result = await Compile(source, output, "--diagnostics", "json");
        Assert.Equal(1, result.ExitCode); Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output);
        var diagnostic = Assert.Single(json.RootElement.GetProperty("diagnostics").EnumerateArray());
        Assert.Equal("error", diagnostic.GetProperty("severity").GetString());
        Assert.Equal("int", diagnostic.GetProperty("expectation").GetProperty("expected")[0].GetString());
        Assert.Equal("integer", diagnostic.GetProperty("expectation").GetProperty("found").GetString());
        var fix = Assert.Single(diagnostic.GetProperty("fixes").EnumerateArray());
        Assert.Equal("automatic", fix.GetProperty("applicability").GetString());
        var edit = Assert.Single(fix.GetProperty("edits").EnumerateArray());
        Assert.Equal("integer", edit.GetProperty("expectedText").GetString());
        Assert.Equal("int", edit.GetProperty("newText").GetString());
        Assert.True(edit.GetProperty("location").GetProperty("endColumn").GetInt32() > edit.GetProperty("location").GetProperty("column").GetInt32());
        Assert.Equal("retained assembly", File.ReadAllText(output));
    }

    [Fact]
    public async Task Text_output_aligns_tabs_graphemes_and_wide_characters_without_redirected_color()
    {
        using var directory = new TemporaryDirectory();
        var source = directory.File("Unicode.fpas");
        const string line = "\tconst Text: string := '😀漢漢é'; const Number: int = 1;";
        File.WriteAllText(source, "program Example;\nbegin\n" + line + "\nend;\nend program;");
        var result = await Compile(source, directory.File("Example.dll"));
        Assert.Equal(1, result.ExitCode); Assert.Equal("", result.Output);
        Assert.Contains("error FP2001", result.Error); Assert.Contains("help:", result.Error); Assert.Contains("fix:", result.Error);
        Assert.Contains(" 3 |     const Text: string := '😀漢漢é'; const Number: int = 1;", result.Error);
        var visualColumn = line.IndexOf("= 1", StringComparison.Ordinal) + 4;
        Assert.Contains("   | " + new string(' ', visualColumn) + "^", result.Error);
        Assert.DoesNotContain("\u001b", result.Error, StringComparison.Ordinal);
    }

    [Fact]
    public async Task Long_lines_are_cropped_around_the_marker_and_control_characters_are_escaped()
    {
        using var directory = new TemporaryDirectory();
        var source = directory.File("Example.fpas"); var output = directory.File("Example.dll");
        File.WriteAllText(source, "program Example; begin const Text: string := '" + new string('x', 500) + "'; const Number: int = 1; end; end program;");
        var longLine = await Compile(source, output);
        Assert.Equal(1, longLine.ExitCode);
        var excerpt = Assert.Single(longLine.Error.Split('\n'), line => line.StartsWith(" 1 |", StringComparison.Ordinal));
        Assert.Contains("…", excerpt); Assert.Contains("const Number", excerpt); Assert.True(excerpt.Length < 135);
        File.WriteAllText(source, "program Example; begin \u001b end; end program;");
        var control = await Compile(source, output);
        Assert.Equal(1, control.ExitCode); Assert.DoesNotContain("\u001b", control.Error, StringComparison.Ordinal); Assert.Contains("\\u001B", control.Error);
    }

    [Fact]
    public async Task Non_source_failures_use_null_location_and_a_single_clean_json_result()
    {
        using var directory = new TemporaryDirectory();
        var result = await Compile(directory.File("Missing.fpas"), directory.File("Example.dll"), "--diagnostics", "json");
        Assert.Equal(1, result.ExitCode); Assert.Equal("", result.Error);
        using var json = JsonDocument.Parse(result.Output);
        var diagnostic = Assert.Single(json.RootElement.GetProperty("diagnostics").EnumerateArray());
        Assert.Equal("FP0002", diagnostic.GetProperty("code").GetString());
        Assert.Equal(JsonValueKind.Null, diagnostic.GetProperty("location").ValueKind);
    }

    private static Task<ProcessResult> Compile(string source, string output, params string[] options)
        => ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", source, "--output", output,
            "--reference-dir", TestEnvironment.Metadata("FpasReferenceDirectory"), .. options]);
}
