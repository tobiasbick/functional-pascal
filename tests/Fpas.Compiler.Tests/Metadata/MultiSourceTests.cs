using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Metadata;

public sealed class MultiSourceTests
{
    [Fact]
    public void Libraries_emit_separate_containers_including_empty_units()
    {
        var result = Compile(CompilationTarget.Library,
            "unit Demo.First; public function Value(): int; begin return 1; end function; end unit;",
            "unit Demo.Empty; end unit;",
            "unit Demo.Last; public function Value(X: double): double; begin return X; end function; end unit;");
        Assert.True(result.Success, string.Join("; ", result.Diagnostics));
        Assert.Null(result.Program!.RuntimeConfiguration);
        using var pe = new PEReader(new MemoryStream(result.Program.AssemblyImage));
        Assert.Equal(0, pe.PEHeaders.CorHeader!.EntryPointTokenOrRelativeVirtualAddress);
        var reader = pe.GetMetadataReader();
        var types = reader.TypeDefinitions.Select(reader.GetTypeDefinition).ToDictionary(type => reader.GetString(type.Name));
        Assert.Equal(4, types.Count);
        Assert.Empty(types["<Module>"].GetMethods());
        Assert.Empty(types["Empty"].GetMethods());
        foreach (var name in new[] { "First", "Last" })
        {
            var type = types[name];
            Assert.Equal("Demo", reader.GetString(type.Namespace));
            var method = reader.GetMethodDefinition(Assert.Single(type.GetMethods()));
            Assert.Equal("Value", reader.GetString(method.Name));
            Assert.Single(type.GetCustomAttributes());
        }
        Assert.Equal(2, reader.MemberReferences.Count);
    }

    [Fact]
    public void Source_and_reference_order_and_filenames_do_not_change_the_PE()
    {
        CompilationSource[] sources =
        [
            new("entry.fpas", "program Entry; uses Lib as API; begin discard API.Value(); end; end program;"),
            new("lib.fpas", "unit Lib; public function Value(): int; begin return 42; end function; end unit;")
        ];
        var first = FpasCompiler.Compile(new(sources, "Stable", TestEnvironment.References));
        var second = FpasCompiler.Compile(new(sources.Reverse().Select(input => input with { Path = "renamed-" + input.Path }).ToArray(),
            "Stable", TestEnvironment.References.Reverse().ToArray()));
        Assert.True(first.Success, string.Join("; ", first.Diagnostics));
        Assert.True(second.Success, string.Join("; ", second.Diagnostics));
        Assert.Equal(first.Program!.AssemblyImage, second.Program!.AssemblyImage);
    }

    [Theory]
    [InlineData(CompilationTarget.Exe, "unit A; end unit;", "unit B; end unit;")]
    [InlineData(CompilationTarget.Exe, "program A; begin end; end program;", "program B; begin end; end program;")]
    [InlineData(CompilationTarget.Library, "program A; begin end; end program;", "unit B; end unit;")]
    public void Output_kind_enforces_the_number_of_programs(CompilationTarget target, string first, string second)
        => Assert.Equal("FP2004", Assert.Single(Compile(target, first, second).Diagnostics).Code);

    [Fact]
    public void Empty_compilations_are_diagnosed()
        => Assert.Equal("FP2004", Assert.Single(Compile(CompilationTarget.Library).Diagnostics).Code);

    [Theory]
    [InlineData("unit Same; end unit;", "unit Same; end unit;", "FP2005")]
    [InlineData("unit A; end unit;", "unit System.Console; end unit;", "FP4002")]
    [InlineData("unit A; function Hidden(): int; begin return 1; end function; end unit;",
        "unit B; uses A as API; public function Value(): int; begin return API.Hidden(); end function; end unit;", "FP3011")]
    [InlineData("unit A; end unit;",
        "unit B; uses A as API; public function Value(): int; begin return API.Missing(); end function; end unit;", "FP3001")]
    [InlineData("unit A; public function Value(X: int): int; begin return X; end function; end unit;",
        "unit B; uses A as API; public function Value(): int; begin return API.Value('bad'); end function; end unit;", "FP3003")]
    [InlineData("unit A; public function Value(): int; begin return 1; end function; end unit;",
        "unit B; public function Other(): int; begin return Value(); end function; end unit;", "FP3001")]
    public void Duplicate_names_visibility_and_unit_calls_are_checked(string first, string second, string code)
        => Assert.Equal(code, Assert.Single(Compile(CompilationTarget.Library, first, second).Diagnostics).Code);

    [Fact]
    public void A_program_is_not_an_importable_unit()
    {
        var result = Compile(CompilationTarget.Exe, "program Entry; begin end; end program;", "unit Helper; uses Entry as App; end unit;");
        Assert.Equal("FP3012", Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Reference_diagnostics_use_the_importing_source_location()
    {
        var result = FpasCompiler.Compile(new(
            [new("entry.fpas", "program Entry; begin end; end program;"),
             new("helper.fpas", "unit Helper;\nuses System.MissingType as Missing;\nend unit;")],
            "Location", TestEnvironment.References));
        var diagnostic = Assert.Single(result.Diagnostics);
        Assert.Equal("FP3001", diagnostic.Code);
        Assert.Equal(new SourceLocation("helper.fpas", 2, 6, 1), diagnostic.Location);
    }

    private static CompilationResult Compile(CompilationTarget target, params string[] sources)
        => FpasCompiler.Compile(new(sources.Select((text, index) => new CompilationSource($"source{index}.fpas", text)).ToArray(),
            "Multiple", TestEnvironment.References, target));
}
