using Fpas.Compiler.Formatting;

namespace Fpas.Compiler.Tests.Syntax;

/// <summary>Checks that malformed and excessively nested source receives syntax diagnostics.</summary>
public sealed class ParserLimitTests
{
    /// <summary>Reports the missing name when a source ends at a type declaration keyword.</summary>
    [Fact]
    public void Incomplete_type_declaration_reports_a_missing_name()
    {
        const string text = "unit Example; type";
        var result = FpasCompiler.Compile(new([new("example.fpas", text)], "Example", [], CompilationTarget.Library));

        var diagnostic = Assert.Single(result.Diagnostics);
        Assert.Equal("FP2001", diagnostic.Code);
        Assert.Equal(text.Length, diagnostic.Location?.Offset);
        Assert.Equal("identifier", diagnostic.Expectation?.Kind);
    }

    /// <summary>Bounds parser recursion and the expression trees passed to later phases.</summary>
    /// <param name="prefix">The start of the test program.</param>
    /// <param name="open">The text repeated before the innermost expression or block.</param>
    /// <param name="body">The innermost expression or block.</param>
    /// <param name="close">The text repeated after the innermost expression or block.</param>
    [Theory]
    [InlineData("discard ", "(", "1", ")")]
    [InlineData("discard ", "not ", "true", "")]
    [InlineData("discard ", "1 + ", "1", "")]
    [InlineData("discard ", "", "Value", ".Member")]
    [InlineData("", "begin ", "begin end", "; end")]
    public void Deep_source_reports_a_nesting_diagnostic(string prefix, string open, string body, string close)
    {
        CompilationSource Source(int depth) => new("example.fpas", "program Example; begin " + prefix +
            string.Concat(Enumerable.Repeat(open, depth)) + body + string.Concat(Enumerable.Repeat(close, depth)) + "; end; end program;");

        Assert.True(FpasFormatter.Format(Source(16)).Success);
        var result = FpasFormatter.Format(Source(300));
        var diagnostic = Assert.Single(result.Diagnostics);
        Assert.Equal("FP2003", diagnostic.Code);
        Assert.Contains("nesting levels", diagnostic.Message);
        Assert.Null(result.Text);
    }
}
