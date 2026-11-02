namespace Fpas.Compiler.Tests;

public sealed class ContractDiagnosticsTests
{
    [Theory]
    [InlineData("requires 1;", "FP3002")]
    [InlineData("ensures (Returned: double): true;", "FP3027")]
    [InlineData("ensures (Returned: int?): true;", "FP3027")]
    [InlineData("ensures (Value: int): true;", "FP3004")]
    [InlineData("ensures (self: int): true;", "FP3004")]
    [InlineData("ensures true; requires true;", "FP3027")]
    [InlineData("requires Sys.Math.Abs(Value) > 0;", "FP3027")]
    [InlineData("requires new Sys.Random().Next() > 0;", "FP3027")]
    [InlineData("requires Sys.Environment.TickCount > 0;", "FP3027")]
    [InlineData("requires new int[1].Length > 0;", "FP3027")]
    [InlineData("ensures BodyLocal > 0;", "FP3001")]
    [InlineData("ensures (First: int): First > 0; ensures First > 0;", "FP3001")]
    public void Invalid_contracts_report_semantic_diagnostics(string contract, string code)
    {
        var result = DiagnosticsTests.Compile($"""
            program Contracts;
            uses System as Sys;
            function Positive(Value: int): int;
            {contract}
            begin const BodyLocal: int := 1; return Value; end function;
            begin end;
            end program;
            """);
        Assert.False(result.Success);
        Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
    }

    [Theory]
    [InlineData("procedure Check(); ensures (Returned: int): true; begin end procedure;", "")]
    [InlineData("function Check(Value: int): int; requires Returned > 0; ensures (Returned: int): true; begin return Value; end function;", "")]
    [InlineData("function Check(Value: int): int; ensures (Returned: int): true; begin return Returned; end function;", "")]
    [InlineData("", "const Callback: function(int): int := function(Value: int): int requires true; => Value;")]
    public void Contract_result_bindings_are_limited_to_their_own_postcondition(string declaration, string body)
    {
        var result = DiagnosticsTests.Compile($"program Scope; {declaration} begin {body} end; end program;");
        Assert.False(result.Success);
        Assert.Null(result.Program);
    }
}
