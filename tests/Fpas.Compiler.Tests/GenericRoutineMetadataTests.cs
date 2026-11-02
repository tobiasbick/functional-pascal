using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;

namespace Fpas.Compiler.Tests;

public sealed class GenericRoutineMetadataTests
{
    [Fact]
    public void Generic_definitions_calls_and_closures_preserve_CLR_type_parameters()
    {
        var result = DiagnosticsTests.Compile("""
            program Metadata;
            function Id<T>(Value: T): T; begin return Value; end function;
            function Hold<T>(Value: T): function(): T; begin return function(): T => Value; end function;
            begin discard Id<int>(42); discard Hold<string>('text')(); end;
            end program;
            """);
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
        using var pe = new PEReader(new MemoryStream(result.Program!.AssemblyImage));
        var reader = pe.GetMetadataReader();
        var identity = reader.MethodDefinitions.Select(reader.GetMethodDefinition).Single(method => reader.GetString(method.Name) == "Id");
        Assert.Single(identity.GetGenericParameters());
        Assert.Equal(new byte[] { 0x10, 1, 1, 0x1e, 0, 0x1e, 0 }, reader.GetBlobBytes(identity.Signature));
        Assert.Equal(2, reader.GetTableRowCount(TableIndex.MethodSpec));
        var specification = reader.GetMethodSpecification(MetadataTokens.MethodSpecificationHandle(1));
        Assert.Equal(HandleKind.MethodDefinition, specification.Method.Kind);
        var closure = reader.TypeDefinitions.Select(reader.GetTypeDefinition).Single(type => reader.GetString(type.Name).StartsWith("<Closure", StringComparison.Ordinal));
        Assert.Single(closure.GetGenericParameters());
        var field = reader.GetFieldDefinition(Assert.Single(closure.GetFields()));
        Assert.Equal(new byte[] { 0x06, 0x13, 0 }, reader.GetBlobBytes(field.Signature));
        Assert.Contains(Enumerable.Range(1, reader.GetTableRowCount(TableIndex.TypeSpec)).Select(MetadataTokens.TypeSpecificationHandle), handle => reader.GetBlobBytes(reader.GetTypeSpecification(handle).Signature).Contains((byte)0x1e));
        Assert.DoesNotContain(reader.AssemblyReferences, handle => reader.GetString(reader.GetAssemblyReference(handle).Name) == "Fpas.Core");
    }
}
