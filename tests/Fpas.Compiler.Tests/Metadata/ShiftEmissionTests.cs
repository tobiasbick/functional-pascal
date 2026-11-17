using Fpas.Compiler.Tests.Support;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Runtime.Loader;

namespace Fpas.Compiler.Tests.Metadata;

/// <summary>Checks explicit portable CIL masks and CLR execution of integral shifts.</summary>
public sealed class ShiftEmissionTests
{
    /// <summary>Executes shifts at and beyond the operand width and checks their emitted masks.</summary>
    /// <param name="type">The FPAS integral operand type.</param>
    [Theory]
    [InlineData("int")]
    [InlineData("uint")]
    [InlineData("long")]
    [InlineData("ulong")]
    [InlineData("nint")]
    [InlineData("nuint")]
    public void Shift_counts_are_explicitly_masked_to_the_runtime_operand_width(string type)
    {
        var result = TestCompilation.Compile($$"""
            unit Shifts;
            public function Left(Value: {{type}}; Count: int): {{type}};
            begin return Value shl Count; end function;
            public function Right(Value: {{type}}; Count: int): {{type}};
            begin return Value shr Count; end function;
            end unit;
            """, CompilationTarget.Library);
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
        using (var pe = new PEReader(new MemoryStream(result.Program!.AssemblyImage)))
        {
            var reader = pe.GetMetadataReader();
            foreach (var method in reader.MethodDefinitions.Select(reader.GetMethodDefinition))
            {
                var left = reader.GetString(method.Name) == "Left";
                var opcode = left ? (byte)0x62 : type is "uint" or "ulong" or "nuint" ? (byte)0x64 : (byte)0x63;
                byte[] mask = type is "nint" or "nuint" ? [0x1e, 0x5a, 0x17, 0x59, 0x5f, opcode] :
                    [0x1f, type is "long" or "ulong" ? (byte)63 : (byte)31, 0x5f, opcode];
                Assert.True(pe.GetMethodBody(method.RelativeVirtualAddress).GetILBytes()!.AsSpan().IndexOf(mask) >= 0);
            }
        }
        var context = new AssemblyLoadContext("shift-masks", isCollectible: true);
        try
        {
            var assembly = context.LoadFromStream(new MemoryStream(result.Program!.AssemblyImage));
            var owner = assembly.GetType("Shifts")!;
            object value = type switch
            {
                "int" => -9, "uint" => uint.MaxValue - 8, "long" => -9L, "ulong" => ulong.MaxValue - 8,
                "nint" => (nint)(-9), _ => nuint.MaxValue - 8
            };
            foreach (var count in new[] { -1, 0, 1, 31, 32, 33, 63, 64, 65, int.MinValue, int.MaxValue })
            {
                Assert.Equal(Shift(value, count, true), owner.GetMethod("Left")!.Invoke(null, [value, count]));
                Assert.Equal(Shift(value, count, false), owner.GetMethod("Right")!.Invoke(null, [value, count]));
            }
        }
        finally { context.Unload(); }
    }

    private static object Shift(object value, int count, bool left) => value switch
    {
        int number => left ? number << count : number >> count,
        uint number => left ? number << count : number >> count,
        long number => left ? number << count : number >> count,
        ulong number => left ? number << count : number >> count,
        nint number => left ? number << count : number >> count,
        nuint number => left ? number << count : number >> count,
        _ => throw new InvalidOperationException()
    };
}
