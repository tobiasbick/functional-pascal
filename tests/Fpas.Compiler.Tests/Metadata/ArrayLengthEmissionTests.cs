using Fpas.Compiler.Tests.Support;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;

namespace Fpas.Compiler.Tests.Metadata;

public sealed class ArrayLengthEmissionTests
{
    [Fact]
    public void Rectangular_array_length_uses_Array_Length_instead_of_the_vector_ldlen()
    {
        const string source = """
            program Lengths;
            uses System.Console as Console;
            begin
              const Grid: array[,] of int := new int[2, 3];
              Console.WriteLine(Grid.Length);
            end;
            end program;
            """;
        var result = TestCompilation.CompileAgainstFramework(source);
        Assert.True(result.Success, string.Join("; ", result.Diagnostics));
        using var pe = new PEReader(new MemoryStream(result.Program!.AssemblyImage));
        var reader = pe.GetMetadataReader();
        var getter = reader.GetMemberReference(Assert.Single(reader.MemberReferences, handle =>
            reader.GetString(reader.GetMemberReference(handle).Name) == "get_Length"));
        var owner = reader.GetTypeReference((TypeReferenceHandle)getter.Parent);
        Assert.Equal("System", reader.GetString(owner.Namespace));
        Assert.Equal("Array", reader.GetString(owner.Name));
    }
}
