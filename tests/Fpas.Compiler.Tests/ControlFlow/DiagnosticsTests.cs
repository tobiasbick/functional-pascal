using Fpas.Compiler;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.ControlFlow;

public sealed class DiagnosticsTests
{
    private const string Header = "program Example; uses System.Console as Console, System.String as Text; begin ";

    [Theory]
    [InlineData("const Message := 'hello';", "FP2002")]
    [InlineData("const Number: int := 'hello';", "FP3002")]
    [InlineData("Console.WriteLine(Missing);", "FP3001")]
    [InlineData("console.WriteLine('hello');", "FP3001")]
    [InlineData("Console.Missing('hello');", "FP3003")]
    [InlineData("Text.Concat('a', 'b');", "FP3006")]
    [InlineData("const Message: string := Console.ReadLine();", "FP3013")]
    [InlineData("const A: string := B; const B: string := 'b';", "FP3001")]
    [InlineData("const A: string := 'a'; const A: string := 'b';", "FP3004")]
    [InlineData("const Console: string := 'a';", "FP3004")]
    [InlineData("const A: integer := 1;", "FP3005")]
    [InlineData("const A: int := 2147483648;", "FP3002")]
    [InlineData("const A: int := 1__0;", "FP1003")]
    [InlineData("const A: string := 'unfinished;", "FP1002")]
    [InlineData("const A: double := 1.5Q;", "FP1004")]
    [InlineData("var A := 1;", "FP2002")]
    public void Invalid_programs_return_diagnostics_without_output(string body, string code)
    {
        var result = Compile(Header + body + " end; end program;");
        Assert.False(result.Success);
        Assert.Null(result.Program);
        Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
    }

    [Theory]
    [InlineData("\n")]
    [InlineData("\r\n")]
    [InlineData("\r")]
    public void Missing_types_report_the_original_source_location(string newline)
    {
        var source = string.Join(newline, "// comment", "program Example;", "begin", "const Value := 'x';", "end;", "end program;");
        var diagnostic = Assert.Single(Compile(source).Diagnostics);
        Assert.Equal("FP2002", diagnostic.Code);
        Assert.Equal(new SourceLocation("example.fpas", 4, 13, 2), diagnostic.Location);
    }

    [Theory]
    [InlineData("Program Example; begin end; end program;")]
    [InlineData("program Example; begin end.")]
    [InlineData("program Example; begin end program;")]
    [InlineData("program Example; begin end;")]
    [InlineData("program Example; begin end; end program; extra")]
    public void Keywords_and_program_terminators_follow_the_new_syntax(string source)
        => Assert.Equal("FP2001", Assert.Single(Compile(source).Diagnostics).Code);

    [Fact]
    public void Duplicate_imports_are_rejected()
    {
        var result = Compile("program Example; uses System.Console as First, System.Console as Second; begin end; end program;");
        Assert.Equal("FP3004", Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Missing_framework_references_are_diagnosed()
    {
        var result = FpasCompiler.Compile(new([new("example.fpas", "program Example; begin end; end program;")], "Example", []));
        Assert.Equal("FP4001", Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Non_assembly_references_are_diagnosed()
    {
        using var directory = new TemporaryDirectory();
        var path = directory.File("invalid.dll");
        File.WriteAllText(path, "not an assembly");
        var result = FpasCompiler.Compile(new([new("example.fpas", "program Example; begin end; end program;")], "Example", [path]));
        Assert.Equal("FP4001", Assert.Single(result.Diagnostics).Code);
    }

    internal static CompilationResult Compile(string source) => FpasCompiler.Compile(
        new([new("example.fpas", source)], "Example", TestEnvironment.References));
}
