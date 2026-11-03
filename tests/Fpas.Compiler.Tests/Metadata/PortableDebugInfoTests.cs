using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using System.Security.Cryptography;
using System.Text;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Metadata;

public sealed class PortableDebugInfoTests
{
    [Fact]
    public void Portable_symbols_describe_sources_scopes_closures_and_async_resumption()
    {
        const string source = """
            unit DebugSymbols;
            uses System.Threading.Tasks as Tasks;
            public function Scoped(Value: int): int;
            begin
              const Local: int := Value;
              begin const Local: int := 2; discard Local; end;
              const Read: function(): int := function(): int => Local;
              return Read();
            end function;
            public async function Wait(Value: Tasks.Task<int>): Tasks.Task<int>;
            begin
              const Answer: int := await Value;
              return Answer;
            end function;
            end unit;
            """;
        CompilationRequest request = new([new("debug.fpas", source)], "DebugSymbols", TestEnvironment.References, CompilationTarget.Library, EmitDebugInformation: true);
        var result = FpasCompiler.Compile(request);
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
        var program = result.Program!;
        Assert.NotNull(program.PortablePdb);
        using var pe = new PEReader(new MemoryStream(program.AssemblyImage));
        using var provider = MetadataReaderProvider.FromPortablePdbStream(new MemoryStream(program.PortablePdb));
        var reader = pe.GetMetadataReader(); var symbols = provider.GetMetadataReader();
        var document = symbols.GetDocument(Assert.Single(symbols.Documents));
        Assert.Equal("debug.fpas", symbols.GetString(document.Name));
        Assert.Equal(SHA256.HashData(Encoding.UTF8.GetBytes(source)), symbols.GetBlobBytes(document.Hash));
        var codeView = pe.ReadCodeViewDebugDirectoryData(pe.ReadDebugDirectory().Single(entry => entry.Type == DebugDirectoryEntryType.CodeView));
        Assert.Equal(new Guid(symbols.DebugMetadataHeader!.Id.AsSpan()[..16]), codeView.Guid);
        var checksum = pe.ReadPdbChecksumDebugDirectoryData(pe.ReadDebugDirectory().Single(entry => entry.Type == DebugDirectoryEntryType.PdbChecksum));
        Assert.Equal(SHA256.HashData(program.PortablePdb), checksum.Checksum.ToArray());
        var methods = reader.MethodDefinitions.Where(handle => reader.GetString(reader.GetMethodDefinition(handle).Name) != ".ctor")
            .ToDictionary(handle => reader.GetString(reader.GetMethodDefinition(handle).Name), handle => handle);
        var scoped = methods["Scoped"];
        var points = symbols.GetMethodDebugInformation(scoped).GetSequencePoints().ToArray();
        Assert.Contains(points, point => point.StartLine == 5);
        Assert.Contains(points, point => point.StartLine == 8);
        Assert.Contains(points, point => point.IsHidden);
        var locals = symbols.GetLocalScopes(scoped).SelectMany(handle =>
        {
            var scope = symbols.GetLocalScope(handle);
            return scope.GetLocalVariables().Select(local => (Scope: scope, Variable: symbols.GetLocalVariable(local)));
        }).Where(local => symbols.GetString(local.Variable.Name) == "Local").ToArray();
        Assert.Equal(2, locals.Length);
        Assert.NotEqual(locals[0].Variable.Index, locals[1].Variable.Index);
        Assert.True(locals[0].Scope.StartOffset <= locals[1].Scope.StartOffset && locals[0].Scope.EndOffset >= locals[1].Scope.EndOffset);
        Assert.Contains(symbols.GetMethodDebugInformation(methods["<Invoke>"]).GetSequencePoints(), point => point.StartLine == 7);
        var move = methods["MoveNext"];
        var machine = reader.GetTypeDefinition(reader.GetMethodDefinition(move).GetDeclaringType());
        var fields = machine.GetFields().Select(handle => reader.GetString(reader.GetFieldDefinition(handle).Name)).ToArray();
        Assert.Contains("Value", fields);
        Assert.Contains(fields, name => name.StartsWith("<Answer>5__", StringComparison.Ordinal));
        Assert.Equal(methods["Wait"], symbols.GetMethodDebugInformation(move).GetStateMachineKickoffMethod());
        var info = symbols.GetCustomDebugInformation(move).Select(symbols.GetCustomDebugInformation).ToArray();
        var stepping = Assert.Single(info, item => symbols.GetGuid(item.Kind) == new Guid("54fd2ac5-e925-401a-9c2a-f94f171072f8"));
        var blob = symbols.GetBlobReader(stepping.Value);
        Assert.Equal(0, blob.ReadInt32());
        var length = pe.GetMethodBody(reader.GetMethodDefinition(move).RelativeVirtualAddress).GetILContent().Length;
        var yield = blob.ReadInt32(); var resume = blob.ReadInt32();
        Assert.InRange(yield, 0, length - 1); Assert.InRange(resume, yield + 1, length - 1);
        Assert.Equal(MetadataTokens.GetRowNumber(move), blob.ReadCompressedInteger()); Assert.Equal(0, blob.RemainingBytes);
        var repeated = FpasCompiler.Compile(request with { References = request.References.Reverse().ToArray() }).Program!;
        Assert.Equal(program.AssemblyImage, repeated.AssemblyImage); Assert.Equal(program.PortablePdb, repeated.PortablePdb);
        Assert.Null(FpasCompiler.Compile(request with { EmitDebugInformation = false }).Program!.PortablePdb);
    }
}
