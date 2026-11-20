// [Documentation](../../../docs/specs/clr-mapping.md)
using Fpas.Compiler.Tests.Support;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;

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
        var result = TestCompilation.CompileAgainstFramework(Hello);
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

    [Theory]
    [InlineData("program Empty; begin end; end program;")]
    [InlineData("\uFEFFprogram Escapes; uses System.Console as Console; begin const @end: string := 'It''s 😀'; Console.WriteLine(@end); end; end program;// EOF comment")]
    [InlineData("program Numbers; uses System.Console as Console; begin const Value: int := 2_147_483_647; Console.WriteLine(Value); end; end program;")]
    public void Supported_programs_emit_valid_managed_images(string source)
    {
        var result = TestCompilation.CompileAgainstFramework(source);
        Assert.True(result.Success, string.Join("; ", result.Diagnostics));
        using var pe = new PEReader(new MemoryStream(result.Program!.AssemblyImage));
        Assert.True(pe.HasMetadata);
        Assert.NotEqual(0, pe.PEHeaders.CorHeader!.EntryPointTokenOrRelativeVirtualAddress);
    }

    [Fact]
    public void Embedded_resources_have_distinct_offsets_and_contribute_to_image_identity()
    {
        CompilationResource[] resources = [new("Text.second", [1, 2, 3]), new("Text.first", [12, 13])];
        var request = new CompilationRequest([new("hello.fpas", Hello)], "Hello", TestCompilation.References, Resources: resources);
        byte[] Compile(CompilationRequest input)
        {
            var result = FpasCompiler.Compile(input);
            Assert.True(result.Success, string.Join("; ", result.Diagnostics));
            return result.Program!.AssemblyImage;
        }

        var image = Compile(request);
        Assert.Equal(image, Compile(request with { Resources = resources.Reverse().ToArray() }));
        using var pe = new PEReader(new MemoryStream(image));
        var metadata = pe.GetMetadataReader();
        var data = pe.GetSectionData(pe.PEHeaders.CorHeader!.ResourcesDirectory.RelativeVirtualAddress);
        foreach (var resource in resources)
        {
            var entry = metadata.ManifestResources.Select(metadata.GetManifestResource)
                .Single(entry => metadata.GetString(entry.Name) == resource.Name);
            Assert.True(entry.Implementation.IsNil);
            var blob = data.GetReader();
            blob.Offset = checked((int)entry.Offset);
            Assert.Equal(resource.Content, blob.ReadBytes(blob.ReadInt32()));
        }
        using var changed = new PEReader(new MemoryStream(Compile(request with { Resources = [resources[0], new("Text.first", [12, 14])] })));
        var changedMetadata = changed.GetMetadataReader();
        Assert.NotEqual(metadata.GetGuid(metadata.GetModuleDefinition().Mvid),
            changedMetadata.GetGuid(changedMetadata.GetModuleDefinition().Mvid));
    }

    [Fact]
    public void Duplicate_manifest_resource_names_fail_without_an_image()
    {
        var result = FpasCompiler.Compile(new([new("hello.fpas", Hello)], "Hello", TestCompilation.References,
            Resources: [new("Duplicate", []), new("Duplicate", [42])]));
        Assert.Null(result.Program);
        Assert.Equal("FP4004", Assert.Single(result.Diagnostics).Code);
    }
}
