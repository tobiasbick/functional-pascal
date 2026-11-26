// [Documentation](../../../docs/pascal/language/types/generics.md)
using Fpas.Compiler.Analysis;
using Fpas.Compiler.Tests.Support;

namespace Fpas.Compiler.Tests.Types;

public sealed class GenericDelimiterTests
{
    [Theory]
    [InlineData("unit Library; type Box<T> = record Value: T; end record; end unit;")]
    [InlineData("unit Library; const A: Box<int> := null; end unit;")]
    [InlineData("program Demo; begin discard Identity<int>(42); end; end program;")]
    [InlineData("unit Library; type Box of (T) = record Value: T; end record; end unit;")]
    [InlineData("unit Library; const A: Box of (int) := null; end unit;")]
    [InlineData("program Demo; begin discard (Box of (int, string) of int).Create(42); end; end program;")]
    public void Generic_syntax_has_one_spelling_per_arity(string source)
        => Assert.NotEmpty(SyntaxAnalysis.Check(new("invalid.fpas", source)));

    [Theory]
    [InlineData("type Box of T=record public Value:T;end record;")]
    [InlineData("type Box of T = record public Value:T;end record;")]
    [InlineData("type Box of T\n=record public Value:T;end record;")]
    [InlineData("type Box of T// declaration\n=record public Value:T;end record;")]
    [InlineData("public type Pair of (T,U)=record First:T;Second:U;end record;")]
    [InlineData("type Choice of T=variant Empty;Value(Item:T);end variant;")]
    [InlineData("type IValue of T=protocol Value:T;end protocol;")]
    [InlineData("type Box of T=where T:class;record public Value:T;end record;")]
    public void Generic_declarations_accept_the_equals_delimiter_without_whitespace(string declaration)
    {
        var text = "unit Library;" + declaration + "end unit;";
        var compiled = TestCompilation.Compile(text, CompilationTarget.Library);
        Assert.True(compiled.Success, string.Join('\n', compiled.Diagnostics));
        var analyzed = FpasCompiler.Analyze(new([new("library.fpas", text)], "Library",
            TestCompilation.References, CompilationTarget.Library));
        Assert.True(analyzed.Success, string.Join('\n', analyzed.Diagnostics));
    }
}
