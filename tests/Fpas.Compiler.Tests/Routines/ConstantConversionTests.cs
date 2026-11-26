// [Documentation](../../../docs/pascal/language/basics/conversions.md)
using Fpas.Compiler.Tests.Support;
using Fpas.Tests.Support;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;

namespace Fpas.Compiler.Tests.Routines;

/// <summary>Checks which CLR conversions can participate in source constants.</summary>
public sealed class ConstantConversionTests
{
    /// <summary>Rejects signed remainder expressions whose CLR evaluation overflows.</summary>
    /// <param name="type">The signed integral type of the parameter.</param>
    /// <param name="expression">The overflowing expression supplied as its default.</param>
    [Theory]
    [InlineData("int", "-2147483648 mod -1")]
    [InlineData("long", "unchecked(-9223372036854775808L mod -1L)")]
    public void Signed_remainder_overflow_is_rejected_in_optional_defaults(string type, string expression)
    {
        var result = TestCompilation.CompileAgainstFramework($"program Constants; procedure P(Value: {type} := {expression}); begin end procedure; begin end; end program;");
        Assert.Null(result.Program);
        Assert.Equal("FP3028", Assert.Single(result.Diagnostics).Code);
    }

    /// <summary>Rejects operations that must unwrap runtime storage in optional defaults.</summary>
    /// <param name="expression">The runtime conversion used as an optional default.</param>
    [Theory]
    [InlineData("cast cast 1L as object as int")]
    [InlineData("cast cast null as int? as int")]
    public void Runtime_unboxing_and_nullable_unwrapping_are_not_optional_constants(string expression)
    {
        var result = TestCompilation.CompileAgainstFramework($"program Constants; procedure P(Value: int := {expression}); begin end procedure; begin end; end program;");
        Assert.False(result.Success);
        Assert.Equal("FP3028", Assert.Single(result.Diagnostics).Code);
        Assert.Null(result.Program);
    }

    /// <summary>Rejects reference downcasts whose successful runtime result is unknown.</summary>
    [Fact]
    public void Runtime_reference_downcasts_are_not_optional_string_constants()
    {
        var result = TestCompilation.CompileAgainstFramework("program Constants; procedure P(Value: string := cast cast 1 as object as string); begin end procedure; begin end; end program;");
        Assert.False(result.Success);
        Assert.Equal("FP3028", Assert.Single(result.Diagnostics).Code);
    }

    /// <summary>Rejects reference equality whose result depends on runtime object identity.</summary>
    /// <param name="expression">The boxed reference comparison used as a default.</param>
    [Theory]
    [InlineData("cast cast 1 as nint as object = cast 1L as object")]
    [InlineData("cast cast 1 as nuint as object = cast 1UL as object")]
    [InlineData("cast 1 as object = cast 1 as object")]
    [InlineData("cast ('a' + 'b') as object = cast 'ab' as object")]
    public void Boxed_reference_identity_is_not_a_compile_time_value_comparison(string expression)
    {
        var result = TestCompilation.CompileAgainstFramework($"program Constants; procedure P(Value: bool := {expression}); begin end procedure; begin end; end program;");
        Assert.False(result.Success);
        Assert.Equal("FP3028", Assert.Single(result.Diagnostics).Code);
        Assert.Null(result.Program);
    }

    /// <summary>Preserves boxed null tests and ordinary string value equality in constants.</summary>
    [Fact]
    public void Null_tests_and_string_value_equality_remain_constant()
    {
        var result = TestCompilation.CompileAgainstFramework("program Constants; procedure P(Present: bool := cast cast 1 as nint as object <> null; EqualText: bool := 'a' + 'b' = 'ab'); begin end procedure; begin P(); end; end program;");
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
        using var pe = new PEReader(new MemoryStream(result.Program!.AssemblyImage));
        var reader = pe.GetMetadataReader();
        var method = reader.MethodDefinitions.Select(reader.GetMethodDefinition).Single(method => reader.GetString(method.Name) == "P");
        foreach (var parameter in method.GetParameters().Select(reader.GetParameter))
        {
            var constant = reader.GetConstant(parameter.GetDefaultValue());
            Assert.True(reader.GetBlobReader(constant.Value).ReadBoolean());
        }
    }

    /// <summary>Keeps runtime range arguments out of compile-time constant evaluation.</summary>
    [Fact]
    public void Subrange_construction_preserves_runtime_unboxing_instead_of_panicking_during_constant_validation()
    {
        var result = TestCompilation.Compile("program Constants; type Small = int range 0..9; begin discard new Small(cast cast 'x' as object as int); end; end program;");
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
    }

    /// <summary>Requires constant patterns to avoid runtime unboxing.</summary>
    [Fact]
    public void Constant_patterns_diagnose_runtime_unboxing()
    {
        var result = TestCompilation.CompileAgainstFramework("program Constants; begin discard case 1 of when cast cast 1L as object as int: 1; else 0; end case; end; end program;");
        Assert.False(result.Success);
        Assert.Equal("FP3024", Assert.Single(result.Diagnostics).Code);
    }

    /// <summary>Rejects runtime unboxing in CLR custom-attribute constants.</summary>
    [Fact]
    public void Attribute_constants_reject_runtime_unboxing()
    {
        var result = TestCompilation.CompileAgainstFramework("program Constants; uses System as Sys; [Sys.ComponentModel.DefaultValueAttribute(cast cast 1L as object as int)] procedure P(); begin end procedure; begin end; end program;");
        Assert.False(result.Success);
        Assert.Equal("FP3035", Assert.Single(result.Diagnostics).Code);
    }

    /// <summary>Preserves legal boxed attribute values and numeric default metadata.</summary>
    [Fact]
    public void Safe_boxing_preserves_attribute_constant_payload_and_numeric_defaults()
    {
        var result = TestCompilation.CompileAgainstFramework("program Constants; uses System as Sys; [Sys.ComponentModel.DefaultValueAttribute(cast 1 as object)] procedure P(Value: long := cast 1 as long); begin end procedure; begin P(); end; end program;");
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
        using var pe = new PEReader(new MemoryStream(result.Program!.AssemblyImage));
        var reader = pe.GetMetadataReader();
        var method = reader.MethodDefinitions.Select(reader.GetMethodDefinition).Single(method => reader.GetString(method.Name) == "P");
        var attribute = AssemblyInspection.Attribute(reader, method.GetCustomAttributes(), "DefaultValueAttribute");
        Assert.Equal(new byte[] { 1, 0, 8, 1, 0, 0, 0, 0, 0 }, reader.GetBlobBytes(attribute.Value));
        var parameter = reader.GetParameter(Assert.Single(method.GetParameters()));
        Assert.Equal(ConstantTypeCode.Int64, reader.GetConstant(parameter.GetDefaultValue()).TypeCode);
    }
}
