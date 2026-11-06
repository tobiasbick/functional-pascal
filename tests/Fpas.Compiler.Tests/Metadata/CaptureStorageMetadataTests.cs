using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Metadata;

public sealed class CaptureStorageMetadataTests
{
    [Fact]
    public void Portable_storage_contract_distinguishes_mutable_cells_from_user_strong_boxes()
    {
        const string source = """
            unit CaptureStorage;
            uses System.Runtime.CompilerServices as Runtime, System.Threading.Tasks as Tasks;
            public function Factory(Seed: int): function(): int;
            begin
              var Value: int := Seed;
              const Box: Runtime.StrongBox<int> := new Runtime.StrongBox<int>(70);
              return function(): int => Value + Box.Value;
            end function;
            public async function Observe(): Tasks.Task<int>;
            begin
              var Value: int := 40;
              const Read: function(): int := function(): int => Value;
              await Tasks.Task.Delay(1);
              return Read();
            end function;
            end unit;
            """;
        var request = new CompilationRequest([new("captures.fpas", source)], "CaptureStorage", TestEnvironment.References,
            CompilationTarget.Library, EmitDebugInformation: true);
        var result = FpasCompiler.Compile(request); Assert.True(result.Success, string.Join(';', result.Diagnostics));
        using var image = new PEReader(new MemoryStream(result.Program!.AssemblyImage)); var metadata = image.GetMetadataReader();
        using var provider = MetadataReaderProvider.FromPortablePdbStream(new MemoryStream(result.Program.PortablePdb!)); var pdb = provider.GetMetadataReader();
        var entries = pdb.CustomDebugInformation.Select(pdb.GetCustomDebugInformation)
            .Where(info => pdb.GetGuid(info.Kind) == new Guid("4C590C68-DC5F-4A9D-89A5-B8CE5B937137")).Select(info =>
            {
                var blob = pdb.GetBlobReader(info.Value); Assert.Equal(1u, blob.ReadUInt32()); var flags = blob.ReadByte();
                Assert.InRange(flags, (byte)0, (byte)1); var name = blob.ReadSerializedString(); Assert.Equal(0, blob.RemainingBytes);
                var storage = info.Parent.Kind == HandleKind.FieldDefinition
                    ? metadata.GetString(metadata.GetFieldDefinition((FieldDefinitionHandle)info.Parent).Name)
                    : pdb.GetString(pdb.GetLocalVariable((LocalVariableHandle)info.Parent).Name);
                return (Kind: info.Parent.Kind, Storage: storage, Name: name, Cell: flags == 1);
            }).ToArray();
        Assert.Contains(entries, item => item.Kind == HandleKind.LocalVariable && item.Name == "Value" && item.Cell);
        Assert.Contains(entries, item => item.Kind == HandleKind.FieldDefinition && item.Storage == "Value" && item.Name == "Value" && item.Cell);
        Assert.Contains(entries, item => item.Kind == HandleKind.FieldDefinition && item.Storage == "Box" && item.Name == "Box" && !item.Cell);
        Assert.Contains(entries, item => item.Kind == HandleKind.FieldDefinition && item.Storage.StartsWith("<Value>5__", StringComparison.Ordinal) && item.Name == "Value" && item.Cell);
        Assert.DoesNotContain(entries, item => item.Kind == HandleKind.LocalVariable && item.Name == "Box");
        var repeated = FpasCompiler.Compile(request).Program!;
        Assert.Equal(result.Program.AssemblyImage, repeated.AssemblyImage); Assert.Equal(result.Program.PortablePdb, repeated.PortablePdb);
    }
}
