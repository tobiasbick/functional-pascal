using Fpas.Compiler.Tests.Support;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Text;
using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Routines;

public sealed class NestedRoutineTests
{
    [Fact]
    public void Nested_method_metadata_symbols_and_debug_output_are_deterministic()
    {
        const string text = """
            unit Factory;
            public function Build<T>(Value: T): function(): T;
              // Read the captured value.
              function Read<U>(Other: U): T; where U: struct;
              begin discard Other; return Value; end function;
              function Get(): T; begin return Read<int>(1); end function;
            begin
              return Get;
            end function;
            end unit;
            """;
        var request = new CompilationRequest([new("factory.fpas", text)], "Nested", TestEnvironment.References,
            CompilationTarget.Library, EmitDebugInformation: true, EmitDocumentation: true);
        var first = FpasCompiler.Compile(request);
        var second = FpasCompiler.Compile(request with { References = request.References.Reverse().ToArray() });
        Assert.True(first.Success, string.Join(';', first.Diagnostics)); Assert.True(second.Success, string.Join(';', second.Diagnostics));
        Assert.Equal(first.Program!.AssemblyImage, second.Program!.AssemblyImage); Assert.Equal(first.Program.PortablePdb, second.Program.PortablePdb);
        using var pe = new PEReader(new MemoryStream(first.Program.AssemblyImage));
        var metadata = pe.GetMetadataReader();
        var local = metadata.MethodDefinitions.Single(handle => metadata.GetString(metadata.GetMethodDefinition(handle).Name) == "Read");
        Assert.Single(metadata.GetMethodDefinition(local).GetGenericParameters());
        var owner = metadata.GetTypeDefinition(metadata.GetMethodDefinition(local).GetDeclaringType());
        Assert.Single(owner.GetGenericParameters());
        Assert.Equal(TypeAttributes.NestedPrivate, owner.Attributes & TypeAttributes.VisibilityMask);
        using var symbols = MetadataReaderProvider.FromPortablePdbStream(new MemoryStream(first.Program.PortablePdb!));
        var helperBodyLine = Array.FindIndex(text.Split('\n'), line => line.Contains("begin discard Other", StringComparison.Ordinal)) + 1;
        Assert.Contains(symbols.GetMetadataReader().GetMethodDebugInformation(local).GetSequencePoints(),
            point => !point.IsHidden && point.StartLine == helperBodyLine);
        var enclosing = metadata.MethodDefinitions.Single(handle => metadata.GetString(metadata.GetMethodDefinition(handle).Name) == "Build");
        Assert.DoesNotContain(symbols.GetMetadataReader().GetMethodDebugInformation(enclosing).GetSequencePoints(),
            point => !point.IsHidden && point.StartLine == helperBodyLine);
        var xml = XDocument.Parse(Encoding.UTF8.GetString(first.Program.XmlDocumentation!));
        Assert.DoesNotContain(xml.Descendants("member"), member => ((string)member.Attribute("name")!).Contains("Read", StringComparison.Ordinal));
    }

    [Theory]
    [InlineData("function F(): int; begin return 1; end function; var F: int := 2;", "FP3004")]
    [InlineData("function F(X: int): int; begin return X; end function; function F(X: int): string; begin return ''; end function;", "FP3004")]
    [InlineData("function F(): int; begin end function;", "FP3007")]
    [InlineData("discard F(); const Value: int := 1; function F(): int; begin return Value; end function;", "FP3033")]
    [InlineData("var Text: string? := 'ready'; function Clear(): int; begin Text := null; return 0; end function; if Text <> null then discard Clear(); discard Text.Length; end if;", "FP3013")]
    [InlineData("function F(out Value: int): int; begin return Value; end function;", "FP3028")]
    [InlineData("function F(): int; requires G() > 0; begin return 1; end function; function G(): int; begin return 1; end function;", "FP3027")]
    [InlineData("begin function F(): int; begin return 1; end function; end; discard F();", "FP3001")]
    [InlineData("const Read: function(): int := F; const Value: int := 1; function F(): int; begin return Value; end function;", "FP3033")]
    [InlineData("function F<T>(Value: T): T; where T: struct; begin return Value; end function; discard F<string>('bad');", "FP3018")]
    public void Invalid_local_declarations_captures_and_flow_are_diagnosed(string body, string code)
    {
        var result = TestCompilation.Compile("program Example; begin " + body + " end; end program;");
        Assert.False(result.Success); Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void A_direct_helper_in_the_outer_statement_block_has_a_placement_diagnostic()
    {
        var result = TestCompilation.Compile("""
            program Example;
            function Outer(): int;
            begin
              function Helper(): int; begin return 42; end function;
              return Helper();
            end function;
            begin end;
            end program;
            """);
        var diagnostic = Assert.Single(result.Diagnostics);
        Assert.Equal("FP2003", diagnostic.Code);
        Assert.Equal(4, diagnostic.Location!.Line);
        Assert.Equal(3, diagnostic.Location.Column);
    }

    [Fact]
    public void Every_capture_in_the_outer_helper_group_must_be_initialized()
    {
        var result = TestCompilation.Compile("""
            program Example;
            function Outer(): int;
              function First(): int; begin return 1; end function;
              function Second(): int; begin return Captured; end function;
            begin
              const Early: int := First();
              const Captured: int := 41;
              return Early + Second();
            end function;
            begin end;
            end program;
            """);
        Assert.Equal("FP3033", Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Nested_routines_cannot_capture_managed_reference_parameters()
    {
        var result = TestCompilation.Compile("program Example; procedure Outer(ref Value: int); function Read(): int; begin return Value; end function; begin  discard Read(); end procedure; begin end; end program;");
        Assert.False(result.Success); Assert.Equal("FP3028", Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Nested_parameters_cannot_replace_the_record_receiver()
    {
        var result = TestCompilation.Compile("""
            program Example;
            type R = record
              public function Read(): int; begin return 1; end function;
              public function Invalid(): int;
                function Inner(self: int): int; begin return Read(); end function;
              begin
                return Inner(1);
              end function;
            end record;
            begin end;
            end program;
            """);
        Assert.False(result.Success); Assert.Equal("FP3004", Assert.Single(result.Diagnostics).Code);
    }
}
