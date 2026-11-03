using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Routines;

public sealed class EventMetadataTests
{
    [Theory]
    [InlineData("subscribe", "private-add", false)]
    [InlineData("unsubscribe", "private-add", true)]
    [InlineData("subscribe", "missing-remove", true)]
    [InlineData("unsubscribe", "missing-remove", false)]
    [InlineData("subscribe", "wrong-parameter", false)]
    [InlineData("unsubscribe", "wrong-parameter", true)]
    [InlineData("subscribe", "", true)]
    public void Each_accessor_is_selected_by_metadata_and_validated_independently(string operation, string shape, bool success)
    {
        using var directory = new TemporaryDirectory();
        var reference = directory.File("Contracts.dll"); File.WriteAllBytes(reference, Contract(shape));
        var result = FpasCompiler.Compile(new([new("consumer.fpas", $$"""
            program Consumer;
            uses Contracts as C, System as Sys;
            procedure Wire(Value: C.IEvents; Handler: Sys.Action);
            begin {{operation}} Value.Changed with Handler; end procedure;
            begin end;
            end program;
            """)], "Consumer", [.. TestEnvironment.References, reference]));
        Assert.Equal(success, result.Success);
        if (!success) { Assert.Equal("FP3034", Assert.Single(result.Diagnostics).Code); return; }
        using var pe = new PEReader(new MemoryStream(result.Program!.AssemblyImage));
        var reader = pe.GetMetadataReader();
        Assert.Contains(reader.MemberReferences, handle => reader.GetString(reader.GetMemberReference(handle).Name) == (operation == "subscribe" ? "Attach" : "Detach"));
    }

    private static byte[] Contract(string shape)
    {
        var metadata = new MetadataBuilder();
        metadata.AddModule(0, metadata.GetOrAddString("Contracts.dll"), metadata.GetOrAddGuid(Guid.Empty), default, default);
        metadata.AddAssembly(metadata.GetOrAddString("Contracts"), new Version(1, 0, 0, 0), default, default, 0, AssemblyHashAlgorithm.None);
        var runtime = metadata.AddAssemblyReference(metadata.GetOrAddString("System.Runtime"), new Version(10, 0, 0, 0), default,
            metadata.GetOrAddBlob(new byte[] { 0xb0, 0x3f, 0x5f, 0x7f, 0x11, 0xd5, 0x0a, 0x3a }), 0, default);
        var action = metadata.AddTypeReference(runtime, metadata.GetOrAddString("System"), metadata.GetOrAddString("Action"));
        metadata.AddTypeDefinition(TypeAttributes.NotPublic, default, metadata.GetOrAddString("<Module>"), default,
            MetadataTokens.FieldDefinitionHandle(1), MetadataTokens.MethodDefinitionHandle(1));
        var contract = metadata.AddTypeDefinition(TypeAttributes.Public | TypeAttributes.Interface | TypeAttributes.Abstract,
            metadata.GetOrAddString("Contracts"), metadata.GetOrAddString("IEvents"), default,
            MetadataTokens.FieldDefinitionHandle(1), MetadataTokens.MethodDefinitionHandle(1));
        MethodDefinitionHandle Accessor(string name, bool add)
        {
            var signature = new BlobBuilder();
            new BlobEncoder(signature).MethodSignature(isInstanceMethod: true).Parameters(1, result => result.Void(), parameters =>
            {
                var parameter = parameters.AddParameter().Type();
                if (add && shape == "wrong-parameter") parameter.Int32(); else parameter.Type(action, isValueType: false);
            });
            var parameterHandle = metadata.AddParameter(ParameterAttributes.None, metadata.GetOrAddString("handler"), 1);
            var access = add && shape == "private-add" ? MethodAttributes.Private : MethodAttributes.Public;
            return metadata.AddMethodDefinition(access | MethodAttributes.Abstract | MethodAttributes.Virtual | MethodAttributes.NewSlot | MethodAttributes.SpecialName,
                MethodImplAttributes.IL, metadata.GetOrAddString(name), metadata.GetOrAddBlob(signature), -1, parameterHandle);
        }
        var attach = Accessor("Attach", true); var detach = Accessor("Detach", false);
        var changed = metadata.AddEvent(EventAttributes.None, metadata.GetOrAddString("Changed"), action);
        metadata.AddEventMap(contract, changed);
        metadata.AddMethodSemantics(changed, MethodSemanticsAttributes.Adder, attach);
        if (shape != "missing-remove") metadata.AddMethodSemantics(changed, MethodSemanticsAttributes.Remover, detach);
        var image = new BlobBuilder();
        new ManagedPEBuilder(PEHeaderBuilder.CreateLibraryHeader(), new MetadataRootBuilder(metadata), new BlobBuilder(), flags: CorFlags.ILOnly).Serialize(image);
        return image.ToArray();
    }
}
