// [Documentation](../../../docs/pascal/tools/diagnostics.md#call-diagnostics)
using Fpas.Compiler.Tests.Support;

namespace Fpas.Compiler.Tests.Routines;

public sealed class CallDiagnosticsTests
{
    [Fact]
    public void Clr_byref_mismatch_marks_the_actual_argument_and_reports_its_passing_mode()
    {
        const string source = """
            program Example;
            begin
              const Text: string := '42';
              var Number: int := 0;
              discard int.TryParse(Text, Number);
            end;
            end program;
            """;
        var diagnostic = Assert.Single(TestCompilation.Compile(source).Diagnostics);
        Assert.Equal("FP3003", diagnostic.Code);
        Assert.Contains("Argument 2 ('Number') expects out int. Received int.", diagnostic.Message);
        var argument = Assert.IsType<DiagnosticArgument>(diagnostic.Argument);
        Assert.Equal(2, argument.Index);
        Assert.Null(argument.Name);
        Assert.Equal("int", argument.Type);
        Assert.Equal("value", argument.Mode);
        var parameter = Assert.Single(argument.Parameters);
        Assert.Equal(("result", "int", "out"), (parameter.Name, parameter.Type, parameter.Mode));
        var location = Assert.IsType<SourceLocation>(diagnostic.Location);
        Assert.Equal("Number", source.Substring(location.Offset!.Value, location.Length));
    }

    [Fact]
    public void Equally_close_overloads_report_their_parameter_alternatives()
    {
        var diagnostic = Check("""
            procedure Take(Value: string); begin end procedure;
            procedure Take(Value: bool); begin end procedure;
            """, "Take(Value := 1);");
        Assert.Equal("FP3003", diagnostic.Code);
        var argument = Assert.IsType<DiagnosticArgument>(diagnostic.Argument);
        Assert.Equal("Value", argument.Name);
        Assert.Equal("int", argument.Type);
        Assert.Equal(new[] { "bool", "string" }, argument.Parameters.Select(item => item.Type).Order(StringComparer.Ordinal));
    }

    [Fact]
    public void Ambiguity_lists_only_the_equally_applicable_signatures()
    {
        var diagnostic = Check("""
            procedure Take(Left: long; Right: int); begin end procedure;
            procedure Take(Left: int; Right: long); begin end procedure;
            procedure Take(Value: string); begin end procedure;
            """, "Take(1, 1);");
        Assert.Equal("FP3003", diagnostic.Code);
        var expectation = Assert.IsType<DiagnosticExpectation>(diagnostic.Expectation);
        Assert.Equal("ambiguous call", expectation.Found);
        Assert.Equal(new[] { "Take(Left: int; Right: long)", "Take(Left: long; Right: int)" }, expectation.Expected);
    }

    [Theory]
    [InlineData("Take(Unknown := 1);", "parameter_name", "Unknown", "First")]
    [InlineData("Take(First := 1, First := 2);", "duplicate_argument", "First", "First")]
    [InlineData("Take(First := 1, 2);", "argument_order", "positional argument", "named argument")]
    [InlineData("Take(First := 1);", "missing_argument", "omitted", "Second")]
    [InlineData("Take(1, 2, 3);", "argument_count", "3", "2")]
    public void Argument_layout_errors_describe_the_failed_rule(string body, string kind, string found, string expected)
    {
        var diagnostic = Check("procedure Take(First: int; Second: int); begin end procedure;", body);
        Assert.Equal("FP3003", diagnostic.Code);
        AssertExpectation(diagnostic, kind, found, expected);
    }

    [Fact]
    public void Generic_errors_distinguish_inference_count_and_constraint()
    {
        const string declaration = "procedure Pick<T>(); where T: class; begin end procedure;";
        AssertExpectation(Check(declaration, "Pick();"), "type_arguments", "inference failed", "Pick<T>()");
        AssertExpectation(Check(declaration, "Pick<int, int>();"), "type_argument_count", "2", "1");
        var diagnostic = Check(declaration, "Pick<int>();");
        Assert.Equal("FP3018", diagnostic.Code);
        Assert.Contains("for 'T' must satisfy 'class'", diagnostic.Message);
        AssertExpectation(diagnostic, "generic_constraint", "int", "class");
    }

    [Fact]
    public void Data_construction_marks_an_unknown_field_and_names_the_actual_fields()
    {
        var diagnostic = Check("type Point = record public X: int; end record;", "discard new Point(Y := 1);");
        Assert.Equal("FP3022", diagnostic.Code);
        AssertExpectation(diagnostic, "field_name", "Y", "X");
    }

    [Fact]
    public void Passing_an_expression_with_in_explains_where_to_store_it()
    {
        var diagnostic = Check("procedure Take(in Value: int); begin end procedure;", "Take(in 1 + 2);");
        Assert.Equal("FP3028", diagnostic.Code);
        Assert.Contains("typed local", diagnostic.Help);
        AssertExpectation(diagnostic, "storage", "non-addressable expression", "addressable storage");
    }

    [Theory]
    [InlineData("procedure Read(); begin end procedure;", "discard Read();", "void", "value")]
    [InlineData("function Read(): int; begin return 0; end function;", "Read();", "int", "void")]
    public void Discard_and_unused_results_have_distinct_expectations(string declaration, string body, string actual, string expected)
    {
        var diagnostic = Check(declaration, body);
        Assert.Equal("FP3006", diagnostic.Code);
        AssertExpectation(diagnostic, "expression_result", actual, expected);
    }

    private static CompilerDiagnostic Check(string declaration, string body)
        => Assert.Single(TestCompilation.Compile($"program Example; {declaration} begin {body} end; end program;").Diagnostics);

    private static void AssertExpectation(CompilerDiagnostic diagnostic, string kind, string found, string expected)
    {
        var expectation = Assert.IsType<DiagnosticExpectation>(diagnostic.Expectation);
        Assert.Equal(kind, expectation.Kind);
        Assert.Equal(found, expectation.Found);
        Assert.Contains(expected, expectation.Expected);
    }
}
