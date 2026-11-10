using Fpas.Compiler.Tests.Support;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Types;

public sealed class VariantMetadataTests
{
    [Theory]
    [InlineData("version")]
    [InlineData("constructor")]
    [InlineData("case-tag")]
    public void Imported_variant_contracts_are_checked_before_coverage_is_trusted(string corruption)
    {
        var library = TestCompilation.Compile("unit Model; public type Choice = variant A; B(Value: int); end variant; end unit;", CompilationTarget.Library);
        Assert.True(library.Success, string.Join(";", library.Diagnostics));
        var bytes = library.Program!.AssemblyImage.ToArray();
        using (var pe = new PEReader(new MemoryStream(bytes)))
        {
            var reader = pe.GetMetadataReader();
            var type = reader.TypeDefinitions.Select(reader.GetTypeDefinition).Single(type => reader.GetString(type.Name) == "Choice");
            if (corruption == "constructor")
            {
                var method = type.GetMethods().Single(handle => reader.GetString(reader.GetMethodDefinition(handle).Name) == ".ctor");
                var offset = pe.PEHeaders.MetadataStartOffset + reader.GetTableMetadataOffset(TableIndex.MethodDef) +
                    (MetadataTokens.GetRowNumber(method) - 1) * reader.GetTableRowSize(TableIndex.MethodDef) + 6;
                var flags = (ushort)((reader.GetMethodDefinition(method).Attributes & ~MethodAttributes.MemberAccessMask) | MethodAttributes.Public);
                bytes[offset] = (byte)flags; bytes[offset + 1] = (byte)(flags >> 8);
            }
            else
            {
                if (corruption == "case-tag") type = reader.TypeDefinitions.Select(reader.GetTypeDefinition).Single(type => reader.GetString(type.Name) == "BCase");
                var attribute = MetadataAttributes.Named(reader, type.GetCustomAttributes(), corruption == "version" ? "VariantAttribute" : "VariantCaseAttribute");
                var offset = pe.PEHeaders.MetadataStartOffset + reader.GetHeapMetadataOffset(HeapIndex.Blob) + MetadataTokens.GetHeapOffset(attribute.Value);
                bytes[offset + 3] = corruption == "version" ? (byte)2 : (byte)0;
            }
        }
        using var directory = new TemporaryDirectory(); var file = directory.File("Model.dll"); File.WriteAllBytes(file, bytes);
        var result = FpasCompiler.Compile(new([new("consumer.fpas", "program Consumer; uses Model as M; begin discard case M.Choice.A of when M.Choice.A: 1; when M.Choice.B(_): 2; end case; end; end program;")],
            "Consumer", [.. TestEnvironment.References, typeof(Core.VariantAttribute).Assembly.Location, file]));
        Assert.False(result.Success); Assert.Equal("FP4001", Assert.Single(result.Diagnostics).Code);
    }
}
