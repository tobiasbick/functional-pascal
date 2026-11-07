using Fpas.Compiler.Analysis;
using Fpas.Compiler.Tests.Support;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Types;

public sealed class GenericDelimiterTests
{
    [Theory]
    [InlineData("type Box<T>=record public Value:T;end record;")]
    [InlineData("type Box<T> = record public Value:T;end record;")]
    [InlineData("type Box<T>\n=record public Value:T;end record;")]
    [InlineData("type Box<T>// declaration\n=record public Value:T;end record;")]
    [InlineData("public type Pair<T,U>=record First:T;Second:U;end record;")]
    [InlineData("type Choice<T>=union Empty;Value(Item:T);end union;")]
    [InlineData("type IValue<T>=interface Value:T;end interface;")]
    [InlineData("type Box<T>=where T:class;record public Value:T;end record;")]
    public void Generic_declarations_accept_the_equals_delimiter_without_whitespace(string declaration)
    {
        var text = "unit Library;" + declaration + "end unit;";
        var compiled = TestCompilation.Compile(text, CompilationTarget.Library);
        Assert.True(compiled.Success, string.Join('\n', compiled.Diagnostics));
        var analyzed = FpasCompiler.Analyze(new([new("library.fpas", text)], "Library",
            [.. TestEnvironment.References, typeof(Core.RecordAttribute).Assembly.Location], CompilationTarget.Library));
        Assert.True(analyzed.Success, string.Join('\n', analyzed.Diagnostics));
    }
}
