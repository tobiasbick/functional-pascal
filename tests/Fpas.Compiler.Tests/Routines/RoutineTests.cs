using Fpas.Compiler.Tests.ControlFlow;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Routines;

public sealed class RoutineTests
{
    [Fact]
    public void A_unit_exports_typed_public_static_methods_without_an_entry_point()
    {
        var result = Compile("unit Demo.Api; public function Echo(Value: string): string; begin return Value; end function; end unit;");
        Assert.True(result.Success, string.Join("; ", result.Diagnostics));
        Assert.Null(result.Program!.RuntimeConfiguration);
        using var pe = new PEReader(new MemoryStream(result.Program.AssemblyImage));
        Assert.Equal(0, pe.PEHeaders.CorHeader!.EntryPointTokenOrRelativeVirtualAddress);
        var reader = pe.GetMetadataReader();
        var method = reader.GetMethodDefinition(Assert.Single(reader.MethodDefinitions));
        Assert.Equal("Echo", reader.GetString(method.Name));
        Assert.True(method.Attributes.HasFlag(MethodAttributes.Public | MethodAttributes.Static));
        Assert.Equal(new byte[] { 0, 1, 0x0e, 0x0e }, reader.GetBlobBytes(method.Signature));
        Assert.Equal("Value", reader.GetString(reader.GetParameter(Assert.Single(method.GetParameters())).Name));
    }

    [Theory]
    [InlineData("unit Api; public function Value(): int; begin end function; end unit;", "FP3007")]
    [InlineData("unit Api; public function Value(): int; begin return 'x'; end function; end unit;", "FP3002")]
    [InlineData("unit Api; public procedure Value(); begin return 1; end procedure; end unit;", "FP3002")]
    [InlineData("unit Api; public function Value(): int; begin return 1; return 2; end function; end unit;", "FP3008")]
    [InlineData("unit Api; public function Value(X: int; X: int): int; begin return X; end function; end unit;", "FP3004")]
    public void Invalid_routine_bodies_are_diagnosed(string source, string code)
        => Assert.Equal(code, Assert.Single(Compile(source).Diagnostics).Code);

    [Fact]
    public void Output_kind_must_match_the_source_container()
    {
        Assert.Equal("FP2004", Assert.Single(DiagnosticsTests.Compile("unit Api; end unit;").Diagnostics).Code);
        Assert.Equal("FP2004", Assert.Single(Compile("program App; begin end; end program;").Diagnostics).Code);
    }

    private static CompilationResult Compile(string source) => FpasCompiler.Compile(
        new([new("api.fpas", source)], "Api", TestEnvironment.References, CompilationTarget.Library));
}
