using Fpas.Compiler.Analysis;
using Fpas.Compiler.Syntax;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Diagnostics;

public sealed class RichDiagnosticsTests
{
    [Fact]
    public void Syntax_expectations_and_safe_edits_use_original_UTF16_coordinates()
    {
        const string source = "program Sample;\r\nbegin\r\n  const Text: string := '😀';\r\n  const Count: int = 1;\r\nend;\r\nend program;";
        var diagnostic = Assert.Single(Compile(source).Diagnostics);
        Assert.Equal("FP2001", diagnostic.Code);
        Assert.Equal("token", diagnostic.Expectation!.Kind);
        Assert.Equal([":="], diagnostic.Expectation.Expected);
        Assert.Equal("=", diagnostic.Expectation.Found);
        var location = Assert.IsType<SourceLocation>(diagnostic.Location);
        Assert.Equal(4, location.Line); Assert.Equal(20, location.Column);
        Assert.Equal(4, location.EndLine); Assert.Equal(21, location.EndColumn);
        Assert.Equal(source.IndexOf("= 1", StringComparison.Ordinal), location.Offset);
        var fix = Assert.Single(diagnostic.Fixes);
        Assert.Equal(DiagnosticApplicability.Automatic, fix.Applicability);
        var edit = Assert.Single(fix.Edits);
        Assert.Equal("=", edit.ExpectedText); Assert.Equal(":=", edit.NewText);
        var corrected = source.Remove(edit.Location.Offset!.Value, edit.Location.Length).Insert(edit.Location.Offset.Value, edit.NewText);
        Assert.True(Compile(corrected).Success);
    }

    [Theory]
    [InlineData("integer", "int", "1")]
    [InlineData("Integer", "int", "1")]
    [InlineData("boolean", "bool", "true")]
    [InlineData("real", "double", "1.5")]
    [InlineData("ansistring", "string", "'text'")]
    public void Foreign_type_hints_have_validated_explicit_replacements(string foreign, string canonical, string value)
    {
        var source = $"program Sample; begin const Value: {foreign} := {value}; end; end program;";
        var diagnostic = Assert.Single(Compile(source).Diagnostics);
        Assert.Equal("FP3005", diagnostic.Code);
        Assert.Equal([canonical], diagnostic.Expectation!.Expected);
        var edit = Assert.Single(Assert.Single(diagnostic.Fixes).Edits);
        Assert.Equal(foreign, edit.ExpectedText);
        Assert.True(Compile(source.Remove(edit.Location.Offset!.Value, foreign.Length).Insert(edit.Location.Offset.Value, edit.NewText)).Success);
    }

    [Fact]
    public void Conversion_data_and_related_bindings_do_not_require_message_parsing()
    {
        var conversion = Assert.Single(Compile("program Sample; begin const Value: int := 'text'; end; end program;").Diagnostics);
        Assert.Equal("type", conversion.Expectation!.Kind);
        Assert.Equal(["int"], conversion.Expectation.Expected); Assert.Equal("string", conversion.Expectation.Found);
        Assert.NotNull(conversion.Help);
        const string source = "program Sample; begin const Value: int := 1; const Value: int := 2; end; end program;";
        var duplicate = Assert.Single(Compile(source).Diagnostics);
        var previous = Assert.Single(duplicate.Related);
        Assert.Equal(source.IndexOf("Value", StringComparison.Ordinal), previous.Location!.Offset);
        Assert.Equal(source.LastIndexOf("Value", StringComparison.Ordinal), duplicate.Location!.Offset);
    }

    [Theory]
    [InlineData("Program Sample; begin end; end program;", "Program", "program")]
    [InlineData("program Sample; begin if true then discard 1; elseif false then discard 2; end if; end; end program;", "elseif", "elsif")]
    [InlineData("program Sample; begin discard 1 != 2; end; end program;", "!=", "<>")]
    public void Canonical_spelling_hints_propose_edits_that_compile(string source, string oldText, string canonical)
    {
        var diagnostic = Assert.Single(Compile(source).Diagnostics);
        var edit = Assert.Single(Assert.Single(diagnostic.Fixes).Edits);
        Assert.Equal(oldText, edit.ExpectedText); Assert.Equal(canonical, edit.NewText);
        Assert.True(Compile(source.Remove(edit.Location.Offset!.Value, edit.Location.Length).Insert(edit.Location.Offset.Value, edit.NewText)).Success);
    }

