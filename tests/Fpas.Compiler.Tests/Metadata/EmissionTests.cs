using Fpas.Compiler.Tests.ControlFlow;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Metadata;

public sealed class EmissionTests
{
    private const string Hello = """
        program Game.Hello;
        uses System.Console as Console;
        begin
          const Message: string := 'Hello from FPAS';
          Console.WriteLine(Message);
        end;
        end program;
        """;

    [Fact]
    public void Output_has_a_real_entry_point_and_the_exact_CLR_call_signature()
    {
        var result = DiagnosticsTests.Compile(Hello);
        Assert.True(result.Success, string.Join("; ", result.Diagnostics));
        using var pe = new PEReader(new MemoryStream(result.Program!.AssemblyImage));
        var reader = pe.GetMetadataReader();
        var token = pe.PEHeaders.CorHeader!.EntryPointTokenOrRelativeVirtualAddress;
        var entry = reader.GetMethodDefinition((MethodDefinitionHandle)MetadataTokens.Handle(token));
        Assert.NotEmpty(pe.GetMethodBody(entry.RelativeVirtualAddress).GetILBytes()!);
        var container = reader.TypeDefinitions.Select(reader.GetTypeDefinition).Single(type => reader.GetString(type.Name) == "Hello");
        Assert.Equal("Game", reader.GetString(container.Namespace));
        Assert.True(container.Attributes.HasFlag(TypeAttributes.Public));
        var call = reader.GetMemberReference(Assert.Single(reader.MemberReferences, handle =>
            reader.GetString(reader.GetMemberReference(handle).Name) == "WriteLine"));
        Assert.Equal("WriteLine", reader.GetString(call.Name));
        Assert.Equal(new byte[] { 0x00, 0x01, 0x01, 0x0e }, reader.GetBlobBytes(call.Signature));
        var owner = reader.GetTypeReference((TypeReferenceHandle)call.Parent);
        Assert.Equal("System", reader.GetString(owner.Namespace));
        Assert.Equal("Console", reader.GetString(owner.Name));
        var assembly = reader.GetAssemblyReference((AssemblyReferenceHandle)owner.ResolutionScope);
        Assert.Equal("System.Console", reader.GetString(assembly.Name));
        Assert.Equal(new Version(10, 0, 0, 0), assembly.Version);
        Assert.Equal(["System.Console", "System.Runtime"], reader.AssemblyReferences
            .Select(handle => reader.GetString(reader.GetAssemblyReference(handle).Name)).Order().ToArray());
    }

    [Fact]
    public void Identical_inputs_emit_identical_assemblies_regardless_of_reference_order()
    {
        var first = DiagnosticsTests.Compile(Hello);
        var second = FpasCompiler.Compile(new([new("elsewhere.fpas", Hello)], "Example", TestEnvironment.References.Reverse().ToArray()));
        Assert.True(first.Success);
        Assert.True(second.Success);
        Assert.Equal(first.Program!.AssemblyImage, second.Program!.AssemblyImage);
    }

    [Theory]
    [InlineData("program Empty; begin end; end program;")]
    [InlineData("\uFEFFprogram Escapes; uses System.Console as Console; begin const @end: string := 'It''s 😀'; Console.WriteLine(@end); end; end program;// EOF comment")]
    [InlineData("program Numbers; uses System.Console as Console; begin const Value: int := 2_147_483_647; Console.WriteLine(Value); end; end program;")]
    public void Supported_programs_emit_valid_managed_images(string source)
    {
        var result = DiagnosticsTests.Compile(source);
        Assert.True(result.Success, string.Join("; ", result.Diagnostics));
        using var pe = new PEReader(new MemoryStream(result.Program!.AssemblyImage));
        Assert.True(pe.HasMetadata);
        Assert.NotEqual(0, pe.PEHeaders.CorHeader!.EntryPointTokenOrRelativeVirtualAddress);
    }
}
