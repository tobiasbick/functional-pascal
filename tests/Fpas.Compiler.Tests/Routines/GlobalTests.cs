using Fpas.Compiler.Tests.Support;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Routines;

public sealed class GlobalTests
{
    [Theory]
    [InlineData("const X := 1;", "", "FP2002")]
    [InlineData("var X: int;", "", "FP2001")]
    [InlineData("const X: int := 1;", "X := 2;", "FP3009")]
    [InlineData("const X: int := 1; var X: int := 2;", "", "FP3032")]
    [InlineData("type X = int; var X: int := 1;", "", "FP3032")]
    [InlineData("var args: int := 1;", "", "FP3032")]
    [InlineData("var X: int := 1; procedure X(); begin end procedure;", "", "FP3004")]
    [InlineData("const X: int := 1; procedure P(ref Value: int); begin end procedure;", "P(ref X);", "FP3028")]
    [InlineData("var X: int := 1; function Read(): int; requires X > 0; begin return X; end function;", "", "FP3027")]
    [InlineData("const X: int := 1; function Read(): int; requires X > 0; begin return X; end function;", "", "FP3027")]
    [InlineData("const X: int := 1; procedure P(Value: int := X); begin end procedure;", "", "FP3028")]
    [InlineData("var X: string? := null;", "if X <> null then discard X.Length; end if;", "FP3013")]
    [InlineData("const X: int := args.Length;", "", "FP3001")]
    [InlineData("var X: int := 'bad';", "", "FP3002")]
    [InlineData("public var X: int := 1;", "", "FP2003")]
    [InlineData("const X: int := await Tasks.Task.FromResult<int>(1);", "", "FP3030")]
    public void Invalid_globals_storage_and_contract_reads_are_diagnosed(string declaration, string body, string code)
    {
        var result = TestCompilation.Compile("program Example; uses System.Threading.Tasks as Tasks; " + declaration + " begin " + body + " end; end program;");
        Assert.False(result.Success); Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
    }

    [Theory]
    [InlineData("public const Value: string := 'text';", "const X: string := M.Value;")]
    [InlineData("public var Value: string? := null;", "M.Value := 'next'; const X: string := M.Value;")]
    public void Source_and_imported_fields_preserve_visibility_mutability_and_nullability(string declaration, string body)
    {
        var library = TestCompilation.Compile("unit Model; " + declaration + " var Hidden: int := 1; end unit;", CompilationTarget.Library);
        Assert.True(library.Success, string.Join(';', library.Diagnostics));
        using var pe = new PEReader(new MemoryStream(library.Program!.AssemblyImage.ToArray()));
        var reader = pe.GetMetadataReader(); var type = reader.GetTypeDefinition(reader.TypeDefinitions.Single(handle => reader.GetString(reader.GetTypeDefinition(handle).Name) == "Model"));
        Assert.False(type.Attributes.HasFlag(TypeAttributes.BeforeFieldInit));
        var fields = type.GetFields().Select(reader.GetFieldDefinition).ToArray(); Assert.Equal(2, fields.Length);
        var field = fields.Single(field => reader.GetString(field.Name) == "Value");
        Assert.True(field.Attributes.HasFlag(FieldAttributes.Static)); Assert.False(field.Attributes.HasFlag(FieldAttributes.Literal));
        Assert.Equal(declaration.Contains("const", StringComparison.Ordinal), field.Attributes.HasFlag(FieldAttributes.InitOnly));
        var initializer = type.GetMethods().Select(reader.GetMethodDefinition).Single(method => reader.GetString(method.Name) == ".cctor");
        Assert.True(initializer.Attributes.HasFlag(MethodAttributes.Static | MethodAttributes.Private | MethodAttributes.SpecialName | MethodAttributes.RTSpecialName));
        using var directory = new TemporaryDirectory(); var reference = directory.File("Records.dll"); File.WriteAllBytes(reference, library.Program.AssemblyImage.ToArray());
        var result = FpasCompiler.Compile(new([new("consumer.fpas", "program Consumer; uses Model as M; begin " + body + " end; end program;")], "Consumer", [.. TestEnvironment.References, reference]));
        if (declaration.Contains("string?", StringComparison.Ordinal)) { Assert.False(result.Success); Assert.Equal("FP3013", Assert.Single(result.Diagnostics).Code); }
        else Assert.True(result.Success, string.Join(';', result.Diagnostics));
    }

