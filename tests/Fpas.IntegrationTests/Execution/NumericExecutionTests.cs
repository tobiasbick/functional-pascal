using Fpas.IntegrationTests.Support;
namespace Fpas.IntegrationTests.Execution;

public sealed class NumericExecutionTests
{
    [Theory]
    [InlineData("int", "2 + 3 * 4", "14")]
    [InlineData("int", "(2 + 3) * 4 - 1", "19")]
    [InlineData("int", "20 - 5 - 3", "12")]
    [InlineData("int", "1 + (2 + (3 + (4 + (5 + (6 + (7 + (8 + (9 + 10))))))))", "55")]
    [InlineData("int", "-7 div 3", "-2")]
    [InlineData("int", "-7 mod 3", "-1")]
    [InlineData("int", "+4 * -2", "-8")]
    [InlineData("int", "-2_147_483_648", "-2147483648")]
    [InlineData("int", "1 shl 3 + 1", "16")]
    [InlineData("int", "-8 shr 1", "-4")]
    [InlineData("int", "1 shl 33", "2")]
    [InlineData("int", "not 0", "-1")]
    [InlineData("int", "7 and 3 xor 8 or 16", "27")]
    [InlineData("double", "1.5 + 2 * 0.5", "2.5")]
    [InlineData("double", "3 / 2.0", "1.5")]
    [InlineData("double", "1_0.5_0e-1 + 2D", "3.05")]
    [InlineData("double", "1e2 + 1d", "101")]
    [InlineData("double", "2", "2")]
    [InlineData("int", "cast<int>(-3.75)", "-3")]
    [InlineData("double", "cast<double>(3) / 2.0", "1.5")]
    [InlineData("bool", "not false and true xor false or false", "True")]
    [InlineData("bool", "2 < 3 and 3 <= 3 and 4 > 3 and 4 >= 4", "True")]
    [InlineData("bool", "2 = 2.0 and 2 <> 3", "True")]
    [InlineData("bool", "false = (1 > 2)", "True")]
    [InlineData("bool", "(1 < 2) = true", "True")]
    [InlineData("bool", "(1.0 / -0.0) < 0.0", "True")]
    [InlineData("bool", "(0.0 / 0.0) <= 1.0", "False")]
    [InlineData("bool", "(0.0 / 0.0) >= 1.0", "False")]
    [InlineData("bool", "(0.0 / 0.0) = (0.0 / 0.0)", "False")]
    [InlineData("bool", "(0.0 / 0.0) <> 1.0", "True")]
    [InlineData("bool", "'a' + 'b' = 'ab'", "True")]
    [InlineData("bool", "'a' <> 'A'", "True")]
    [InlineData("int", "unchecked(2147483647 + 1)", "-2147483648")]
    [InlineData("int", "unchecked(-(-2147483648))", "-2147483648")]
    [InlineData("double", "if false then 1 elsif true then 2.5 else 3 end if", "2.5")]
    [InlineData("int", "if true then 1 else 1 div 0 end if", "1")]
    public Task Typed_expressions_execute_with_CLR_semantics(string type, string expression, string expected)
        => ProgramExecution.Output($"program Numbers; uses System.Console as Console; begin const Value: {type} := {expression}; Console.WriteLine(Value); end; end program;", expected);

    [Theory]
    [InlineData("2147483647 + 1", "OverflowException")]
    [InlineData("-2147483648 - 1", "OverflowException")]
    [InlineData("100000 * 100000", "OverflowException")]
    [InlineData("-(-2147483648)", "OverflowException")]
    [InlineData("cast<int>(2147483648D)", "OverflowException")]
    [InlineData("1 div 0", "DivideByZeroException")]
    [InlineData("unchecked(-2147483648 div -1)", "OverflowException")]
    [InlineData("unchecked(1 + 2147483647) + -1", "OverflowException")]
    public async Task Numeric_failures_use_ordinary_CLR_exceptions(string expression, string exception)
    {
        var result = await ProgramExecution.Run($"program Failure; begin const Value: int := {expression}; end; end program;");
        Assert.NotEqual(0, result.ExitCode);
        Assert.Contains("System." + exception, result.Error);
    }

    [Fact]
    public Task Metadata_overloads_and_source_calls_apply_numeric_widening()
        => ProgramExecution.Output("""
            program Calls;
            uses System.Console as Console, System.Math as Math;
            function Half(Value: double): double;
            begin
              return Value / 2.0;
            end function;
            begin
              Console.WriteLine(Math.Pow(2, 3));
              Console.WriteLine(Math.Max(2, 3));
              Console.WriteLine(Half(5));
            end;
            end program;
            """, "8", "3", "2.5");
}
