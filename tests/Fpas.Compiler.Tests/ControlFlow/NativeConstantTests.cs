using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Tests.Support;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;

namespace Fpas.Compiler.Tests.ControlFlow;

/// <summary>Checks portable native-integer constant evaluation independently of the compiler host.</summary>
public sealed class NativeConstantTests
{
    /// <summary>Simulates native integral conversion and arithmetic at both CLR pointer widths.</summary>
    [Fact]
    public void Explicit_width_evaluation_does_not_truncate_to_the_host_pointer_size()
    {
        Assert.Equal(-1L, NumericValue.ConvertTo(4294967295L, TypeSymbol.NativeInt, false, 32));
        Assert.Equal(4294967295L, NumericValue.ConvertTo(4294967295L, TypeSymbol.NativeInt, false, 64));
        Assert.Equal(long.MaxValue, NumericValue.ConvertTo(long.MaxValue, TypeSymbol.NativeInt, true, 64));
        Assert.Equal((ulong)uint.MaxValue, NumericValue.ConvertTo(ulong.MaxValue, TypeSymbol.NativeUInt, false, 32));
        Assert.Equal(ulong.MaxValue, NumericValue.ConvertTo(ulong.MaxValue, TypeSymbol.NativeUInt, true, 64));
        Assert.Equal(-2147483648L, NumericValue.Binary("+", 2147483647L, 1L, TypeSymbol.NativeInt, false, 32));
        Assert.Equal(2147483648L, NumericValue.Binary("+", 2147483647L, 1L, TypeSymbol.NativeInt, false, 64));
        Assert.Equal(0UL, NumericValue.Binary("+", (ulong)uint.MaxValue, 1UL, TypeSymbol.NativeUInt, false, 32));
        Assert.Equal(4294967296UL, NumericValue.Binary("+", (ulong)uint.MaxValue, 1UL, TypeSymbol.NativeUInt, false, 64));
        Assert.Equal(1L, NumericValue.Binary("shl", 1L, 32, TypeSymbol.NativeInt, false, 32));
        Assert.Equal(4294967296L, NumericValue.Binary("shl", 1L, 32, TypeSymbol.NativeInt, false, 64));
    }

    /// <summary>Checks width-specific overflow for integral and floating conversions and arithmetic.</summary>
    [Fact]
    public void Checked_native_operations_observe_the_requested_width()
    {
        Assert.Throws<OverflowException>(() => NumericValue.ConvertTo(4294967296L, TypeSymbol.NativeInt, true, 32));
        Assert.Equal(4294967296L, NumericValue.ConvertTo(4294967296L, TypeSymbol.NativeInt, true, 64));
        Assert.Throws<OverflowException>(() => NumericValue.ConvertTo(4294967296.0, TypeSymbol.NativeInt, true, 32));
        Assert.Equal(4294967296L, NumericValue.ConvertTo(4294967296.0, TypeSymbol.NativeInt, true, 64));
        Assert.Throws<OverflowException>(() => NumericValue.Binary("+", 2147483647L, 1L, TypeSymbol.NativeInt, true, 32));
        Assert.Equal(2147483648L, NumericValue.Binary("+", 2147483647L, 1L, TypeSymbol.NativeInt, true, 64));
    }

    /// <summary>Rejects defaults whose native intermediate results differ or overflow on one architecture.</summary>
    /// <param name="expression">The native expression producing an optional Boolean default.</param>
    [Theory]
    [InlineData("unchecked(cast<nint>(2147483647) + cast<nint>(1)) < 0")]
    [InlineData("cast<nint>(2147483647) + cast<nint>(1) < 0")]
    [InlineData("cast<nint>(1) shl 32 = cast<nint>(1)")]
    [InlineData("cast<nint>(4294967296.0) > 0")]
    public void Architecture_dependent_optional_constants_are_diagnosed(string expression)
    {
        var result = TestCompilation.CompileAgainstFramework($"program Constants; procedure P(Value: bool := {expression}); begin end procedure; begin end; end program;");
        Assert.False(result.Success);
        Assert.Equal("FP3028", Assert.Single(result.Diagnostics).Code);
        Assert.Null(result.Program);
    }

    /// <summary>Checks native portability at scalar, pattern, attribute, and subrange validation boundaries.</summary>
    /// <param name="declarations">Declarations containing or supporting a native constant.</param>
    /// <param name="body">The executable body to validate.</param>
    /// <param name="code">The owning source diagnostic code.</param>
    [Theory]
    [InlineData("type E = enum A = cast<int>(cast<nint>(1) shl 32); end enum;", "", "FP3031")]
    [InlineData("", "discard case cast<nint>(1) of when (cast<nint>(1) shl 32): 1; else 0; end case;", "FP3024")]
    [InlineData("[Sys.ComponentModel.DefaultValueAttribute(cast<long>(cast<nint>(1) shl 32))] procedure P(); begin end procedure;", "", "FP3035")]
    [InlineData("type R = long range 0L..5000000000L;", "discard new R(cast<long>(cast<nint>(1) shl 32));", "FP3031")]
    public void Architecture_dependent_constants_do_not_escape_other_validation_paths(string declarations, string body, string code)
    {
        var result = TestCompilation.Compile($"program Constants; uses System as Sys; {declarations} begin {body} end; end program;");
        Assert.False(result.Success);
        Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
        Assert.Null(result.Program);
    }

    /// <summary>Preserves portable native defaults and runtime native expressions.</summary>
    [Fact]
    public void Portable_native_constants_keep_their_metadata_and_nonconstant_arithmetic_remains_valid()
    {
        var result = TestCompilation.Compile("""
            unit Constants;
            public procedure P(Value: nint := cast<nint>(42); Positive: bool := cast<nint>(10) + cast<nint>(1) = cast<nint>(11));
            begin end procedure;
            public function Runtime(Value: nint): nint;
            begin return unchecked(Value + cast<nint>(1)); end function;
            end unit;
            """, CompilationTarget.Library);
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
        using var pe = new PEReader(new MemoryStream(result.Program!.AssemblyImage));
        var reader = pe.GetMetadataReader();
        var method = reader.MethodDefinitions.Select(reader.GetMethodDefinition).Single(method => reader.GetString(method.Name) == "P");
        var parameters = method.GetParameters().Select(reader.GetParameter).ToArray();
        Assert.Equal(ConstantTypeCode.Int32, reader.GetConstant(parameters[0].GetDefaultValue()).TypeCode);
        var boolean = reader.GetConstant(parameters[1].GetDefaultValue());
        Assert.True(reader.GetBlobReader(boolean.Value).ReadBoolean());
    }
}
