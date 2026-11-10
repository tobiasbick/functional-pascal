using Fpas.Compiler.Tests.Support;
namespace Fpas.Compiler.Tests.Types;

/// <summary>Checks variant declarations, constructors and exhaustive payload patterns.</summary>
public sealed class VariantTests
{
    private const string Header = "program Example; type Choice = variant A; B(Value: int); end variant; ";

    /// <summary>Accepts one canonical pair of keywords for a variant declaration.</summary>
    /// <param name="declaration">A declaration using an unsupported spelling or terminator.</param>
    [Theory]
    [InlineData("type Choice = union A; end union;")]
    [InlineData("type Choice = variant A; end union;")]
    [InlineData("type Choice = variant record A; end variant;")]
    [InlineData("type Choice = variant A; end variant record;")]
    public void Only_variant_and_end_variant_form_a_variant_declaration(string declaration)
    {
        var result = TestCompilation.Compile("program Spelling; " + declaration + " begin end; end program;");
        Assert.False(result.Success);
        Assert.Single(result.Diagnostics);
    }

    /// <summary>Reserves variant while permitting union as an ordinary source identifier.</summary>
    [Fact]
    public void Variant_is_reserved_and_union_is_an_ordinary_identifier()
    {
        Assert.Equal("@variant", Compiler.Syntax.IdentifierFacts.Spell("variant"));
        Assert.Equal("union", Compiler.Syntax.IdentifierFacts.Spell("union"));
        var result = TestCompilation.Compile("""
            program Names;
            type union = variant Empty; Value(Number: int); end variant;
            begin
              const @variant: union := union.Value(42);
              discard case @variant of
                when union.Empty: 0;
                when union.Value(const Number: int): Number;
              end case;
            end;
            end program;
            """);
        Assert.True(result.Success, string.Join(";", result.Diagnostics));
    }

    [Theory]
    [InlineData("const C: Choice := Choice.A();", "FP3023")]
    [InlineData("const C: Choice := Choice.B();", "FP3022")]
    [InlineData("const C: Choice := new Choice();", "FP3005")]
    [InlineData("const C: Choice := Choice.B(1); discard case C of when Choice.A: 0; end case;", "FP3025")]
    [InlineData("discard case Choice.A of when Choice.A: 0; when Choice.A: 1; when Choice.B(_): 2; end case;", "FP3025")]
    [InlineData("discard case Choice.A of when Choice.A: 0; when Choice.B: 1; end case;", "FP3024")]
    [InlineData("discard case Choice.A of when Choice.A: 0; when Choice.B(const V: double): 1; end case;", "FP3024")]
    [InlineData("discard case Choice.A of when Choice.A: 0; when Choice.B(const V: int) if V > 0: 1; end case;", "FP3025")]
    [InlineData("discard case Choice.A of when Choice.A: 0; when Choice.B(_): 1; else 2; end case;", "FP3025")]
    [InlineData("case Choice.B(1) of when Choice.B(const V: int): discard V; when Choice.A: begin end; end case; discard V;", "FP3001")]
    [InlineData("discard case Choice.A of when Choice.A, Choice.B(const V: int): 1; end case;", "FP3024")]
    [InlineData("discard case 1 of when 1: 2; end case;", "FP3025")]
    [InlineData("discard case true of when true: 1; end case;", "FP3025")]
    [InlineData("discard case 1 of when 2..1: 1; else 0; end case;", "FP3024")]
    public void Invalid_variant_operations_and_case_patterns_are_diagnosed(string body, string code)
    {
        var result = TestCompilation.Compile(Header + "begin " + body + " end; end program;");
        Assert.False(result.Success);
        Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
    }

    [Theory]
    [InlineData("type U = variant end variant;")]
    [InlineData("type U = variant A; A; end variant;")]
    [InlineData("type U = variant Tag; end variant;")]
    [InlineData("type U = variant A(Tag: int); end variant;")]
    [InlineData("type U = variant A(X: int; X: int); end variant;")]
    [InlineData("type U = variant A; ACase; end variant;")]
    [InlineData("type int = variant A; end variant;")]
    public void Invalid_variant_declarations_are_diagnosed(string declaration)
        => Assert.Equal("FP3023", Assert.Single(TestCompilation.Compile("program Example; " + declaration + " begin end; end program;").Diagnostics).Code);

    [Fact]
    public void Nested_variant_coverage_is_proved_recursively()
    {
        var result = TestCompilation.Compile("""
            program Nested;
            type Bit = variant Zero; One; end variant;
            type Pair = variant Make(Left: Bit; Right: Bit); end variant;
            function Read(Value: Pair): int;
            begin
              case Value of
                when Pair.Make(Bit.Zero, _): return 0;
                when Pair.Make(Bit.One, Bit.Zero): return 1;
                when Pair.Make(Bit.One, Bit.One): return 2;
              end case;
            end function;
            begin end;
            end program;
            """);
        Assert.True(result.Success, string.Join(";", result.Diagnostics));
    }
}
