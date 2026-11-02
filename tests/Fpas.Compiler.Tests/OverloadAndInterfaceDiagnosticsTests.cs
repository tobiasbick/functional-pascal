namespace Fpas.Compiler.Tests;

public sealed class OverloadAndInterfaceDiagnosticsTests
{
    [Theory]
    [InlineData("function F(V: int): int; begin return V; end function; function F(Other: int): string; begin return ''; end function;")]
    [InlineData("function F(V: string): int; begin return 1; end function; function F(V: string?): int; begin return 2; end function;")]
    [InlineData("function F(ref V: int): int; begin return V; end function; function F(in V: int): int; begin return V; end function;")]
    [InlineData("function F<T>(V: T): T; begin return V; end function; function F<U>(Other: U): U; begin return Other; end function;")]
    public void Duplicate_CLR_signatures_do_not_create_overloads(string declarations)
    {
        var result = DiagnosticsTests.Compile($"program Duplicate; {declarations} begin end; end program;");
        Assert.False(result.Success);
        Assert.Equal("FP3004", Assert.Single(result.Diagnostics).Code);
    }

    [Theory]
    [InlineData("Sys.IDisposable", "")]
    [InlineData("Sys.IComparable<int>", "function CompareTo(Value: double): int; begin return 0; end function;")]
    [InlineData("Sys.String", "")]
    [InlineData("Sys.IDisposable?", "procedure Dispose(); begin end procedure;")]
    [InlineData("Sys.IDisposable, Sys.IDisposable", "procedure Dispose(); begin end procedure;")]
    public void Invalid_record_interface_implementations_are_rejected(string contract, string members)
    {
        var result = RecordTests.Compile($"program Interfaces; uses System as Sys; type R = record implements {contract} {members} end record; begin end; end program;");
        Assert.False(result.Success);
        Assert.Equal("FP3029", Assert.Single(result.Diagnostics).Code);
    }
}
