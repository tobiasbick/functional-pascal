using Fpas.Compiler.Tests.Support;
using Fpas.Compiler.Analysis;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Types;

public sealed class RecordMetadataTests
{
    [Fact]
    public void Record_argument_labels_follow_field_slots_even_if_CLR_constructor_parameters_have_other_names()
    {
        using var directory = new TemporaryDirectory();
        var path = directory.File("Library.dll");
        const string source = "unit Coordinates; public type Point = record public X: int; public Y: int; end record; end unit;";
        var library = FpasCompiler.Compile(new([new("point.fpas", source)], "Library", TestCompilation.References, CompilationTarget.Library));
        Assert.True(library.Success);
        var bytes = library.Program!.AssemblyImage.ToArray();
        int ctorToken;
        Guid moduleId;
        using (var pe = new PEReader(new MemoryStream(bytes)))
        {
            var reader = pe.GetMetadataReader();
            var point = reader.TypeDefinitions.Select(reader.GetTypeDefinition).Single(type => reader.GetString(type.Name) == "Point");
            var ctor = point.GetMethods().Single(handle => reader.GetString(reader.GetMethodDefinition(handle).Name) == ".ctor");
            ctorToken = MetadataTokens.GetToken(ctor);
            moduleId = reader.GetGuid(reader.GetModuleDefinition().Mvid);
            var rows = reader.GetMethodDefinition(ctor).GetParameters().Where(handle => reader.GetParameter(handle).SequenceNumber > 0).ToArray();
            Assert.Equal(2, rows.Length);
            var table = pe.PEHeaders.MetadataStartOffset + reader.GetTableMetadataOffset(TableIndex.Param);
            var stride = reader.GetTableRowSize(TableIndex.Param);
            // Each Param row ends in a string-heap index following its flags and sequence number.
            Span<byte> Name(ParameterHandle handle) => bytes.AsSpan(table + (MetadataTokens.GetRowNumber(handle) - 1) * stride + 4, stride - 4);
            var x = Name(rows[0]).ToArray();
            Name(rows[1]).CopyTo(Name(rows[0]));
            x.CopyTo(Name(rows[1]));
        }
        const string consumer = "program Test; uses Coordinates as C; begin discard new C.Point(1, 2); end; end program;";
        var request = new AnalysisRequest([new("test.fpas", consumer)], "Test", [.. TestCompilation.References, path],
            ReferenceImages: [new(path, bytes)]);
        var result = FpasCompiler.Analyze(request);
        Assert.True(result.Success, string.Join(";", result.Diagnostics));
        var call = Assert.Single(result.Snapshot!.Operations, op => op.Kind == "construct");
        Assert.Equal(new[] { "X", "Y" }, call.SuggestedArgumentLabels().Select(item => item.Label));
        var template = ApiUsageTemplates.Create(request, "Coordinates+Point", moduleId, ctorToken, [], false);
        Assert.True(template.Source?.Contains("(X := X, Y := Y)", StringComparison.Ordinal) == true, template.Reason + template.Source);
    }

    [Theory]
    [InlineData("version")]
    [InlineData("copy")]
    [InlineData("constructor")]
    public void Invalid_imported_record_contracts_are_rejected_before_emission(string corruption)
    {
        var library = TestCompilation.Compile("unit Model; public type Item = record public Value: int := 1; end record; end unit;", CompilationTarget.Library);
        Assert.True(library.Success);
        var bytes = library.Program!.AssemblyImage.ToArray();
        using (var pe = new PEReader(new MemoryStream(bytes)))
        {
            var reader = pe.GetMetadataReader();
            var type = reader.TypeDefinitions.Select(reader.GetTypeDefinition).Single(type => reader.GetString(type.Name) == "Item");
            if (corruption == "version")
            {
                var attribute = AssemblyInspection.Attribute(reader, type.GetCustomAttributes(), "RecordAttribute");
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
            "Consumer", [.. TestCompilation.References, reference]));
        Assert.False(result.Success);
        Assert.Equal("FP4001", Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Forward_recursive_types_and_multi_unit_records_have_deterministic_metadata()
    {
        CompilationSource[] sources =
        [
            new("model.fpas", "unit Demo.Model; public type Link = record public Next: Link? := null; public Other: Payload? := null; end record; public type Payload = record public Value: int; end record; end unit;"),
            new("api.fpas", "unit Demo.Api; uses Demo.Model as M; public function Make(): M.Link; begin return new M.Link(Other := new M.Payload(2)); end function; end unit;")
        ];
        string[] references = TestCompilation.References;
        var first = FpasCompiler.Compile(new(sources, "Forward", references, CompilationTarget.Library));
        var second = FpasCompiler.Compile(new(sources.Reverse().ToArray(), "Forward", references.Reverse().ToArray(), CompilationTarget.Library));
        Assert.True(first.Success, string.Join(";", first.Diagnostics));
        Assert.True(second.Success, string.Join(";", second.Diagnostics));
        Assert.Equal(first.Program!.AssemblyImage, second.Program!.AssemblyImage);
    }
}
