using Fpas.Compiler.Tests.Support;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Types;

public sealed class ScalarMetadataTests
{
    [Theory]
    [InlineData("domain-version")]
    [InlineData("domain-mutable")]
    [InlineData("domain-constructor")]
    [InlineData("domain-equality")]
    [InlineData("domain-ordering")]
    [InlineData("alias-version")]
    [InlineData("alias-public")]
    public void Invalid_imported_scalar_contracts_are_rejected_before_emission(string corruption)
    {
        var library = TestCompilation.Compile("unit Model; public type Id = distinct long; public type Count = ulong; end unit;", CompilationTarget.Library);
        Assert.True(library.Success, string.Join(";", library.Diagnostics));
        var bytes = library.Program!.AssemblyImage.ToArray();
        using (var pe = new PEReader(new MemoryStream(bytes)))
        {
            var reader = pe.GetMetadataReader();
            var type = reader.TypeDefinitions.Select(reader.GetTypeDefinition).Single(type => reader.GetString(type.Name) == "Id");
            if (corruption.EndsWith("version", StringComparison.Ordinal))
            {
                var attributes = corruption.StartsWith("domain", StringComparison.Ordinal) ? type.GetCustomAttributes()
                    : reader.GetMethodDefinition(reader.MethodDefinitions.Single(handle => reader.GetString(reader.GetMethodDefinition(handle).Name) == "<Alias:Count>")).GetCustomAttributes();
                var attribute = reader.GetCustomAttribute(attributes.Last());
                var offset = pe.PEHeaders.MetadataStartOffset + reader.GetHeapMetadataOffset(HeapIndex.Blob) + MetadataTokens.GetHeapOffset(attribute.Value);
                // A one-byte blob length precedes the two-byte attribute prolog.
                bytes[offset + 3] = 2;
            }
            else if (corruption == "domain-mutable")
            {
                var field = type.GetFields().First();
                var offset = pe.PEHeaders.MetadataStartOffset + reader.GetTableMetadataOffset(TableIndex.Field) +
                    (MetadataTokens.GetRowNumber(field) - 1) * reader.GetTableRowSize(TableIndex.Field);
                var flags = (ushort)(reader.GetFieldDefinition(field).Attributes & ~FieldAttributes.InitOnly);
                bytes[offset] = (byte)flags; bytes[offset + 1] = (byte)(flags >> 8);
            }
            else
            {
                var method = corruption == "alias-public"
                    ? reader.MethodDefinitions.Single(handle => reader.GetString(reader.GetMethodDefinition(handle).Name) == "<Alias:Count>")
                    : type.GetMethods().Single(handle => reader.GetString(reader.GetMethodDefinition(handle).Name) ==
                        (corruption == "domain-equality" ? "op_Equality" : corruption == "domain-ordering" ? "op_LessThan" : ".ctor"));
                var offset = pe.PEHeaders.MetadataStartOffset + reader.GetTableMetadataOffset(TableIndex.MethodDef) +
                    (MetadataTokens.GetRowNumber(method) - 1) * reader.GetTableRowSize(TableIndex.MethodDef) + 6;
                var flags = (ushort)((reader.GetMethodDefinition(method).Attributes & ~MethodAttributes.MemberAccessMask) |
                    (corruption == "alias-public" ? MethodAttributes.Public : MethodAttributes.Private));
                bytes[offset] = (byte)flags; bytes[offset + 1] = (byte)(flags >> 8);
            }
        }
        using var directory = new TemporaryDirectory();
        var reference = directory.File("Records.dll"); File.WriteAllBytes(reference, bytes);
        var body = corruption.StartsWith("alias", StringComparison.Ordinal) ? "const Value: M.Count := 1UL;" : "discard new M.Id(1L);";
        var result = FpasCompiler.Compile(new([new("consumer.fpas", "program Consumer; uses Model as M; begin " + body + " end; end program;")],
            "Consumer", [.. TestEnvironment.References, typeof(Core.DomainAttribute).Assembly.Location, reference]));
        Assert.False(result.Success); Assert.Equal("FP4001", Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Scalar_declarations_and_aliases_emit_deterministically_across_source_order()
    {
        CompilationSource[] sources =
        [
            new("types.fpas", "unit Model; public type Count = ulong; public type Id = distinct long; public type Port = ushort range 1..65535; public type Choice = enum : byte First; Last = 255; end enum; end unit;"),
            new("api.fpas", "unit Api; uses Model as M; public function Create(Value: M.Count): M.Id; begin return new M.Id(cast<long>(Value)); end function; end unit;")
        ];
        string[] references = [.. TestEnvironment.References, typeof(Core.DomainAttribute).Assembly.Location];
        var first = FpasCompiler.Compile(new(sources, "Scalars", references, CompilationTarget.Library));
        var second = FpasCompiler.Compile(new(sources.Reverse().ToArray(), "Scalars", references.Reverse().ToArray(), CompilationTarget.Library));
        Assert.True(first.Success, string.Join(";", first.Diagnostics)); Assert.True(second.Success, string.Join(";", second.Diagnostics));
        Assert.Equal(first.Program!.AssemblyImage, second.Program!.AssemblyImage);
    }
}
