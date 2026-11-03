using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests;

public sealed class LoopTests
{
    [Fact]
    public void Repeat_until_has_a_visible_condition_sequence_point()
    {
        var result = FpasCompiler.Compile(new([new("repeat.fpas", "program DebugLoop;\nbegin\nvar Count: int := 0;\nrepeat\nCount := Count + 1;\nuntil Count = 2;\nend;\nend program;")],
            "DebugLoop", TestEnvironment.References, EmitDebugInformation: true));
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
        using var pe = new PEReader(new MemoryStream(result.Program!.AssemblyImage));
        var metadata = pe.GetMetadataReader(); var entry = metadata.MethodDefinitions.Single(handle => metadata.GetString(metadata.GetMethodDefinition(handle).Name) == "<EntryPoint>");
        using var provider = MetadataReaderProvider.FromPortablePdbStream(new MemoryStream(result.Program.PortablePdb!));
        Assert.Contains(provider.GetMetadataReader().GetMethodDebugInformation(entry).GetSequencePoints(), point => !point.IsHidden && point.StartLine == 6);
    }

    [Theory]
    [InlineData("", "for Index: bool := false to true do end for;", "FP3002")]
    [InlineData("", "for Index: double := 0.0 to 1.0 do end for;", "FP3002")]
    [InlineData("", "for Index: int? := 0 to 1 do end for;", "FP3002")]
    [InlineData("type Id = distinct int;", "for Index: Id := new Id(0) to new Id(1) do end for;", "FP3002")]
    [InlineData("type R = int range 1..3;", "for Index: R := 1 to 3 do end for;", "FP3002")]
    [InlineData("type R = int range 1..3;", "for Index: R? := null to null do end for;", "FP3002")]
    [InlineData("type E = enum A; B; end enum;", "for Index: E := E.A to E.B do end for;", "FP3002")]
    [InlineData("", "for Index: byte := 0 to 1 do Index := 2; end for;", "FP3009")]
    [InlineData("", "repeat var Inner: int := 1; until Inner = 1;", "FP3001")]
    [InlineData("", "repeat until 1;", "FP3002")]
    [InlineData("", "repeat until true; discard Inner;", "FP3001")]
    [InlineData("", "repeat continue; discard 1; until true;", "FP3008")]
    [InlineData("", "repeat until false; discard 1;", "FP3008")]
    [InlineData("", "try repeat break; until true; finally break; end try;", "FP3017")]
    public void Invalid_counter_types_scope_conditions_and_control_transfers_are_rejected(string declarations, string body, string code)
    {
        var result = RecordTests.Compile("program Example; " + declarations + " begin " + body + " end; end program;");
        Assert.False(result.Success); Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
    }

    [Theory]
    [InlineData("repeat continue; until Number = 1;")]
    [InlineData("repeat break; until true;")]
    public void Repeat_paths_cannot_read_or_publish_an_unassigned_out_parameter(string body)
    {
        var result = RecordTests.Compile("program Example; procedure Initialize(out Number: int); begin " + body + " end procedure; begin end; end program;");
        Assert.False(result.Success); Assert.Equal("FP3028", Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Repeat_nullable_facts_account_for_ref_out_writes_before_the_test()
    {
        var result = RecordTests.Compile("""
            program Example;
            procedure Clear(out Text: string?); begin Text := null; end procedure;
            begin
              var Text: string? := 'initial';
              repeat Clear(out Text); until Text.Length > 0;
            end;
            end program;
            """);
        Assert.False(result.Success); Assert.Equal("FP3013", Assert.Single(result.Diagnostics).Code);
    }
}
