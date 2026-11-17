using Fpas.Compiler.Tests.Support;
using Fpas.Tests.Support;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;

namespace Fpas.Compiler.Tests.Routines;

/// <summary>Checks which CLR conversions can participate in source constants.</summary>
public sealed class ConstantConversionTests
{
    /// <summary>Rejects operations that must unwrap runtime storage in optional defaults.</summary>
    /// <param name="expression">The runtime conversion used as an optional default.</param>
    [Theory]
    [InlineData("cast<int>(cast<object>(1L))")]
    [InlineData("cast<int>(cast<int?>(null))")]
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
        var result = TestCompilation.CompileAgainstFramework("program Constants; procedure P(Value: string := cast<string>(cast<object>(1))); begin end procedure; begin end; end program;");
        Assert.False(result.Success);
        Assert.Equal("FP3028", Assert.Single(result.Diagnostics).Code);
    }

    /// <summary>Rejects reference equality whose result depends on runtime object identity.</summary>
    /// <param name="expression">The boxed reference comparison used as a default.</param>
    [Theory]
    [InlineData("cast<object>(cast<nint>(1)) = cast<object>(1L)")]
    [InlineData("cast<object>(cast<nuint>(1)) = cast<object>(1UL)")]
    [InlineData("cast<object>(1) = cast<object>(1)")]
    [InlineData("cast<object>('a' + 'b') = cast<object>('ab')")]
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
        var result = TestCompilation.CompileAgainstFramework("program Constants; procedure P(Present: bool := cast<object>(cast<nint>(1)) <> null; EqualText: bool := 'a' + 'b' = 'ab'); begin end procedure; begin P(); end; end program;");
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
        var result = TestCompilation.Compile("program Constants; type Small = int range 0..9; begin discard new Small(cast<int>(cast<object>('x'))); end; end program;");
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
    }

    /// <summary>Requires constant patterns to avoid runtime unboxing.</summary>
    [Fact]
    public void Constant_patterns_diagnose_runtime_unboxing()
    {
        var result = TestCompilation.CompileAgainstFramework("program Constants; begin discard case 1 of when cast<int>(cast<object>(1L)): 1; else 0; end case; end; end program;");
        Assert.False(result.Success);
        Assert.Equal("FP3024", Assert.Single(result.Diagnostics).Code);
    }

    /// <summary>Rejects runtime unboxing in CLR custom-attribute constants.</summary>
    [Fact]
    public void Attribute_constants_reject_runtime_unboxing()
    {
        var result = TestCompilation.CompileAgainstFramework("program Constants; uses System as Sys; [Sys.ComponentModel.DefaultValueAttribute(cast<int>(cast<object>(1L)))] procedure P(); begin end procedure; begin end; end program;");
        Assert.False(result.Success);
        Assert.Equal("FP3035", Assert.Single(result.Diagnostics).Code);
    }

    /// <summary>Preserves legal boxed attribute values and numeric default metadata.</summary>
    [Fact]
    public void Safe_boxing_preserves_attribute_constant_payload_and_numeric_defaults()
    {
        var result = TestCompilation.CompileAgainstFramework("program Constants; uses System as Sys; [Sys.ComponentModel.DefaultValueAttribute(cast<object>(1))] procedure P(Value: long := cast<long>(1)); begin end procedure; begin P(); end; end program;");
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
