using Fpas.Compiler.Tests.Support;
namespace Fpas.Compiler.Tests.Types;

public sealed class UnionTests
{
    private const string Header = "program Example; type Choice = union A; B(Value: int); end union; ";

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
    public void Invalid_union_operations_and_case_patterns_are_diagnosed(string body, string code)
    {
        var result = TestCompilation.Compile(Header + "begin " + body + " end; end program;");
        Assert.False(result.Success);
        Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
    }

    [Theory]
    [InlineData("type U = union end union;")]
    [InlineData("type U = union A; A; end union;")]
    [InlineData("type U = union Tag; end union;")]
    [InlineData("type U = union A(Tag: int); end union;")]
    [InlineData("type U = union A(X: int; X: int); end union;")]
    [InlineData("type U = union A; ACase; end union;")]
    [InlineData("type int = union A; end union;")]
    public void Invalid_union_declarations_are_diagnosed(string declaration)
        => Assert.Equal("FP3023", Assert.Single(TestCompilation.Compile("program Example; " + declaration + " begin end; end program;").Diagnostics).Code);

    [Fact]
    public void Nested_union_coverage_is_proved_recursively()
    {
        var result = TestCompilation.Compile("""
            program Nested;
            type Bit = union Zero; One; end union;
            type Pair = union Make(Left: Bit; Right: Bit); end union;
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
