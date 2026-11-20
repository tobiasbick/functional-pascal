// [Documentation](../../../docs/pascal/language/pattern-matching/exhaustiveness.md)
using Fpas.Compiler.Tests.Support;
namespace Fpas.Compiler.Tests.Types;

public sealed class CaseCoverageTests
{
    [Fact]
    public void Boolean_payload_coverage_agrees_with_an_independent_truth_table()
    {
        var patterns = new List<(string Text, int Values)>();
        foreach (var left in new[] { "false", "true", "_" })
            foreach (var right in new[] { "false", "true", "_" })
            {
                var values = 0;
                for (var x = 0; x < 2; x++) for (var y = 0; y < 2; y++)
                    if ((left == "_" || left == (x == 0 ? "false" : "true")) && (right == "_" || right == (y == 0 ? "false" : "true"))) values |= 1 << (2 * x + y);
                patterns.Add(($"Pair.Make({left}, {right})", values));
            }
        for (var selection = 0; selection < 1 << patterns.Count; selection++)
        {
            var covered = 0; var unreachable = false; var arms = new List<string>();
            for (var index = 0; index < patterns.Count; index++)
            {
                if ((selection & 1 << index) == 0) continue;
                var pattern = patterns[index];
                unreachable |= (pattern.Values & ~covered) == 0;
                covered |= pattern.Values;
                arms.Add($"when {pattern.Text}: return {index};");
            }
            var result = TestCompilation.Compile("program TruthTable; type Pair = variant Make(Left: bool; Right: bool); end variant; function Read(P: Pair): int; begin case P of " +
                string.Join(' ', arms) + " end case; end function; begin end; end program;");
            Assert.True(result.Success == (!unreachable && covered == 15), $"Selection {selection}: {string.Join(';', result.Diagnostics)}");
            if (unreachable || covered == 15) continue;
            var expectation = Assert.IsType<DiagnosticExpectation>(Assert.Single(result.Diagnostics).Expectation);
            var witness = Assert.Single(expectation.Expected);
            var matches = patterns.Single(pattern => pattern.Text == witness).Values;
            Assert.NotEqual(0, matches);
            Assert.Equal(0, covered & matches);
        }
    }

    [Theory]
    [InlineData("Choice", "when Choice.A: return 0;", "Choice.B(_, _)")]
    [InlineData("Choice", "when Choice.A: return 0; when Choice.B(_, _) if false: return 1;", "Choice.B(_, _)")]
    [InlineData("Choice", "when Choice.A: return 0; when Choice.B(Bit.Off, _): return 1; when Choice.B(Bit.On, true): return 2;", "Choice.B(Bit.On, false)")]
    [InlineData("Choice?", "when Choice.A: return 0; when Choice.B(_, _): return 1;", "null")]
    [InlineData("Flag", "when 1..2: return 0;", "cast<byte>(3)")]
    [InlineData("bool", "when true: return 0;", "false")]
    public void Missing_patterns_are_disjoint_from_covered_arms(string selector, string arms, string expected)
    {
        var prefix = """
            program Choices;
            type Bit = variant Off; On; end variant;
            type Choice = variant A; B(Value: Bit; Valid: bool); end variant;
            type Flag = byte range 1..3;
            """;
        var body = $" function Read(Value: {selector}): int; begin case Value of {arms}";
        const string suffix = " end case; end function; begin end; end program;";
        var diagnostic = Assert.Single(TestCompilation.Compile(prefix + body + suffix).Diagnostics);
        Assert.Equal("FP3025", diagnostic.Code);
        Assert.Contains($"Missing pattern: {expected}.", diagnostic.Message);
        var expectation = Assert.IsType<DiagnosticExpectation>(diagnostic.Expectation);
        Assert.Equal("pattern", expectation.Kind);
        Assert.Equal(expected, Assert.Single(expectation.Expected));
        var repaired = TestCompilation.Compile(prefix + body + $" when {expected}: return 3;" + suffix);
        Assert.True(repaired.Success, string.Join(";", repaired.Diagnostics));
    }

    [Fact]
    public void A_missing_case_is_found_after_an_open_payload_and_uses_escaped_names()
    {
        var result = TestCompilation.Compile("""
            program Example;
            type @record = variant Text(Value: string); @end; end variant;
            function Read(Value: @record): int;
            begin
              return case Value of when @record.Text('first'): 0; end case;
            end function;
            begin end;
            end program;
            """);
        var expectation = Assert.IsType<DiagnosticExpectation>(Assert.Single(result.Diagnostics).Expectation);
        Assert.Equal("@record.@end", Assert.Single(expectation.Expected));
    }

    [Fact]
    public void An_open_selector_reports_the_need_for_a_fallback_without_a_guessed_pattern()
    {
        var result = TestCompilation.Compile("program Example; begin discard case 'a' of when 'a': 1; end case; end; end program;");
        var diagnostic = Assert.Single(result.Diagnostics);
        Assert.Equal("FP3025", diagnostic.Code);
        Assert.Null(diagnostic.Expectation);
    }

    [Theory]
    [InlineData("discard case 'x' of when is Sys.Exception as E: 1; else 0; end case;", "FP3025")]
    [InlineData("discard case 1.0 of when is int as N: N; else 0; end case;", "FP3025")]
    [InlineData("discard case 1 of when null..2: 0; else 1; end case;", "FP3024")]
    [InlineData("discard case 1 of when -(-2147483648): 0; else 1; end case;", "FP3024")]
    [InlineData("discard case 1 of when (1 div 0): 0; else 1; end case;", "FP3024")]
    [InlineData("discard case 1 of when (1 mod 0)..2: 0; else 1; end case;", "FP3024")]
    [InlineData("discard case 1 of when 0..10: 0; when 2..4: 1; else 2; end case;", "FP3025")]
    [InlineData("discard case 1 of when -2147483648..2147483647: 0; end case;", "FP3025")]
    [InlineData("discard case cast<object>('x') of when is object as O: 0; when is string as S: 1; else 2; end case;", "FP3025")]
    [InlineData("var Text: string? := 'known'; case true of when true: Text := null; when false: begin end; end case; discard Text.Length;", "FP3013")]
    [InlineData("var Text: string? := 'known'; const Clear: function(): bool := function(): bool begin Text := null; return false; end function; Text := 'known'; case 1 of when 1 if Clear(): begin end; else discard Text.Length; end case;", "FP3013")]
    [InlineData("var Text: string? := 'known'; const Clear: function(): bool := function(): bool begin Text := null; return false; end function; Text := 'known'; discard case 1 of when 1 if Clear(): 0; else Text.Length; end case;", "FP3013")]
    [InlineData("var Text: string? := 'known'; for I: int in [1, 2] do discard Text.Length; case I of when 1: const Clear: procedure() := procedure() begin Text := null; end procedure; Clear(); else begin end; end case; end for;", "FP3013")]
    public void Invalid_coverage_and_unsafe_flow_are_diagnosed(string body, string code)
    {
        var result = TestCompilation.Compile("program Cases; uses System as Sys; begin " + body + " end; end program;");
        Assert.False(result.Success); Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
    }
}
