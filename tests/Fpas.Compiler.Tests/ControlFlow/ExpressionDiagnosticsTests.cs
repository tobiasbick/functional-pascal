namespace Fpas.Compiler.Tests.ControlFlow;

public sealed class ExpressionDiagnosticsTests
{
    [Theory]
    [InlineData("and", "or")]
    [InlineData("and", "xor")]
    [InlineData("or", "and")]
    [InlineData("or", "xor")]
    [InlineData("xor", "and")]
    [InlineData("xor", "or")]
    public void Mixed_logical_chains_diagnose_the_first_different_operator(string first, string second)
    {
        var statement = $"const Value: bool := true {first} (false) {second} true;";
        var result = DiagnosticsTests.Compile($"program Bad;\nbegin\n{statement}\nend;\nend program;");
        var diagnostic = Assert.Single(result.Diagnostics);
        Assert.Equal("FP2003", diagnostic.Code);
        Assert.Equal(3, diagnostic.Location!.Line);
        Assert.Equal(statement.IndexOf($" {second} ", StringComparison.Ordinal) + 2, diagnostic.Location.Column);
    }

    [Theory]
    [InlineData("const X: int := 1.0;", "FP3002")]
    [InlineData("const X: int := true;", "FP3002")]
    [InlineData("const X: bool := 1;", "FP3002")]
    [InlineData("const X: int := 1 / 2;", "FP3002")]
    [InlineData("const X: double := 1.0 div 2;", "FP3002")]
    [InlineData("const X: bool := 'a' < 'b';", "FP3002")]
    [InlineData("const X: bool := 1 and true;", "FP3002")]
    [InlineData("const X: int := not 1.0;", "FP3002")]
    [InlineData("const X: bool := -true;", "FP3002")]
    [InlineData("const X: int := cast<int>('1');", "FP3002")]
    [InlineData("const X: string := if true then 'x' else 1 end if;", "FP3002")]
    [InlineData("const X: string := if 1 then 'x' else 'y' end if;", "FP3002")]
    [InlineData("const X: bool := 1 < 2 < 3;", "FP2003")]
    [InlineData("const X: bool := true = 1 < 2;", "FP2003")]
    [InlineData("const X: bool := 1 < 2 = true;", "FP2003")]
    [InlineData("const X: int := 1; X := 2;", "FP3009")]
    [InlineData("Missing := 1;", "FP3009")]
    [InlineData("if 1 then end if;", "FP3002")]
    [InlineData("while 'yes' do end while;", "FP3002")]
    [InlineData("for I := 0 to 1 do end for;", "FP2002")]
    [InlineData("for I: double := 0.0 to 1.0 do end for;", "FP3002")]
    [InlineData("for I: int := 0 to 1 do I := 2; end for;", "FP3009")]
    [InlineData("for I: int := 0 to 1 do end for; Console.WriteLine(I);", "FP3001")]
    [InlineData("begin const X: int := 1; end; Console.WriteLine(X);", "FP3001")]
    [InlineData("break;", "FP3010")]
    [InlineData("continue;", "FP3010")]
    [InlineData("while true do break; Console.WriteLine(1); end while;", "FP3008")]
    [InlineData("const X: double := 1e999;", "FP3002")]
    [InlineData("const X: double := 1e+;", "FP1004")]
    [InlineData("const X: double := 1.0__1;", "FP1003")]
    [InlineData("const X: double := 1e_2;", "FP1004")]
    [InlineData("const X: double := 1.0D_1;", "FP1004")]
    public void Invalid_expressions_and_control_flow_are_diagnosed(string body, string code)
    {
        var result = DiagnosticsTests.Compile("program Bad; uses System.Console as Console; begin " + body + " end; end program;");
        Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
        Assert.Null(result.Program);
    }

    [Theory]
    [InlineData("function F(X: bool): int; begin if X then return 1; end if; end function;", "FP3007")]
    [InlineData("function F(X: int): int; begin X := 2; return X; end function;", "FP3009")]
    [InlineData("procedure F(); begin return; end procedure; function G(): int; begin return F(); end function;", "FP3002")]
    [InlineData("function F(X: bool): int; begin if X then return 1; else return 2; end if; return 3; end function;", "FP3008")]
    [InlineData("procedure F(); begin end procedure; function G(): int; begin return if true then F() else F() end if; end function;", "FP3002")]
    public void Routine_flow_and_void_values_are_checked(string declarations, string code)
        => Assert.Equal(code, Assert.Single(DiagnosticsTests.Compile("program Bad; " + declarations + " begin end; end program;").Diagnostics).Code);

    [Fact]
    public void Unreachable_diagnostics_point_to_the_unreachable_statement()
    {
        var result = DiagnosticsTests.Compile("program Bad;\nbegin\nreturn;\nConsole.WriteLine(1);\nend; end program;");
        var diagnostic = Assert.Single(result.Diagnostics);
        Assert.Equal("FP3008", diagnostic.Code);
        Assert.Equal(4, diagnostic.Location!.Line);
        Assert.Equal(1, diagnostic.Location.Column);
    }
}