    [Theory]
    [InlineData("M.Hidden := 2;", "FP3011")]
    [InlineData("discard M.Hidden;", "FP3011")]
    [InlineData("M.Value := 2;", "FP3009")]
    public void Other_source_units_cannot_bypass_global_visibility_or_readonly_storage(string body, string code)
    {
        var result = FpasCompiler.Compile(new([new("model.fpas", "unit Model; public const Value: int := 1; var Hidden: int := 0; end unit;"),
            new("consumer.fpas", "program Consumer; uses Model as M; begin " + body + " end; end program;")], "Globals", TestEnvironment.References));
        Assert.False(result.Success); Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Public_fields_cannot_expose_private_source_types()
    {
        var result = TestCompilation.Compile("unit Model; type Id = distinct int; public var Value: Id := new Id(); end unit;", CompilationTarget.Library);
        Assert.False(result.Success); Assert.Equal("FP3032", Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void A_global_named_self_does_not_supply_an_implicit_record_receiver()
    {
        var result = TestCompilation.Compile("""
            program Example;
            var self: int := 1;
            type R = record
              public function Read(): int; begin return 1; end function;
              public static function Invalid(): int; begin return Read(); end function;
            end record;
            begin end;
            end program;
            """);
        Assert.False(result.Success); Assert.Equal("FP3003", Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Global_initializer_debug_points_follow_declarations()
    {
        var result = FpasCompiler.Compile(new([new("state.fpas", "unit Model;\nconst First: int := 1;\nvar Second: int := First + 1;\nend unit;")],
            "Globals", TestEnvironment.References, CompilationTarget.Library, EmitDebugInformation: true));
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
        using var pe = new PEReader(new MemoryStream(result.Program!.AssemblyImage));
        var reader = pe.GetMetadataReader(); var method = reader.MethodDefinitions.Single(handle => reader.GetString(reader.GetMethodDefinition(handle).Name) == ".cctor");
        using var symbols = MetadataReaderProvider.FromPortablePdbStream(new MemoryStream(result.Program.PortablePdb!));
        var points = symbols.GetMetadataReader().GetMethodDebugInformation(method).GetSequencePoints().Where(point => !point.IsHidden).ToArray();
        Assert.Equal(new[] { 2, 3 }, points.Select(point => point.StartLine));
    }

    [Fact]
    public void Global_field_and_initializer_layout_is_deterministic_across_source_and_reference_order()
    {
        CompilationSource[] sources =
        [
            new("state.fpas", "unit State; public var Value: int := 1; public const Read: function(): int := function(): int => Value; end unit;"),
            new("api.fpas", "unit Api; uses State as S; public const Initial: int := S.Read(); public procedure Write(); begin S.Value := 2; end procedure; end unit;")
        ];
        var first = FpasCompiler.Compile(new(sources, "Globals", TestEnvironment.References, CompilationTarget.Library, EmitDebugInformation: true));
        var second = FpasCompiler.Compile(new(sources.Reverse().ToArray(), "Globals", TestEnvironment.References.Reverse().ToArray(), CompilationTarget.Library, EmitDebugInformation: true));
        Assert.True(first.Success, string.Join(';', first.Diagnostics)); Assert.True(second.Success, string.Join(';', second.Diagnostics));
        Assert.Equal(first.Program!.AssemblyImage, second.Program!.AssemblyImage); Assert.Equal(first.Program.PortablePdb, second.Program.PortablePdb);
    }
}
