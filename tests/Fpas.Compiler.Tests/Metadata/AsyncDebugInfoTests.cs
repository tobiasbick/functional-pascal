using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Metadata;

public sealed class AsyncDebugInfoTests
{
    [Fact]
    public void Hoisted_local_scopes_cover_their_statements_and_exclude_the_following_block()
    {
        const string source = """
            unit ScopedSymbols;
            uses System.Threading.Tasks as Tasks;
            public async function Wait(Value: Tasks.Task<int>; Other: Tasks.Task<int>): Tasks.Task<int>;
            begin
              const Value_1: int := 40;
              begin
                const Value: int := 2;
                const Inner: int := await Other;
                discard Value + Inner;
              end;
              const Final: int := await Value;
              return Value_1 + Final;
            end function;
            end unit;
            """;
        CompilationRequest request = new([new("scoped.fpas", source)], "ScopedSymbols", TestEnvironment.References,
            CompilationTarget.Library, EmitDebugInformation: true);
        var result = FpasCompiler.Compile(request);
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
        var program = result.Program!;
        using var pe = new PEReader(new MemoryStream(program.AssemblyImage));
        using var provider = MetadataReaderProvider.FromPortablePdbStream(new MemoryStream(program.PortablePdb!));
        var reader = pe.GetMetadataReader(); var symbols = provider.GetMetadataReader();
        var move = Assert.Single(reader.MethodDefinitions, handle => reader.GetString(reader.GetMethodDefinition(handle).Name) == "MoveNext");
        var fields = reader.GetTypeDefinition(reader.GetMethodDefinition(move).GetDeclaringType()).GetFields()
            .Select(handle => reader.GetString(reader.GetFieldDefinition(handle).Name)).ToArray();
        Assert.Contains(fields, name => name.StartsWith("<Value_1>5__", StringComparison.Ordinal));
        Assert.Contains(fields, name => name.StartsWith("<Value_2>5__", StringComparison.Ordinal));
        Assert.DoesNotContain(fields, name => name.StartsWith("<Value>5__", StringComparison.Ordinal));
        var hoisted = Assert.Single(symbols.GetCustomDebugInformation(move).Select(symbols.GetCustomDebugInformation),
            info => symbols.GetGuid(info.Kind) == new Guid("6da9a61e-f8c7-4874-be62-68bc5630df71"));
        var blob = symbols.GetBlobReader(hoisted.Value);
        var ranges = new List<(int Start, int End)>();
        while (blob.RemainingBytes > 0)
        {
            var start = blob.ReadInt32(); ranges.Add((start, start + blob.ReadInt32()));
        }
        var points = symbols.GetMethodDebugInformation(move).GetSequencePoints().Where(point => !point.IsHidden).ToArray();
        var inside = points.Where(point => point.StartLine == 9).ToArray();
        var after = points.Where(point => point.StartLine == 12).ToArray();
        Assert.NotEmpty(inside); Assert.NotEmpty(after);
        foreach (var field in fields.Where(name => name.StartsWith("<Value_2>5__", StringComparison.Ordinal) || name.StartsWith("<Inner>5__", StringComparison.Ordinal)))
        {
            var slot = int.Parse(field[(field.LastIndexOf("__", StringComparison.Ordinal) + 2)..], System.Globalization.CultureInfo.InvariantCulture) - 1;
            var range = ranges[slot];
            Assert.All(inside, point => Assert.InRange(point.Offset, range.Start, range.End - 1));
            Assert.All(after, point => Assert.False(point.Offset >= range.Start && point.Offset < range.End));
        }
        foreach (var field in fields.Where(name => name.StartsWith("<>s__", StringComparison.Ordinal)))
        {
            var slot = int.Parse(field[5..], System.Globalization.CultureInfo.InvariantCulture) - 1;
            Assert.Equal((0, 0), ranges[slot]);
        }
        var repeated = FpasCompiler.Compile(request).Program!;
        Assert.Equal(program.AssemblyImage, repeated.AssemblyImage); Assert.Equal(program.PortablePdb, repeated.PortablePdb);
    }
}
