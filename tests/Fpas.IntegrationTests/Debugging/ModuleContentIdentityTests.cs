using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using Fpas.Debugging.Core;
using Fpas.Debugging.Hosting;
using Fpas.Tests.Support;
using Fpas.Workspaces.Processes;

namespace Fpas.IntegrationTests.Debugging;

public sealed class ModuleContentIdentityTests
{
    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task Referenced_constant_changes_with_identical_source_have_distinct_module_identities(bool runtimeSwap)
    {
        using var program = new Debuggee("own-module-content-identity");
        File.WriteAllText(program.Source, """
            program SameSource;
            uses ConstantFixture.Values as Values, System.IO.File as File;
            type Number = int range Values.Lower..100;
            begin
              const Numbers: Number[] := new Number[1];
              File.WriteAllText('executed.txt', 'bad');
            end;
            end program;
            """);
        var variants = new List<string>();
        foreach (var value in new[] { 41, 42 })
        {
            var directory = program.FilePath(value.ToString(System.Globalization.CultureInfo.InvariantCulture)); Directory.CreateDirectory(directory);
            var reference = Path.Combine(directory, "Constants.dll"); File.WriteAllBytes(reference, Constants(value));
            var assembly = Path.Combine(directory, "Program.dll");
            await program.CompileFile(program.Source, assembly, "--reference", reference); variants.Add(assembly);
        }
        var original = File.ReadAllBytes(variants[0]); var replacement = File.ReadAllBytes(variants[1]);
        Assert.NotEqual(original, replacement);
        Assert.Equal(File.ReadAllBytes(Path.ChangeExtension(variants[0], ".pdb")), File.ReadAllBytes(Path.ChangeExtension(variants[1], ".pdb")));
        if (!runtimeSwap)
        {
            Assert.NotEqual(Identity(original), Identity(replacement)); Assert.NotEqual(Guid.Empty, Identity(original));
            var reference = Path.Combine(Path.GetDirectoryName(variants[0])!, "Constants.dll");
            await program.CompileFile(program.Source, variants[0], "--reference", reference);
            Assert.Equal(original, File.ReadAllBytes(variants[0])); return;
        }
        foreach (var extension in new[] { ".dll", ".pdb", ".runtimeconfig.json" })
            File.Copy(Path.ChangeExtension(variants[0], extension), Path.ChangeExtension(program.Assembly, extension));
        File.Copy(Path.Combine(Path.GetDirectoryName(TestEnvironment.Metadata("FpasCliPath"))!, "Fpas.Core.dll"), program.FilePath("Fpas.Core.dll"));
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        await using var host = await DebugSessionHost.Start(WorkerLauncher.ForAssembly(TestEnvironment.Metadata("FpasCliPath"), BuildProcess.DotnetHost()), program.DirectoryPath, deadline.Token);
        await host.Prepare(new(BuildProcess.DotnetHost(), program.Assembly, program.DirectoryPath), deadline.Token);
        File.Move(program.Assembly, program.Assembly + ".preflight"); File.Copy(variants[1], program.Assembly);
        File.Move(Path.ChangeExtension(program.Assembly, ".pdb"), program.Assembly + ".preflight.pdb");
        File.Copy(Path.ChangeExtension(variants[1], ".pdb"), Path.ChangeExtension(program.Assembly, ".pdb"));
        try { await host.Initialize(deadline.Token); } catch (DebugException) when (host.Snapshot.State == "failed") { }
        var stopped = await host.WaitForStop(deadline.Token);
        Assert.Equal("failed", stopped.State); Assert.Equal("module_mismatch", stopped.Error?.Code);
        Assert.False(File.Exists(program.FilePath("executed.txt"))); await host.Stop(deadline.Token);
    }

    private static Guid Identity(byte[] image)
    {
        using var pe = new PEReader(new MemoryStream(image)); var metadata = pe.GetMetadataReader();
        return metadata.GetGuid(metadata.GetModuleDefinition().Mvid);
    }

    private static byte[] Constants(int value)
    {
        var metadata = new MetadataBuilder();
        metadata.AddModule(0, metadata.GetOrAddString("Constants.dll"), metadata.GetOrAddGuid(new Guid(value, 0, 0, new byte[8])), default, default);
        metadata.AddAssembly(metadata.GetOrAddString("Constants"), new Version(1, 0, 0, 0), default, default, 0, AssemblyHashAlgorithm.None);
        var runtime = metadata.AddAssemblyReference(metadata.GetOrAddString("System.Runtime"), new Version(10, 0, 0, 0), default,
            metadata.GetOrAddBlob(new byte[] { 0xb0, 0x3f, 0x5f, 0x7f, 0x11, 0xd5, 0x0a, 0x3a }), 0, default);
        var parent = metadata.AddTypeReference(runtime, metadata.GetOrAddString("System"), metadata.GetOrAddString("Object"));
        metadata.AddTypeDefinition(TypeAttributes.NotPublic, default, metadata.GetOrAddString("<Module>"), default, MetadataTokens.FieldDefinitionHandle(1), MetadataTokens.MethodDefinitionHandle(1));
        metadata.AddTypeDefinition(TypeAttributes.Public | TypeAttributes.Abstract | TypeAttributes.Sealed, metadata.GetOrAddString("ConstantFixture"), metadata.GetOrAddString("Values"), parent,
            MetadataTokens.FieldDefinitionHandle(1), MetadataTokens.MethodDefinitionHandle(1));
        var signature = new BlobBuilder(); signature.WriteByte(6); signature.WriteByte(8);
        var field = metadata.AddFieldDefinition(FieldAttributes.Public | FieldAttributes.Static | FieldAttributes.Literal | FieldAttributes.HasDefault,
            metadata.GetOrAddString("Lower"), metadata.GetOrAddBlob(signature));
        metadata.AddConstant(field, value);
        var image = new BlobBuilder();
        new ManagedPEBuilder(PEHeaderBuilder.CreateLibraryHeader(), new MetadataRootBuilder(metadata), new BlobBuilder(), flags: CorFlags.ILOnly).Serialize(image);
        return image.ToArray();
    }
}
