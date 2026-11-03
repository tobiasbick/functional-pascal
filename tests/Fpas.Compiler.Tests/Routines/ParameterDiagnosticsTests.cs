using Fpas.Compiler.Tests.ControlFlow;
namespace Fpas.Compiler.Tests.Routines;

public sealed class ParameterDiagnosticsTests
{
    [Theory]
    [InlineData("procedure Check(out Value: int); begin end procedure;", "", "FP3028")]
    [InlineData("procedure Check(out Value: int); begin Value := Value + 1; end procedure;", "", "FP3028")]
    [InlineData("procedure Check(out Value: int); requires Value > 0; begin Value := 1; end procedure;", "", "FP3028")]
    [InlineData("procedure Check(out Value: int; Choose: bool); begin if Choose then Value := 1; end if; end procedure;", "", "FP3028")]
    [InlineData("procedure Check(out Value: int); begin while false do Value := 1; end while; end procedure;", "", "FP3028")]
    [InlineData("procedure Check(in Value: int); begin Value := 1; end procedure;", "", "FP3009")]
    [InlineData("procedure Check(Value: int); begin Value := 1; end procedure;", "", "FP3009")]
    [InlineData("procedure Check(ref Value: int := 0); begin end procedure;", "", "FP3028")]
    [InlineData("procedure Check(params Value: int); begin end procedure;", "", "FP3028")]
    [InlineData("procedure Check(params Values: int[]; Last: int); begin end procedure;", "", "FP3028")]
    [InlineData("procedure Check(Value: int := 1; Required: int); begin end procedure;", "", "FP3028")]
    [InlineData("procedure Check(Value: int := Sys.Math.Abs(-1)); begin end procedure;", "", "FP3028")]
    [InlineData("procedure Check(Value: int := if true then 1 else Sys.Environment.TickCount end if); begin end procedure;", "", "FP3028")]
    [InlineData("procedure Check(Value: bool := false and Sys.Environment.TickCount > 0); begin end procedure;", "", "FP3028")]
    [InlineData("procedure Check(ref Value: int); begin end procedure;", "const Value: int := 1; Check(ref Value);", "FP3028")]
    [InlineData("procedure Check(ref Value: double); begin end procedure;", "var Value: int := 1; Check(ref Value);", "FP3003")]
    [InlineData("procedure Check(ref Value: int); begin end procedure;", "var Value: int := 1; Check(Value);", "FP3003")]
    [InlineData("procedure Check(in Value: int); begin end procedure;", "Check(in 1);", "FP3028")]
    [InlineData("procedure Check(Value: int := 1); begin end procedure;", "Check(Missing := 1);", "FP3003")]
    [InlineData("procedure Check(First: int; Second: int); begin end procedure;", "Check(First := 1, 2);", "FP3003")]
    [InlineData("procedure Check(First: int; Second: int); begin end procedure;", "Check(First := 1, First := 2);", "FP3003")]
    [InlineData("procedure Check(ref Value: int); begin const F: procedure() := procedure() begin Value := 2; end procedure; end procedure;", "", "FP3028")]
    public void Invalid_parameter_declarations_and_calls_are_rejected(string declaration, string body, string code)
    {
        var result = DiagnosticsTests.Compile($"program Parameters; uses System as Sys; {declaration} begin {body} end; end program;");
        Assert.False(result.Success);
        Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
    }
}
