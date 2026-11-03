using Fpas.Compiler.Tests.Support;

namespace Fpas.Compiler.Tests.ControlFlow;

public sealed class NullableOperatorDiagnosticsTests
{
    [Theory]
    [InlineData("bool?", "A and B", "bool?")]
    [InlineData("bool?", "A or B", "bool?")]
    [InlineData("bool?", "A xor B", "bool?")]
    [InlineData("bool?", "not A", "bool?")]
    [InlineData("int?", "A / B", "int?")]
    [InlineData("double?", "A and B", "double?")]
    [InlineData("decimal?", "A + cast<double?>(B)", "decimal?")]
    [InlineData("int?", "A + 'text'", "int?")]
    [InlineData("int?", "null + null", "int?")]
    public void Nullable_operators_do_not_invent_underlying_operators_or_Boolean_logic(string type, string expression, string result)
    {
        var compiled = TestCompilation.Compile($$"""
            program Invalid;
            function Test(A: {{type}}; B: {{type}}): {{result}};
            begin return {{expression}}; end function;
            begin end; end program;
            """);
        Assert.Equal("FP3002", Assert.Single(compiled.Diagnostics).Code);
        Assert.Null(compiled.Program);
    }

    [Fact]
    public void Lifted_results_still_require_explicit_unwrapping()
    {
        var compiled = TestCompilation.Compile("""
            program Invalid;
            function Test(A: int?; B: int?): int;
            begin return A + B; end function;
            begin end; end program;
            """);
        Assert.Equal("FP3013", Assert.Single(compiled.Diagnostics).Code);
        Assert.Null(compiled.Program);
    }

    [Theory]
    [InlineData("where T: struct;", "T?", "A + B")]
    [InlineData("where T: struct;", "bool", "A = B")]
    [InlineData("where T: struct, N.IAdditionOperators<T, T, string>;", "string", "A + B")]
    public void Generic_lifting_requires_a_supported_value_operator_signature(string constraints, string result, string expression)
    {
        var compiled = TestCompilation.Compile($$"""
            program Invalid;
            uses System.Numerics as N;
            function Test<T>(A: T?; B: T?): {{result}};
            {{constraints}}
            begin return {{expression}}; end function;
            begin end; end program;
            """);
        Assert.Equal("FP3002", Assert.Single(compiled.Diagnostics).Code);
        Assert.Null(compiled.Program);
    }

    [Fact]
    public void Lifting_does_not_erase_distinct_domains_to_make_arithmetic_applicable()
    {
        var compiled = TestCompilation.Compile("""
            program Invalid;
            type Meters = distinct int;
            function Add(A: Meters?; B: Meters?): Meters?; begin return A + B; end function;
            begin end; end program;
            """);
        Assert.Equal("FP3002", Assert.Single(compiled.Diagnostics).Code);
        Assert.Null(compiled.Program);
    }
}
