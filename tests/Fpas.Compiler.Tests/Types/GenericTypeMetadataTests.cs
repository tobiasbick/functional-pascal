using Fpas.Compiler.Tests.Support;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;

namespace Fpas.Compiler.Tests.Types;

public sealed class GenericTypeMetadataTests
{
    [Fact]
    public void Fields_method_constraints_and_closures_retain_distinct_CLR_generic_slots()
    {
        var result = TestCompilation.Compile("""
            program Metadata;
            type Box<T> = record
              Value: T;
              function Upcast<U>(Other: U): T; where U: T; begin return Other; end function;
              function Capture<U>(Other: U): function(): U;
              begin return function(): U begin discard self.Value; return Other; end function; end function;
            end record;
            begin discard new Box<int>(42).Capture<string>('text')(); end;
            end program;
            """);
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
        using var pe = new PEReader(new MemoryStream(result.Program!.AssemblyImage));
        var reader = pe.GetMetadataReader();
        var box = reader.TypeDefinitions.Select(reader.GetTypeDefinition).Single(type => reader.GetString(type.Name) == "Box`1");
        Assert.Single(box.GetGenericParameters());
        Assert.Equal(new byte[] { 6, 0x13, 0 }, reader.GetBlobBytes(reader.GetFieldDefinition(Assert.Single(box.GetFields())).Signature));
        var upcast = box.GetMethods().Select(reader.GetMethodDefinition).Single(method => reader.GetString(method.Name) == "Upcast");
        Assert.Equal(new byte[] { 0x30, 1, 1, 0x13, 0, 0x1e, 0 }, reader.GetBlobBytes(upcast.Signature));
        var parameter = reader.GetGenericParameter(Assert.Single(upcast.GetGenericParameters()));
        var constraint = reader.GetGenericParameterConstraint(Assert.Single(parameter.GetConstraints()));
        Assert.Equal(new byte[] { 0x13, 0 }, reader.GetBlobBytes(reader.GetTypeSpecification((TypeSpecificationHandle)constraint.Type).Signature));
        var closure = reader.TypeDefinitions.Select(reader.GetTypeDefinition).Single(type => reader.GetString(type.Name).StartsWith("<Closure", StringComparison.Ordinal));
        Assert.Equal(new[] { 0, 1 }, closure.GetGenericParameters().Select(handle => reader.GetGenericParameter(handle).Index));
        var invoke = closure.GetMethods().Select(reader.GetMethodDefinition).Single(method => reader.GetString(method.Name) == "<Invoke>");
        Assert.Equal(new byte[] { 0x20, 0, 0x13, 1 }, reader.GetBlobBytes(invoke.Signature));
    }
}