    [Fact]
    public void Record_methods_have_isolated_recovery_and_semantic_errors_share_the_limit()
    {
        const string record = "program Sample; type Point = record public procedure First(); begin discard Missing; end procedure; public procedure Second(); begin break; end procedure; end record; begin end; end program;";
        Assert.Equal(["FP3001", "FP3010"], Compile(record).Diagnostics.Select(diagnostic => diagnostic.Code));
        var source = "program Sample; " + string.Concat(Enumerable.Range(0, 100).Select(index => $"procedure P{index}(); begin break; end procedure; ")) + "begin end; end program;";
        var result = Compile(source);
        Assert.Equal(50, result.Diagnostics.Count); Assert.Equal("FP0003", result.Diagnostics[^1].Code); Assert.Null(result.Program);
    }

    [Theory]
    [InlineData("\n")]
    [InlineData("\r\n")]
    [InlineData("\r")]
    public void Multiline_ranges_are_half_open_and_preserve_line_endings(string newline)
    {
        var source = new SourceText("sample.fpas", "😀X" + newline + "Yé");
        var range = source.Location(2, 1 + newline.Length + 1);
        Assert.Equal(new SourceLocation("sample.fpas", 1, 3, 2 + newline.Length, 2, 2, 2), range);
        var end = source.Location(source.Text.Length, 1);
        Assert.Equal(0, end.Length); Assert.Equal(2, end.EndLine); Assert.Equal(3, end.EndColumn);
    }

    [Fact]
    public void Syntax_recovery_finds_independent_errors_and_exposes_only_checked_snapshot_regions()
    {
        const string source = """
            program Sample;
            begin
              if true then
                const First := 1;
                const Second: int = 2;
              end if;
              const Third := 3;
            end;
            end program;
            """;
        var compiled = Compile(source);
        Assert.Null(compiled.Program);
        Assert.Equal(["FP2002", "FP2001", "FP2002"], compiled.Diagnostics.Select(diagnostic => diagnostic.Code));
        var analyzed = FpasCompiler.Analyze(new([new("sample.fpas", source)], "Sample", TestEnvironment.References));
        Assert.False(analyzed.IsComplete); Assert.NotNull(analyzed.Snapshot);
        Assert.Empty(analyzed.Snapshot.Bodies); Assert.False(analyzed.Snapshot.Coverage.IsComplete);
        Assert.DoesNotContain(analyzed.Snapshot.Symbols, symbol => symbol.Name is "First" or "Second" or "Third");
        Assert.Equal(compiled.Diagnostics.Select(diagnostic => diagnostic.Code), analyzed.Diagnostics.Select(diagnostic => diagnostic.Code));
    }

    [Fact]
    public void A_failed_routine_does_not_poison_independent_routines_or_entry_flow()
    {
        const string source = """
            program Sample;
            procedure First();
            begin
              while true do
                const Value: int := Missing;
              end while;
            end procedure;
            procedure Second();
            begin
              const Value: int := 'text';
            end procedure;
            begin
              break;
            end;
            end program;
            """;
        var result = Compile(source);
        Assert.Null(result.Program);
        Assert.Equal(["FP3001", "FP3002", "FP3010"], result.Diagnostics.Select(diagnostic => diagnostic.Code));
    }

