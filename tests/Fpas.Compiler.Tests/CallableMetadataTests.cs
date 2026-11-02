using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests;

public sealed class CallableMetadataTests
{
    private const string Source = """
        program Metadata;
        uses System.Linq.Enumerable as Seq, System.Collections.Generic as C;
        function Mapper(Offset: int): function(int): int;
        begin
          var Calls: int := 0;
          return function(Value: int): int begin Calls := Calls + 1; return Value + Offset + Calls; end function;
        end function;
        begin
          const Values: C.IEnumerable<int> := Seq.Select<int, int>([1, 2], Mapper(3));
          for Value: int in Values do discard Value; end for;
        end;
        end program;
        """;

    [Fact]
    public void Generic_calls_and_closures_have_real_CLR_metadata_and_no_language_runtime_dependency()
    {
        var result = DiagnosticsTests.Compile(Source);
        Assert.True(result.Success, string.Join("; ", result.Diagnostics));
        using var pe = new PEReader(new MemoryStream(result.Program!.AssemblyImage));
        var reader = pe.GetMetadataReader();
        Assert.Equal(1, reader.GetTableRowCount(TableIndex.MethodSpec));
        var specification = reader.GetMethodSpecification(MetadataTokens.MethodSpecificationHandle(1));
        Assert.Equal(HandleKind.MemberReference, specification.Method.Kind);
        var method = reader.GetMemberReference((MemberReferenceHandle)specification.Method);
        Assert.Equal("Select", reader.GetString(method.Name));
        Assert.Equal(new byte[] { 0x0a, 2, 0x08, 0x08 }, reader.GetBlobBytes(specification.Signature));
        var signature = reader.GetBlobReader(method.Signature);
        Assert.True(signature.ReadSignatureHeader().IsGeneric);
        Assert.Equal(2, signature.ReadCompressedInteger());
        var closure = reader.TypeDefinitions.Select(reader.GetTypeDefinition).Single(type => reader.GetString(type.Name).StartsWith("<Closure", StringComparison.Ordinal));
        Assert.Equal(TypeAttributes.NestedPrivate, closure.Attributes & TypeAttributes.VisibilityMask);
        Assert.Equal("Metadata", reader.GetString(reader.GetTypeDefinition(closure.GetDeclaringType()).Name));
        Assert.Equal(2, closure.GetMethods().Count);
        Assert.Equal(2, closure.GetFields().Count);
        Assert.All(closure.GetFields(), handle => Assert.True(reader.GetFieldDefinition(handle).Attributes.HasFlag(FieldAttributes.InitOnly)));
        Assert.Contains(reader.TypeReferences, handle => reader.GetString(reader.GetTypeReference(handle).Name) == "StrongBox`1");
        Assert.DoesNotContain(reader.AssemblyReferences, handle => reader.GetString(reader.GetAssemblyReference(handle).Name) == "Fpas.Core");
    }

    [Fact]
    public void Closure_layout_and_generic_instantiations_are_deterministic()
    {
        var first = DiagnosticsTests.Compile(Source);
        var second = FpasCompiler.Compile(new([new("elsewhere.fpas", Source)], "Example", TestEnvironment.References.Reverse().ToArray()));
        Assert.True(first.Success);
        Assert.True(second.Success);
        Assert.Equal(first.Program!.AssemblyImage, second.Program!.AssemblyImage);
    }
}
