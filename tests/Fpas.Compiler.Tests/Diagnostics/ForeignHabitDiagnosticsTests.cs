using Fpas.Compiler.Analysis;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Diagnostics;

/// <summary>Checks targeted guidance for foreign syntax while preserving ordinary FPAS bindings.</summary>
public sealed class ForeignHabitDiagnosticsTests
{
    /// <summary>Reject Pascal block comments at their opener and recommend line comments.</summary>
    /// <param name="comment">Unsupported comment text inserted into the entry body.</param>
    /// <param name="opener">The expected rejected opening token.</param>
    [Theory]
    [InlineData("{ Pascal comment }", "{")]
    [InlineData("{ unfinished", "{")]
    [InlineData("(* Pascal comment *)", "(*")]
    [InlineData("(* unfinished !", "(*")]
    [InlineData("(* first\r\nsecond *)", "(*")]
    public void Pascal_comments_are_rejected_at_the_opener_with_line_comment_guidance(string comment, string opener)
    {
        var source = "program Sample;\r\nbegin\r\n  " + comment + "\r\nend;\r\nend program;";
        var diagnostic = Rejected(source, "FP1001");
        Assert.Contains("'//' line comments", diagnostic.Help);
        Assert.Equal("comment", diagnostic.Expectation!.Kind);
        Assert.Equal(["//"], diagnostic.Expectation.Expected); Assert.Equal(opener, diagnostic.Expectation.Found);
        Assert.Equal(source.IndexOf(opener, StringComparison.Ordinal), diagnostic.Location!.Offset);
        Assert.Equal(3, diagnostic.Location.Line); Assert.Equal(3, diagnostic.Location.Column);
        Assert.Equal(opener.Length, diagnostic.Location.Length); Assert.Empty(diagnostic.Fixes);
        Assert.True(Compile("program Sample; begin // Pascal comment\nend; end program;").Success);
    }

    /// <summary>Preserve comment-like and operator-like text inside literals and line comments.</summary>
    [Fact]
    public void Foreign_comment_and_operator_spellings_inside_strings_or_line_comments_remain_data()
    {
        var result = Compile("program Sample; begin const Text: string := '{ not a comment } (* /= *)'; // (* { /=\nend; end program;");
        Assert.True(result.Success); Assert.Empty(result.Diagnostics);
    }

    /// <summary>Offer an exact, compilable correction for adjacent Ada inequality tokens.</summary>
    /// <param name="expression">An expression containing the unsupported operator.</param>
    [Theory]
    [InlineData("1 /= 2")]
    [InlineData("1/=2")]
    [InlineData("'😀' /= 'text'")]
    public void Ada_inequality_exposes_an_exact_edit_that_compiles(string expression)
    {
        var source = "program Sample; begin discard " + expression + "; end; end program;";
        var diagnostic = Rejected(source, "FP1001");
        Assert.Contains("'<>'", diagnostic.Help);
        Assert.Equal("operator", diagnostic.Expectation!.Kind);
        Assert.Equal(["<>"], diagnostic.Expectation.Expected); Assert.Equal("/=", diagnostic.Expectation.Found);
        var fix = Assert.Single(diagnostic.Fixes);
        Assert.Equal(DiagnosticApplicability.Automatic, fix.Applicability);
        var edit = Assert.Single(fix.Edits);
        Assert.Equal("/=", edit.ExpectedText); Assert.Equal("<>", edit.NewText);
        Assert.Equal(source.IndexOf("/=", StringComparison.Ordinal), edit.Location.Offset);
        Assert.Equal(2, edit.Location.Length);
        var corrected = source.Remove(edit.Location.Offset!.Value, edit.Location.Length).Insert(edit.Location.Offset.Value, edit.NewText);
        Assert.True(Compile(corrected).Success);
    }

    /// <summary>Keep separated division and equality tokens outside the Ada correction.</summary>
    [Fact]
    public void Separate_division_and_equality_tokens_do_not_receive_an_Ada_operator_fix()
    {
        var diagnostic = Rejected("program Sample; begin discard 1 / = 2; end; end program;", "FP2003");
        Assert.Null(diagnostic.Help); Assert.Empty(diagnostic.Fixes);
    }

    /// <summary>Explain the entry-body and program closers without guessing structural edits.</summary>
    [Fact]
    public void Pascal_program_ending_explains_both_required_FPAs_closers_without_a_structural_edit()
    {
        const string source = "program Sample; begin end.";
        var diagnostic = Rejected(source, "FP2001");
        Assert.Contains("'end;'", diagnostic.Help); Assert.Contains("'end program;'", diagnostic.Help);
        Assert.Equal(source.Length - 1, diagnostic.Location!.Offset);
        Assert.Empty(diagnostic.Fixes);
        Assert.True(Compile("program Sample; begin end; end program;").Success);
    }

    /// <summary>Keep nested block errors separate from program-ending guidance.</summary>
    /// <param name="source">A source containing an invalid nested block terminator.</param>
    [Theory]
    [InlineData("unit Sample; procedure Run(); begin begin end. end procedure; end unit;")]
    [InlineData("program Sample; begin begin end. end; end program;")]
    public void Nested_blocks_do_not_receive_program_entry_ending_guidance(string source)
    {
        var diagnostic = Rejected(source, "FP2001");
        Assert.Null(diagnostic.Help); Assert.Empty(diagnostic.Fixes);
    }

