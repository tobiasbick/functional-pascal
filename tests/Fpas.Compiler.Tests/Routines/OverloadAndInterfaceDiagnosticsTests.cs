// [Documentation](../../../docs/pascal/interop/interfaces.md)
using Fpas.Compiler.Tests.Support;
namespace Fpas.Compiler.Tests.Routines;

public sealed class OverloadAndInterfaceDiagnosticsTests
{
    /// <summary>Distinguishes scalar identities in direct and nested overload signatures.</summary>
    /// <param name="declaration">The common declaration form for two distinct scalar types.</param>
    /// <param name="first">An expression constructing the first scalar.</param>
    /// <param name="second">An expression constructing the second scalar.</param>
    [Theory]
    [InlineData("distinct int", "new A(1)", "new B(1)")]
    [InlineData("int range 0..9", "new A(1)", "new B(1)")]
    [InlineData("enum One; end enum", "A.One", "B.One")]
    public void Different_source_scalar_types_define_distinct_overloads(string declaration, string first, string second)
    {
        var result = TestCompilation.Compile($$"""
            program ScalarOverloads;
            type A = {{declaration}};
            type B = {{declaration}};
            function F(Value: A): int; begin return 1; end function;
            function F(Value: B): int; begin return 2; end function;
            function G(Value: array of A?): int; begin return 1; end function;
            function G(Value: array of B?): int; begin return 2; end function;
            begin
              discard F({{first}});
              discard F({{second}});
              discard G([cast<A?>({{first}})]);
              discard G([cast<B?>({{second}})]);
            end;
            end program;
            """);
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
    }

    [Theory]
    [InlineData("function F(V: int): int; begin return V; end function; function F(Other: int): string; begin return ''; end function;")]
    [InlineData("function F(V: string): int; begin return 1; end function; function F(V: string?): int; begin return 2; end function;")]
    [InlineData("function F(ref V: int): int; begin return V; end function; function F(in V: int): int; begin return V; end function;")]
    [InlineData("function F<T>(V: T): T; begin return V; end function; function F<U>(Other: U): U; begin return Other; end function;")]
    public void Duplicate_CLR_signatures_do_not_create_overloads(string declarations)
    {
        var result = TestCompilation.CompileAgainstFramework($"program Duplicate; {declarations} begin end; end program;");
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
        var result = TestCompilation.Compile($"program Interfaces; uses System as Sys; type R = record implements {contract} {members} end record; begin end; end program;");
        Assert.False(result.Success);
        Assert.Equal("FP3029", Assert.Single(result.Diagnostics).Code);
    }
}
