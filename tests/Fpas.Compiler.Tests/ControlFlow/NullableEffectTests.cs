using Fpas.Compiler.Tests.Support;
namespace Fpas.Compiler.Tests.ControlFlow;

public sealed class NullableEffectTests
{
    [Theory]
    [InlineData("while body", "while Clear(out Text, true) do discard Text.Length; break; end while;")]
    [InlineData("while exit", "while Clear(out Text, false) do end while; discard Text.Length;")]
    [InlineData("repeated while condition", "while Text.Length > 0 and Clear(out Text, true) do end while;")]
    [InlineData("repeat body back edge", "repeat discard Text.Length; until Clear(out Text, false);")]
    [InlineData("repeated until condition", "repeat until Text.Length = 0 or Clear(out Text, false);")]
    [InlineData("and true branch", "if Text <> null and Clear(out Text, true) then discard Text.Length; end if;")]
    [InlineData("or false branch", "if Text = null or Clear(out Text, false) then begin end; else discard Text.Length; end if;")]
    [InlineData("conditional expression", "discard if Text <> null and Clear(out Text, true) then Text.Length else 0 end if;")]
    [InlineData("ref argument", "if Text <> null and ClearRef(ref Text) then discard Text.Length; end if;")]
    [InlineData("later catch", "try raise new Sys.Exception(); catch E: Sys.Exception when Clear(out Text, false) do catch E: Sys.Exception do discard Text.Length; end try;")]
    [InlineData("finally after filter", "try raise new Sys.Exception(); catch E: Sys.Exception when Clear(out Text, false) do catch E: Sys.Exception do finally discard Text.Length; end try;")]
    [InlineData("statement case selector", "case Text of when _ if Clear(out Text, false): begin end; when is string as Saved: discard Text.Length; else begin end; end case;")]
    [InlineData("expression case selector", "discard case Text of when _ if Clear(out Text, false): 0; when is string as Saved: Text.Length; else 0; end case;")]
    [InlineData("deferred conditional argument", "Take(if Clear(out Text, true) then [] else [] end if, Text.Length);")]
    [InlineData("block lambda out write", "for Index: int := 1 to 2 do discard Text.Length; const Action: procedure() := procedure() begin discard Clear(out Text, true); end procedure; Action(); end for;")]
    [InlineData("block lambda ref write", "for Index: int := 1 to 2 do discard Text.Length; const Action: procedure() := procedure() begin discard ClearRef(ref Text); end procedure; Action(); end for;")]
    public void Effects_cannot_reuse_a_null_fact(string scenario, string body)
    {
        var result = Compile(body);
        Assert.False(result.Success, scenario);
        Assert.Equal("FP3013", Assert.Single(result.Diagnostics).Code);
    }

    [Theory]
    [InlineData("while condition revalidation", "while Clear(out Text, true) and Text <> null do discard Text.Length; end while;")]
    [InlineData("while back edge revalidation", "while Text <> null do discard Text.Length; discard Clear(out Text, true); end while;")]
    [InlineData("repeat revalidation", "repeat if Text <> null then discard Text.Length; end if; until Clear(out Text, false);")]
    [InlineData("and revalidation", "if Text <> null and Clear(out Text, true) and Text <> null then discard Text.Length; end if;")]
    [InlineData("or revalidation", "if Clear(out Text, false) or Text = null then begin end; else discard Text.Length; end if;")]
    [InlineData("later catch revalidation", "try raise new Sys.Exception(); catch E: Sys.Exception when Clear(out Text, false) do catch E: Sys.Exception do if Text <> null then discard Text.Length; end if; end try;")]
    [InlineData("case saved binding", "case Text of when _ if Clear(out Text, false): begin end; when is string as Saved: discard Saved.Length; else begin end; end case;")]
    [InlineData("case original revalidation", "case Text of when _ if Clear(out Text, false): begin end; when is string as Saved: if Text <> null then discard Text.Length; end if; else begin end; end case;")]
    [InlineData("case expression saved binding", "discard case Text of when _ if Clear(out Text, false): 0; when is string as Saved: Saved.Length; else 0; end case;")]
    [InlineData("deferred argument coalescing", "Take(if Clear(out Text, true) then [] else [] end if, (Text ?? '').Length);")]
    [InlineData("read before deferred argument effect", "TakeSize(Text.Length, if Clear(out Text, true) then [] else [] end if);")]
    [InlineData("deferred read before eager argument effect", "Take(if Text.Length > 0 then [] else [] end if, if Clear(out Text, true) then 1 else 0 end if);")]
    [InlineData("block lambda write with revalidation", "for Index: int := 1 to 2 do if Text <> null then discard Text.Length; end if; const Action: procedure() := procedure() begin discard Clear(out Text, true); end procedure; Action(); end for;")]
    [InlineData("block lambda read without mutation", "for Index: int := 1 to 2 do discard Text.Length; const Action: procedure() := procedure() begin discard Text; end procedure; Action(); end for;")]
    [InlineData("block lambda writes its own local", "for Index: int := 1 to 2 do discard Text.Length; const Action: procedure() := procedure() begin var Text: string? := 'local'; discard Clear(out Text, true); end procedure; Action(); end for;")]
    public void Revalidation_and_saved_values_remain_valid(string scenario, string body)
    {
        var result = Compile(body);
        Assert.True(result.Success, scenario + ": " + string.Join(';', result.Diagnostics));
    }

    private static CompilationResult Compile(string body) => TestCompilation.Compile("""
        program NullableEffects;
        uses System as Sys;
        function Clear(out Text: string?; Result: bool): bool;
        begin Text := null; return Result; end function;
        function ClearRef(ref Text: string?): bool;
        begin Text := null; return true; end function;
        procedure Take(Items: int[]; Size: int); begin end procedure;
        procedure TakeSize(Size: int; Items: int[]); begin end procedure;
        begin
          var Text: string? := 'initial';
        """ + body + " end; end program;");
}