    /// <summary>Explain the required source container for an Ada-style main procedure.</summary>
    /// <param name="source">An unsupported Ada-style source container.</param>
    [Theory]
    [InlineData("procedure Main is begin null; end Main;")]
    [InlineData("procedure Main(Count: Integer) is begin null; end Main;")]
    [InlineData("Procedure Main is begin null; end Main;")]
    public void Ada_main_procedures_explain_the_FPAs_program_container_without_guessing_a_rewrite(string source)
    {
        var diagnostic = Rejected(source, "FP2001");
        Assert.Contains("'program Name;'", diagnostic.Help); Assert.Contains("'begin ... end; end program;'", diagnostic.Help);
        Assert.Equal(["program", "unit"], diagnostic.Expectation!.Expected);
        Assert.Empty(diagnostic.Fixes);
        Assert.True(Compile("program Main; procedure Helper(); begin end procedure; begin Helper(); end; end program;").Success);
    }

    /// <summary>Recommend explicit return for an undeclared Delphi-style result assignment.</summary>
    /// <param name="name">The undeclared result identifier spelling.</param>
    [Theory]
    [InlineData("Result")]
    [InlineData("result")]
    [InlineData("RESULT")]
    public void Undeclared_Delphi_result_targets_in_functions_explain_explicit_return_without_changing_control_flow(string name)
    {
        var diagnostic = Rejected(Function(name + " := 1;"), "FP3009");
        Assert.Contains("'return Value;'", diagnostic.Help); Assert.Contains("control flow", diagnostic.Help);
        Assert.Equal(name.Length, diagnostic.Location!.Length); Assert.Empty(diagnostic.Fixes);
        Assert.True(Compile(Function("return 1;")).Success);
    }

    /// <summary>Accept explicitly declared mutable result storage.</summary>
    /// <param name="body">A function body using ordinary result-named storage.</param>
    [Theory]
    [InlineData("var Result: int := 0; Result := 1; return Result;")]
    [InlineData("var result: int := 0; result := 1; return result;")]
    [InlineData("var RESULT: int := 0; RESULT := 1; return RESULT;")]
    public void Declared_mutable_result_names_remain_normal_storage(string body)
    {
        var result = Compile(Function(body));
        Assert.True(result.Success); Assert.Empty(result.Diagnostics);
    }

    /// <summary>Preserve normal errors for entry bodies, procedures, escapes and read-only storage.</summary>
    /// <param name="source">A source where return-value advice is inappropriate.</param>
    [Theory]
    [InlineData("program Sample; begin Result := 1; end; end program;")]
    [InlineData("program Sample; procedure Run(); begin Result := 1; end procedure; begin Run(); end; end program;")]
    [InlineData("program Sample; function Answer(): int; begin @Result := 1; end function; begin discard Answer(); end; end program;")]
    [InlineData("program Sample; function Answer(): int; begin const Result: int := 0; Result := 1; return Result; end function; begin discard Answer(); end; end program;")]
    public void Entry_procedures_and_readonly_result_storage_do_not_receive_a_return_value_hint(string source)
    {
        var diagnostic = Rejected(source, "FP3009");
        Assert.Null(diagnostic.Help); Assert.Empty(diagnostic.Fixes);
    }

    /// <summary>Suppress undeclared-result advice for declared callable, member and type names.</summary>
    /// <param name="source">A source with a declared non-writable result identifier.</param>
    [Theory]
    [InlineData("program Sample; function Result(): int; begin return 0; end function; function Answer(): int; begin Result := 1; end function; begin discard Answer(); end; end program;")]
    [InlineData("program Sample; type Box = record public function Result(): int; begin return 0; end function; public function Answer(): int; begin Result := 1; end function; end record; begin end; end program;")]
    [InlineData("program Sample; type Box = record public Result: int := 0; public function Answer(): int; begin Result := 1; end function; end record; begin end; end program;")]
    [InlineData("program Sample; uses System.Int32 as Result; function Answer(): int; begin Result := 1; end function; begin discard Answer(); end; end program;")]
    [InlineData("program Sample; type Result = int; function Answer(): int; begin Result := 1; end function; begin discard Answer(); end; end program;")]
    [InlineData("program Sample; function Answer<Result>(): int; begin Result := 1; end function; begin discard Answer<int>(); end; end program;")]
    public void Declared_routines_members_and_types_do_not_receive_an_undeclared_result_hint(string source)
    {
        var diagnostic = Rejected(source, "FP3009");
        Assert.Null(diagnostic.Help); Assert.Empty(diagnostic.Fixes);
    }

    private static string Function(string body)
        => "program Sample; function Answer(): int; begin " + body + " end function; begin discard Answer(); end; end program;";

    private static CompilerDiagnostic Rejected(string source, string code)
    {
        var result = Compile(source);
        Assert.False(result.Success); Assert.Null(result.Program);
        var diagnostic = Assert.Single(result.Diagnostics);
        Assert.Equal(code, diagnostic.Code);
        var analysis = FpasCompiler.Analyze(new([new("sample.fpas", source)], "Sample", [.. TestEnvironment.References, typeof(Core.RecordAttribute).Assembly.Location]));
        Assert.False(analysis.Success);
        var checkedDiagnostic = Assert.Single(analysis.Diagnostics);
        Assert.Equal(diagnostic.Code, checkedDiagnostic.Code); Assert.Equal(diagnostic.Location, checkedDiagnostic.Location);
        Assert.Equal(diagnostic.Help, checkedDiagnostic.Help);
        return diagnostic;
    }

    private static CompilationResult Compile(string source)
        => FpasCompiler.Compile(new([new("sample.fpas", source)], "Sample", [.. TestEnvironment.References, typeof(Core.RecordAttribute).Assembly.Location]));
}
