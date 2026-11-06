using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Execution;

public sealed class RectangularInteropTests
{
    [Fact]
    public async Task A_csharp_consumer_calls_fpas_rectangular_array_returns_and_generic_parameters()
    {
        using var directory = new TemporaryDirectory();
        var source = directory.File("GridBridge.fpas"); var library = directory.File("GridBridge.dll");
        File.WriteAllText(source, """
            unit GridBridge;
            public function Echo<T>(Grid: T[,]): T[,]; begin return Grid; end function;
            public function Create(): int[,];
            begin
              const Grid: int[,] := new int[2, 3];
              Grid[1, 2] := 42;
              return Grid;
            end function;
            end unit;
            """);
        var compiled = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", source, "--output", library,
            "--target", "library", "--reference-dir", TestEnvironment.ReferenceDirectory]);
        Assert.True(compiled.ExitCode == 0, compiled.Output + compiled.Error);
        File.Copy(Path.Combine(Path.GetDirectoryName(TestEnvironment.Metadata("FpasCliPath"))!, "Fpas.Core.dll"), directory.File("Fpas.Core.dll"));
        File.WriteAllText(directory.File("Consumer.csproj"), """
            <Project Sdk="Microsoft.NET.Sdk">
              <PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><NuGetAudit>false</NuGetAudit></PropertyGroup>
              <ItemGroup>
                <Reference Include="GridBridge"><HintPath>GridBridge.dll</HintPath></Reference>
                <Reference Include="Fpas.Core"><HintPath>Fpas.Core.dll</HintPath></Reference>
              </ItemGroup>
            </Project>
            """);
        File.WriteAllText(directory.File("Consumer.cs"), "System.Console.WriteLine(GridBridge.Echo<int>(GridBridge.Create())[1, 2]);");
        var build = await ProcessRunner.RunAsync(["build", directory.File("Consumer.csproj"), "--configuration", "Release", "--nologo", "--verbosity", "quiet"],
            workingDirectory: directory.Path);
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var executed = await ProcessRunner.RunAsync([directory.File("bin/Release/net10.0/Consumer.dll")], workingDirectory: directory.Path);
        Assert.True(executed.ExitCode == 0, executed.Output + executed.Error); Assert.Equal("42", executed.Output.Trim());
    }

    [Theory]
    [InlineData("canonical", true)]
    [InlineData("missing-bounds", false)]
    [InlineData("nonzero-bounds", false)]
    [InlineData("fixed-size", false)]
    public async Task Imported_rectangular_signatures_bind_exactly_or_fail_before_execution(string shape, bool supported)
    {
        using var directory = new TemporaryDirectory();
        var library = directory.File("Shapes.dll"); File.WriteAllBytes(library, Library(shape));
        var assembly = directory.File("Consumer.dll");
        var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", "-", "--output", assembly,
            "--reference-dir", TestEnvironment.ReferenceDirectory, "--reference", library, "--diagnostics", "json"], """
            program Consumer;
            uses ShapeFixture.Producer as Producer, System.Console as Console, System as Sys;
            begin
              const Grid: int[,] := new int[1, 2];
              Grid[0, 1] := 42;
              const Result: int[,]? := Producer.Echo(Grid);
              if Result = null then raise new Sys.InvalidOperationException('Missing array'); end if;
              Console.WriteLine(Result[0, 1]);
              discard Producer.Create();
            end;
            end program;
            """);
        if (!supported)
        {
            Assert.Equal(1, result.ExitCode); Assert.Contains("FP3005", result.Output); Assert.Contains("array signature", result.Output);
            Assert.False(File.Exists(assembly)); return;
        }
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        File.Copy(Path.Combine(Path.GetDirectoryName(TestEnvironment.Metadata("FpasCliPath"))!, "Fpas.Core.dll"), directory.File("Fpas.Core.dll"));
        var executed = await ProcessRunner.RunAsync([assembly], workingDirectory: directory.Path);
        Assert.True(executed.ExitCode == 0, executed.Output + executed.Error); Assert.Equal("42", executed.Output.Trim());
    }

    private static byte[] Library(string shape)
    {
        var metadata = new MetadataBuilder();
        metadata.AddModule(0, metadata.GetOrAddString("Shapes.dll"), metadata.GetOrAddGuid(Guid.Empty), default, default);
        metadata.AddAssembly(metadata.GetOrAddString("Shapes"), new Version(1, 0), default, default, 0, AssemblyHashAlgorithm.None);
        var runtime = metadata.AddAssemblyReference(metadata.GetOrAddString("System.Runtime"), new Version(10, 0, 0, 0), default,
            metadata.GetOrAddBlob(new byte[] { 0xb0, 0x3f, 0x5f, 0x7f, 0x11, 0xd5, 0x0a, 0x3a }), 0, default);
        var objectType = metadata.AddTypeReference(runtime, metadata.GetOrAddString("System"), metadata.GetOrAddString("Object"));
        metadata.AddTypeDefinition(TypeAttributes.NotPublic, default, metadata.GetOrAddString("<Module>"), default,
            MetadataTokens.FieldDefinitionHandle(1), MetadataTokens.MethodDefinitionHandle(1));
        metadata.AddTypeDefinition(TypeAttributes.Public | TypeAttributes.Abstract | TypeAttributes.Sealed, metadata.GetOrAddString("ShapeFixture"),
            metadata.GetOrAddString("Producer"), objectType, MetadataTokens.FieldDefinitionHandle(1), MetadataTokens.MethodDefinitionHandle(1));
        var bodies = new BlobBuilder(); var encoder = new MethodBodyStreamEncoder(bodies);
        AddMethod("Echo", true); AddMethod("Create", false);
        var image = new BlobBuilder();
        new ManagedPEBuilder(PEHeaderBuilder.CreateLibraryHeader(), new MetadataRootBuilder(metadata), bodies, flags: CorFlags.ILOnly).Serialize(image);
        return image.ToArray();

        void AddMethod(string name, bool parameter)
        {
            var signature = new BlobBuilder(); signature.WriteByte(0); signature.WriteCompressedInteger(parameter ? 1 : 0);
            ArrayType(signature); if (parameter) ArrayType(signature);
            var instructions = new BlobBuilder(); var il = new InstructionEncoder(instructions);
            if (parameter) il.LoadArgument(0); else il.OpCode(ILOpCode.Ldnull);
            il.OpCode(ILOpCode.Ret);
            var first = MetadataTokens.ParameterHandle(metadata.GetRowCount(TableIndex.Param) + 1);
            if (parameter) metadata.AddParameter(ParameterAttributes.None, metadata.GetOrAddString("value"), 1);
            metadata.AddMethodDefinition(MethodAttributes.Public | MethodAttributes.Static | MethodAttributes.HideBySig, MethodImplAttributes.IL,
                metadata.GetOrAddString(name), metadata.GetOrAddBlob(signature), encoder.AddMethodBody(il, maxStack: 1), first);
        }

        void ArrayType(BlobBuilder signature)
        {
            signature.WriteByte(0x14); signature.WriteByte(0x08); signature.WriteCompressedInteger(2);
            signature.WriteCompressedInteger(shape == "fixed-size" ? 1 : 0);
            if (shape == "fixed-size") signature.WriteCompressedInteger(3);
            signature.WriteCompressedInteger(shape == "missing-bounds" ? 0 : 2);
            if (shape != "missing-bounds")
            {
                signature.WriteCompressedSignedInteger(shape == "nonzero-bounds" ? -2 : 0);
                signature.WriteCompressedSignedInteger(0);
            }
        }
    }
}
