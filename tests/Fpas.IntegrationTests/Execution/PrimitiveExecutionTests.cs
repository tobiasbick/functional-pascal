// [Documentation](../../../docs/pascal/language/basics/primitive-types.md)
using Fpas.IntegrationTests.Support;
namespace Fpas.IntegrationTests.Execution;

public sealed class PrimitiveExecutionTests
{
    [Fact]
    public Task Unchecked_widening_retains_source_signedness_and_constant_evaluation_matches_runtime() => ProgramExecution.Output("""
        program Conversion;
        uses System.Console as Console;
        function Folded(Value: int := unchecked(cast 1.0e30 as int)): int; begin return Value; end function;
        function DecimalByte(Value: byte := unchecked(cast 12.75M as byte)): byte; begin return Value; end function;
        begin
          const Negative: int := -1; const Positive: uint := 4294967295U;
          Console.WriteLine(unchecked(cast Negative as ulong) = 18446744073709551615UL);
          Console.WriteLine(unchecked(cast Positive as long) = 4294967295L);
          Console.WriteLine(unchecked(cast Negative as nuint) = unchecked(cast -1L as nuint));
          Console.WriteLine(unchecked(cast Positive as nint) = unchecked(cast 4294967295L as nint));
          const Large: double := 1.0e30; Console.WriteLine(Folded() = unchecked(cast Large as int));
          const U32: uint := 1U; const U64: ulong := 1UL;
          const Next32: uint := U32 + 1; const Next64: ulong := 1 + U64;
          Console.WriteLine(Next32 = Next64);
          Console.WriteLine(DecimalByte());
        end;
        end program;
        """, "True", "True", "True", "True", "True", "True", "12");

    [Theory]
    [InlineData("long", "-9223372036854775808L", "-9223372036854775808")]
    [InlineData("ulong", "$FFFF_FFFF_FFFF_FFFFUL", "18446744073709551615")]
    [InlineData("uint", "4294967295U shr 1", "2147483647")]
    [InlineData("long", "2147483647 + 1L", "2147483648")]
    [InlineData("byte", "255", "255")]
    [InlineData("sbyte", "-128", "-128")]
    [InlineData("short", "-32768", "-32768")]
    [InlineData("ushort", "65535", "65535")]
    [InlineData("float", "1.5F * 2.0F", "3")]
    [InlineData("decimal", "0.1M + 0.2M", "0.3")]
    [InlineData("byte", "unchecked(cast 256 as byte)", "0")]
    [InlineData("long", "cast 4294967295U as long", "4294967295")]
    [InlineData("double", "cast 18446744073709551615UL as double", "1.8446744073709552E+19")]
    [InlineData("nint", "cast 42L as nint", "42")]
    [InlineData("nuint", "cast 42UL as nuint", "42")]
    public Task All_primitives_literals_and_conversions_execute(string type, string expression, string output)
        => ProgramExecution.Output("program Numbers; uses System.Console as Console; begin const X: " + type + " := " + expression + "; Console.WriteLine(X); end; end program;", output);

    [Fact]
    public Task Signed_unsigned_decimal_arrays_nullable_values_and_contracts_execute() => ProgramExecution.Output("""
        program Numbers;
        uses System.Console as Console;
        function Amount(Value: decimal := 0.25M): decimal;
        requires Value >= 0.0M;
        ensures (Returned: decimal): Returned = Value * 2.0M;
        begin return Value * 2.0M; end function;
        begin
          Console.WriteLine(18446744073709551615UL > 1UL);
          Console.WriteLine(4294967295U div 2U);
          Console.WriteLine(Amount());
          const Values: array of byte := [1, 2, 255]; Console.WriteLine(Values[2]);
          const Optional: ulong? := 18446744073709551615UL; Console.WriteLine(Optional ?? 0UL);
          const Small: short := 2; const Sum: int := Small + Small; Console.WriteLine(Sum);
        end;
        end program;
        """, "True", "2147483647", "0.500", "255", "18446744073709551615", "4");

    [Theory]
    [InlineData("discard 18446744073709551615UL + 1UL;")]
    [InlineData("discard 9223372036854775807L + 1L;")]
    [InlineData("const X: int := 256; discard cast X as byte;")]
    [InlineData("discard cast 18446744073709551615UL as long;")]
    public async Task Integral_overflow_remains_checked(string body)
    {
        var result = await ProgramExecution.Run("program Numbers; begin " + body + " end; end program;");
        Assert.NotEqual(0, result.ExitCode); Assert.Contains("OverflowException", result.Error);
    }
}