    [Fact]
    public void An_invalid_local_binding_does_not_generate_dependent_name_errors()
    {
        var result = Compile("program Sample; begin const Broken: int := 'text'; discard Broken + Missing; end; end program;");
        Assert.Equal("FP3002", Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Files_are_recovered_independently_in_deterministic_order()
    {
        var result = FpasCompiler.Compile(new([new("z.fpas", "unit Z; ???"), new("a.fpas", "NotAUnit"), new("m.fpas", "unit M; begin end.")],
            "Sample", TestEnvironment.References, CompilationTarget.Library));
        Assert.Null(result.Program);
        Assert.Equal(["a.fpas", "m.fpas", "z.fpas"], result.Diagnostics.Select(diagnostic => diagnostic.Location!.Path));
    }

    [Fact]
    public void Error_limit_is_explicit_bounded_and_identical_for_compile_and_analysis()
    {
        var source = "program Sample; begin " + string.Concat(Enumerable.Range(0, 100).Select(index => $"const Value{index} := {index}; ")) + "end; end program;";
        var compiled = Compile(source);
        Assert.Equal(50, compiled.Diagnostics.Count); Assert.Null(compiled.Program);
        Assert.Equal("FP0003", compiled.Diagnostics[^1].Code);
        var analysis = FpasCompiler.Analyze(new([new("sample.fpas", source)], "Sample", TestEnvironment.References));
        Assert.Equal(50, analysis.Diagnostics.Count); Assert.False(analysis.IsComplete); Assert.Null(analysis.Snapshot);
    }

    [Fact]
    public void Promoted_documentation_errors_share_the_limit_without_capping_ordinary_warnings()
    {
        var source = "unit Sample;\n// ## Parameters\n" + string.Concat(Enumerable.Repeat("// - `Missing`: No such parameter.\n", 100)) +
            "public function Read(Value: int): int; begin return Value; end function; end unit;";
        var request = new CompilationRequest([new("sample.fpas", source)], "Sample", TestEnvironment.References, CompilationTarget.Library, EmitDocumentation: true);
        var warnings = FpasCompiler.Compile(request);
        Assert.True(warnings.Success); Assert.True(warnings.Diagnostics.Count > 50);
        var errors = FpasCompiler.Compile(request with { DiagnosticOptions = new(TreatWarningsAsErrors: true) });
        Assert.Null(errors.Program); Assert.Equal(50, errors.Diagnostics.Count); Assert.Equal("FP0003", errors.Diagnostics[^1].Code);
        Assert.All(errors.Diagnostics, diagnostic => Assert.Equal(DiagnosticSeverity.Error, diagnostic.Severity));
    }

    [Fact]
    public void Reference_failures_have_no_invented_source_location()
    {
        var result = FpasCompiler.Compile(new([new("sample.fpas", "program Sample; begin end; end program;")], "Sample", []));
        // Missing framework metadata is a setup failure, not an error at the program header.
        Assert.Equal("FP4001", Assert.Single(result.Diagnostics).Code);
        Assert.Null(result.Diagnostics[0].Location);
    }

    [Theory]
    [InlineData("const Number: long := 2147483648;", "-2147483648..2147483647", "2147483648")]
    [InlineData("const Letter: char := #70000;", "0..65535", "70000")]
    [InlineData("const Number: double := 1e999;", "finite double", "1e999")]
    public void Numeric_value_expectations_retain_the_selected_literal_representation(string body, string expected, string found)
    {
        var diagnostic = Assert.Single(Compile("program Sample; begin " + body + " end; end program;").Diagnostics);
        Assert.Equal("value", diagnostic.Expectation!.Kind); Assert.Equal([expected], diagnostic.Expectation.Expected); Assert.Equal(found, diagnostic.Expectation.Found);
        Assert.Empty(diagnostic.Fixes);
    }

    [Theory]
    [InlineData("WriteLn('text');", "Console.WriteLine")]
    [InlineData("discard ReadLn();", "Console.ReadLine")]
    [InlineData("Exit();", "return")]
    public void Contextual_hints_do_not_fabricate_import_or_return_edits(string body, string help)
    {
        var diagnostic = Assert.Single(Compile("program Sample; begin " + body + " end; end program;").Diagnostics);
        Assert.Contains(help, diagnostic.Help); Assert.Empty(diagnostic.Fixes);
    }

    [Theory]
    [InlineData("elseif")]
    [InlineData("writeln")]
    [InlineData("exit")]
    [InlineData("integer")]
    public void Hints_never_reserve_valid_identifiers_or_reject_resolved_user_routines(string name)
    {
        var source = $"program Sample; procedure {name}(); begin end procedure; begin if true then {name}(); end if; end; end program;";
        var result = Compile(source);
        Assert.True(result.Success, string.Join('\n', result.Diagnostics.Select(diagnostic => diagnostic.Message)));
        Assert.Empty(result.Diagnostics);
    }

    [Theory]
    [InlineData("program Sample; begin if true then discard 1 else discard 2; end if; end; end program;")]
    [InlineData("program Sample; begin const Value: int := 1 procedure Helper(); begin end procedure; Helper(); end; end program;")]
    [InlineData("program Sample; begin end; end program")]
    public void Separator_recovery_preserves_the_next_branch_or_nested_routine_and_proposes_an_insertion(string source)
    {
        var diagnostic = Assert.Single(Compile(source).Diagnostics);
        Assert.Equal("FP2001", diagnostic.Code);
        var edit = Assert.Single(Assert.Single(diagnostic.Fixes).Edits);
        Assert.Equal(0, edit.Location.Length); Assert.Equal("", edit.ExpectedText); Assert.Equal(";", edit.NewText);
        Assert.True(Compile(source.Insert(edit.Location.Offset!.Value, edit.NewText)).Success);
    }

    private static CompilationResult Compile(string source)
        => FpasCompiler.Compile(new([new("sample.fpas", source)], "Sample", [.. TestEnvironment.References, typeof(Core.RecordAttribute).Assembly.Location]));
}
