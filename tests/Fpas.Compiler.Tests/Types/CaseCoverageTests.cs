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
            var result = TestCompilation.Compile("program TruthTable; type Pair = union Make(Left: bool; Right: bool); end union; function Read(P: Pair): int; begin case P of " +
                string.Join(' ', arms) + " end case; end function; begin end; end program;");
            Assert.True(result.Success == (!unreachable && covered == 15), $"Selection {selection}: {string.Join(';', result.Diagnostics)}");
        }
    }

    [Theory]
    [InlineData("discard case 'x' of when is Sys.Exception as E: 1; else 0; end case;", "FP3025")]
    [InlineData("discard case 1.0 of when is int as N: N; else 0; end case;", "FP3025")]
    [InlineData("discard case 1 of when null..2: 0; else 1; end case;", "FP3024")]
    [InlineData("discard case 1 of when -(-2147483648): 0; else 1; end case;", "FP3024")]
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
