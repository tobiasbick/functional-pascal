using Fpas.Compiler.Tests.Support;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Types;

public sealed class RecordMetadataTests
{
    [Theory]
    [InlineData("version")]
    [InlineData("copy")]
    [InlineData("constructor")]
    public void Invalid_imported_record_contracts_are_rejected_before_emission(string corruption)
    {
        var library = TestCompilation.Compile("unit Model; public type Item = record Value: int := 1; end record; end unit;", CompilationTarget.Library);
        Assert.True(library.Success);
        var bytes = library.Program!.AssemblyImage.ToArray();
        using (var pe = new PEReader(new MemoryStream(bytes)))
        {
            var reader = pe.GetMetadataReader();
            var type = reader.TypeDefinitions.Select(reader.GetTypeDefinition).Single(type => reader.GetString(type.Name) == "Item");
            if (corruption == "version")
            {
                var attribute = reader.GetCustomAttribute(type.GetCustomAttributes().Last());
                var offset = pe.PEHeaders.MetadataStartOffset + reader.GetHeapMetadataOffset(HeapIndex.Blob) + MetadataTokens.GetHeapOffset(attribute.Value);
                // The small blob has a one-byte length prefix followed by the two-byte attribute prolog.
                bytes[offset + 3] = 2;
            }
            else
            {
                var method = type.GetMethods().Single(handle => reader.GetString(reader.GetMethodDefinition(handle).Name) == (corruption == "copy" ? "<With:Value>" : ".ctor"));
                var offset = pe.PEHeaders.MetadataStartOffset + reader.GetTableMetadataOffset(TableIndex.MethodDef) +
                    (MetadataTokens.GetRowNumber(method) - 1) * reader.GetTableRowSize(TableIndex.MethodDef) + 6;
                var flags = reader.GetMethodDefinition(method).Attributes;
                var changed = (ushort)((flags & ~MethodAttributes.MemberAccessMask) | MethodAttributes.Private);
                bytes[offset] = (byte)changed; bytes[offset + 1] = (byte)(changed >> 8);
            }
        }
        using var directory = new TemporaryDirectory();
        var reference = directory.File("Records.dll"); File.WriteAllBytes(reference, bytes);
        var result = FpasCompiler.Compile(new([new("consumer.fpas", "program Consumer; uses Model as M; begin const Item: M.Item := new M.Item(); end; end program;")],
            "Consumer", [.. TestEnvironment.References, typeof(Core.RecordAttribute).Assembly.Location, reference]));
        Assert.False(result.Success);
        Assert.Equal("FP4001", Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Forward_recursive_types_and_multi_unit_records_have_deterministic_metadata()
    {
        CompilationSource[] sources =
        [
            new("model.fpas", "unit Demo.Model; public type Link = record Next: Link? := null; Other: Payload? := null; end record; public type Payload = record Value: int; end record; end unit;"),
            new("api.fpas", "unit Demo.Api; uses Demo.Model as M; public function Make(): M.Link; begin return new M.Link(Other := new M.Payload(2)); end function; end unit;")
        ];
        string[] references = [.. TestEnvironment.References, typeof(Core.RecordAttribute).Assembly.Location];
        var first = FpasCompiler.Compile(new(sources, "Forward", references, CompilationTarget.Library));
        var second = FpasCompiler.Compile(new(sources.Reverse().ToArray(), "Forward", references.Reverse().ToArray(), CompilationTarget.Library));
        Assert.True(first.Success, string.Join(";", first.Diagnostics));
        Assert.True(second.Success, string.Join(";", second.Diagnostics));
        Assert.Equal(first.Program!.AssemblyImage, second.Program!.AssemblyImage);
    }
}
