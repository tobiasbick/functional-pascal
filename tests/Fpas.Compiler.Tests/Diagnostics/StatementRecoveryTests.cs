// [Documentation](../../../docs/pascal/tools/diagnostics.md#recovery-and-limits)
using Fpas.Compiler.Tests.Support;

namespace Fpas.Compiler.Tests.Diagnostics;

public sealed class StatementRecoveryTests
{
    [Fact]
    public void Failed_declarations_hide_outer_names_and_suppress_transitive_errors()
    {
        var result = TestCompilation.Compile("""
            program Example;
            procedure Inspect(Value: int);
            begin
              begin
                const Value: string := 7;
                const Forwarded: int := Value.Length;
                discard Forwarded.ToString();
                discard MissingInside;
              end;
              discard Value.ToString();
              discard MissingOutside;
            end procedure;
            begin end;
            end program;
            """);
        Assert.Equal(new[] { "FP3002", "FP3001", "FP3001" }, result.Diagnostics.Select(item => item.Code));
        Assert.Contains("MissingInside", result.Diagnostics[1].Message);
        Assert.Contains("MissingOutside", result.Diagnostics[2].Message);
        Assert.Null(result.Program);
    }

    [Fact]
    public void Failed_using_and_nested_routines_preserve_independent_errors()
    {
        var result = TestCompilation.Compile("""
            program Example;
            procedure Inspect();
              function Inner(): int;
              begin
                const First: int := MissingInner;
                return First;
              end function;
              function Other(): int;
              begin
                return MissingOther;
              end function;
            begin
              using const Resource: string := 'not disposable';
              discard Resource.Length;
              discard MissingOuter;
            end procedure;
            begin end;
            end program;
            """);
        Assert.Equal(new[] { "FP3016", "FP3001", "FP3001", "FP3001" }, result.Diagnostics.Select(item => item.Code));
        Assert.Contains("MissingOuter", result.Diagnostics[1].Message);
        Assert.Contains("MissingInner", result.Diagnostics[2].Message);
        Assert.Contains("MissingOther", result.Diagnostics[3].Message);
    }

    [Fact]
    public void Failed_lambda_and_return_do_not_create_cascading_flow_errors()
    {
        var result = TestCompilation.Compile("""
            program Example;
            function Read(): int;
            begin
              const ReadValue: function(): int := function(): int => MissingLambda;
              return ReadValue();
              return MissingReturn;
            end function;
            begin end;
            end program;
            """);
        Assert.Equal(new[] { "FP3001", "FP3001" }, result.Diagnostics.Select(item => item.Code));
        Assert.Contains("MissingLambda", result.Diagnostics[0].Message);
        Assert.Contains("MissingReturn", result.Diagnostics[1].Message);
    }

    [Fact]
    public void Failed_nested_signature_does_not_block_other_routines_or_statements()
    {
        var result = TestCompilation.Compile("""
            program Example;
            function Read(): int;
              function Broken of T(Value: MissingType): int;
              begin
                return Value;
              end function;
              function Other(): int;
              begin
                return MissingOther;
              end function;
            begin
              discard Broken of int(1);
              return MissingOuter;
            end function;
            begin end;
            end program;
            """);
        Assert.Equal(3, result.Diagnostics.Count);
        Assert.Contains("MissingType", result.Diagnostics[0].Message);
        Assert.Contains("MissingOuter", result.Diagnostics[1].Message);
        Assert.Contains("MissingOther", result.Diagnostics[2].Message);
    }

    [Fact]
    public void Statement_recovery_respects_the_compilation_diagnostic_limit()
    {
        var statements = string.Join("\n", Enumerable.Range(1, 60).Select(index => $"discard Missing{index};"));
        var result = TestCompilation.Compile($"program Example; begin {statements} end; end program;");
        Assert.Equal(50, result.Diagnostics.Count);
        Assert.All(result.Diagnostics.Take(49), item => Assert.Equal("FP3001", item.Code));
        Assert.Equal("FP0003", result.Diagnostics[^1].Code);
        Assert.Null(result.Program);
    }
}
