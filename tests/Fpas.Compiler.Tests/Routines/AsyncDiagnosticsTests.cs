using Fpas.Compiler.Tests.ControlFlow;
namespace Fpas.Compiler.Tests.Routines;

public sealed class AsyncDiagnosticsTests
{
    [Theory]
    [InlineData("function F(): Tasks.Task; begin await Tasks.Task.Delay(1); return Tasks.Task.CompletedTask; end function;")]
    [InlineData("async function F(): int; begin return 1; end function;")]
    [InlineData("async function F(): Tasks.ValueTask<int>; begin return 1; end function;")]
    [InlineData("async function F(): Tasks.Task?; begin return; end function;")]
    [InlineData("async function F(ref V: int): Tasks.Task; begin return; end function;")]
    [InlineData("async function F(out V: int): Tasks.Task; begin V := 1; return; end function;")]
    [InlineData("async function F(in V: int): Tasks.Task; begin return; end function;")]
    [InlineData("async function F(): Tasks.Task; begin await 1; end function;")]
    [InlineData("async function F(): Tasks.Task; begin try await Tasks.Task.Delay(1); catch E: Sys.Exception when await Tasks.Task.FromResult<bool>(true) do end try; end function;")]
    [InlineData("procedure Touch(ref V: int; Value: int); begin V := Value; end procedure; async function F(): Tasks.Task; begin var V: int := 0; Touch(ref V, await Tasks.Task.FromResult<int>(1)); end function;")]
    [InlineData("async function F(): Tasks.Task; begin const Points: array of Drawing.Point := [new Drawing.Point(0, 0)]; Points[0].X := await Tasks.Task.FromResult<int>(1); end function;")]
    public void Unsupported_async_forms_report_diagnostics(string declarations)
    {
        var result = DiagnosticsTests.Compile($"program AsyncErrors; uses System as Sys, System.Drawing as Drawing, System.Threading.Tasks as Tasks; {declarations} begin end; end program;");
        Assert.False(result.Success);
        Assert.Equal("FP3030", Assert.Single(result.Diagnostics).Code);
    }
}
