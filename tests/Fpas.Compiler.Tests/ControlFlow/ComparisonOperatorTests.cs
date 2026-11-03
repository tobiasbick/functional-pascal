using Fpas.Compiler.Tests.Support;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;

namespace Fpas.Compiler.Tests.ControlFlow;

public sealed class ComparisonOperatorTests
{
    [Fact]
    public void Generated_record_equality_retains_its_known_null_guard_semantics()
    {
        var result = TestCompilation.Compile("""
            program Guard;
            type Item = record Value: int; end record;
            function Read(Value: Item?): int;
            begin if Value <> null then return Value.Value; else return 0; end if; end function;
            begin end;
            end program;
            """);
        Assert.True(result.Success, string.Join("; ", result.Diagnostics));
    }

    [Theory]
    [InlineData("=", "op_Equality")]
    [InlineData("<>", "op_Inequality")]
    [InlineData("<", "op_LessThan")]
    [InlineData(">", "op_GreaterThan")]
    [InlineData("<=", "op_LessThanOrEqual")]
    [InlineData(">=", "op_GreaterThanOrEqual")]
    public void Imported_value_comparisons_emit_actual_CLR_operator_calls(string op, string method)
    {
        var result = DiagnosticsTests.Compile($$"""
            program Compare;
            uses System as Sys;
            function Test(A: Sys.DateTime; B: Sys.DateTime): bool;
            begin return A {{op}} B; end function;
            begin end;
            end program;
            """);
        Assert.True(result.Success, string.Join("; ", result.Diagnostics));
        using var pe = new PEReader(new MemoryStream(result.Program!.AssemblyImage));
        var reader = pe.GetMetadataReader();
        Assert.Contains(reader.MemberReferences, handle => reader.GetString(reader.GetMemberReference(handle).Name) == method);
    }

    [Theory]
    [InlineData("Sys.DateTime", "=", "FP3027")]
    [InlineData("Sys.TimeSpan", "<", "FP3027")]
    [InlineData("Sys.Random", "<", "FP3002")]
    [InlineData("Sys.DateTime?", "=", "FP3027")]
    public void Comparisons_preserve_contract_restrictions_and_reject_unsupported_operands(string type, string op, string code)
    {
        var contract = code == "FP3027";
        var result = DiagnosticsTests.Compile($$"""
            program Compare;
            uses System as Sys;
            function Test(A: {{type}}; B: {{type}}): bool;
            {{(contract ? $"requires A {op} B;" : "")}}
            begin return {{(contract ? "true" : $"A {op} B")}}; end function;
            begin end;
            end program;
            """);
        Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
        Assert.Null(result.Program);
    }
}
