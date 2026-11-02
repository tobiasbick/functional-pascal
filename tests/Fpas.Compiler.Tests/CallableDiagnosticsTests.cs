namespace Fpas.Compiler.Tests;

public sealed class CallableDiagnosticsTests
{
    [Theory]
    [InlineData("const F: function(int): int := function(Value): int => Value;", "FP2002")]
    [InlineData("const F: function(int): int := function(Value: int) => Value;", "FP2001")]
    [InlineData("const F: function(int): int := function(Value: int; Value: int): int => Value;", "FP3004")]
    [InlineData("const F: function(): int := function(): int begin end function;", "FP3007")]
    [InlineData("const F: function(): int := function(): int => 'wrong';", "FP3002")]
    [InlineData("const F: procedure() := procedure() begin return 1; end procedure;", "FP3002")]
    [InlineData("const F: function(): int := function(): int => Missing;", "FP3001")]
    [InlineData("const Value: int := 1; const F: procedure() := procedure() begin Value := 2; end procedure;", "FP3009")]
    [InlineData("for Value: int in [1] do const F: procedure() := procedure() begin break; end procedure; end for;", "FP3010")]
    [InlineData("try raise new Sys.Exception(); catch do const F: procedure() := procedure() begin raise; end procedure; end try;", "FP3017")]
    [InlineData("const F: function(int): int := function(Value: int): int => Value; discard F();", "FP3003")]
    [InlineData("const F: Sys.Func<int>? := null; discard F();", "FP3013")]
    [InlineData("const F: function(int): int := function(Value: int): int => Value; discard F('text');", "FP3002")]
    [InlineData("const F: Sys.Func<string?, int> := function(Value: string): int => Value.Length;", "FP3002")]
    [InlineData("const F: Sys.Func<string> := function(): string? => null;", "FP3002")]
    [InlineData("const F: function(int): bool := function(Value: int): bool => true; const G: Sys.Predicate<int> := F;", "FP3002")]
    [InlineData("discard Sys.Math.Abs;", "FP3019")]
    [InlineData("discard Seq.ToArray(null);", "FP3026")]
    [InlineData("discard Seq.ToArray<int, int>([1]);", "FP3003")]
    [InlineData("var Text: string? := 'known'; const Clear: procedure() := procedure() begin Text := null; end procedure; Text := 'known'; if Text <> null then Clear(); discard Text.Length; end if;", "FP3013")]
    [InlineData("var Text: string? := 'known'; const Read: function(): int := function(): int => Text.Length;", "FP3013")]
    [InlineData("var Text: string? := 'known'; for I: int in [1, 2] do discard Text.Length; const Clear: procedure() := procedure() begin Text := null; end procedure; Clear(); end for;", "FP3013")]
    [InlineData("var Text: string? := 'known'; const Clear: function(): int := function(): int begin Text := null; return 0; end function; Text := 'known'; discard 1 + Clear(); discard Text.Length;", "FP3013")]
    [InlineData("var Text: string? := 'known'; const Clear: function(): int := function(): int begin Text := null; return 0; end function; Text := 'known'; discard if true then Clear() else 0 end if; discard Text.Length;", "FP3013")]
    [InlineData("var Text: string? := 'known'; const Clear: function(): bool := function(): bool begin Text := null; return true; end function; Text := 'known'; if Clear() then discard Text.Length; end if;", "FP3013")]
    [InlineData("var Text: string? := 'known'; const Clear: function(): bool := function(): bool begin Text := null; return true; end function; Text := 'known'; if Text <> null and Clear() then discard Text.Length; end if;", "FP3013")]
    [InlineData("var Text: string? := 'known'; const Clear: function(): bool := function(): bool begin Text := null; return false; end function; Text := 'known'; discard if Text <> null and Clear() then Text.Length else 0 end if;", "FP3013")]
    public void Unsafe_or_invalid_callables_are_diagnosed(string body, string code)
    {
        var result = DiagnosticsTests.Compile("program Bad; uses System as Sys, System.Linq.Enumerable as Seq; begin " + body + " end; end program;");
        Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
        Assert.Null(result.Program);
    }
}
