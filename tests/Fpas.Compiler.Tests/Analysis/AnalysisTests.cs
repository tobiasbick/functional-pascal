using Fpas.Compiler.Analysis;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Analysis;

public sealed class AnalysisTests
{
    private static string[] References => [.. TestEnvironment.References, typeof(Core.RecordAttribute).Assembly.Location];

    [Theory]
    [InlineData("program Empty; begin end; end program;", CompilationTarget.Exe)]
    [InlineData("unit Library; public function Read(): int; begin return 42; end function; end unit;", CompilationTarget.Library)]
    [InlineData("program Records; type Point = record public X: int; end record; begin const P: Point := new Point(42); end; end program;", CompilationTarget.Exe)]
    [InlineData("async program Async; uses System.Threading.Tasks as T; async function F(): T.Task<int>; begin return await T.Task.FromResult<int>(42); end function; begin discard await F(); end; end program;", CompilationTarget.Exe)]
    public void Successful_analysis_has_independent_completion_and_does_not_publish_files(string source, CompilationTarget target)
    {
        using var directory = new TemporaryDirectory();
        var result = FpasCompiler.Analyze(new([new(directory.File("Input.fpas"), source)], "Analyzed", References, target, ValidateDocumentation: true));
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
        Assert.True(result.IsComplete); Assert.Empty(result.Diagnostics);
        Assert.Empty(Directory.GetFileSystemEntries(directory.Path));
    }

    [Theory]
    [InlineData("const Value := 1;", "FP2002")]
    [InlineData("const Value: int := 'wrong';", "FP3002")]
    [InlineData("discard Missing;", "FP3001")]
    [InlineData("return; discard 1;", "FP3008")]
    [InlineData("const Value: string := null;", "FP3013")]
    public void Analysis_and_compile_preserve_expected_semantic_failures(string body, string code)
    {
        var source = "program Invalid; begin " + body + " end; end program;";
        var request = new CompilationRequest([new("invalid.fpas", source)], "Invalid", References);
        var analysis = FpasCompiler.Analyze(new(request.Sources, request.AssemblyName, request.References));
        var compiled = FpasCompiler.Compile(request);
        Assert.False(analysis.Success); Assert.False(analysis.IsComplete);
        Assert.Equal(code, Assert.Single(analysis.Diagnostics).Code);
        Assert.Equal(compiled.Diagnostics, analysis.Diagnostics);
    }

    [Theory]
    [InlineData("\n")]
    [InlineData("\r\n")]
    [InlineData("\r")]
    public void Analysis_retains_original_error_coordinates(string newline)
    {
        var text = string.Join(newline, "// comment", "program Invalid;", "begin", "const Value := 'x';", "end;", "end program;");
        var result = FpasCompiler.Analyze(new([new("input.fpas", text)], "Input", References));
        var diagnostic = Assert.Single(result.Diagnostics);
        Assert.Equal("FP2002", diagnostic.Code);
        Assert.Equal(new SourceLocation("input.fpas", 4, 13, 2, 4, 15, text.IndexOf(":=", StringComparison.Ordinal)), diagnostic.Location);
    }

    [Fact]
    public void Analysis_runs_managed_reference_liveness_checks_from_async_lowering()
    {
        const string source = """
            program Invalid;
            uses System.Threading.Tasks as T;
            procedure Touch(ref Value: int; Other: int); begin Value := Other; end procedure;
            async function Work(): T.Task;
            begin
              var Value: int := 0;
              Touch(ref Value, await T.Task.FromResult<int>(42));
            end function;
            begin end;
            end program;
            """;
        var analyzed = FpasCompiler.Analyze(new([new("async.fpas", source)], "Invalid", References));
        var diagnostic = Assert.Single(analyzed.Diagnostics);
        Assert.False(analyzed.Success); Assert.Equal("FP3030", diagnostic.Code);
        Assert.Contains("cannot remain live across await", diagnostic.Message);
        Assert.Equal(7, diagnostic.Location!.Line);
        Assert.Equal(FpasCompiler.Compile(new([new("async.fpas", source)], "Invalid", References)).Diagnostics, analyzed.Diagnostics);
    }

    [Theory]
    [InlineData("procedure Work(out Value: int); begin end procedure;", "FP3028")]
    [InlineData("function Work(): int; requires Work() > 0; begin return 1; end function;", "FP3027")]
    public void Parameter_and_contract_validation_is_not_skipped(string declaration, string code)
    {
        var analyzed = FpasCompiler.Analyze(new([new("invalid.fpas", "program Invalid; " + declaration + " begin end; end program;")], "Invalid", References));
        Assert.False(analyzed.Success); Assert.Equal(code, Assert.Single(analyzed.Diagnostics).Code);
    }

    [Theory]
    [InlineData("program Invalid; begin end; end program;", CompilationTarget.Library)]
    [InlineData("unit Invalid; end unit;", CompilationTarget.Exe)]
    public void Target_rules_are_identical_to_compilation(string source, CompilationTarget target)
    {
        var result = FpasCompiler.Analyze(new([new("input.fpas", source)], "Input", References, target));
        Assert.False(result.Success); Assert.Equal("FP2004", Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Missing_and_malformed_references_return_FP4001()
    {
        using var directory = new TemporaryDirectory();
        var path = directory.File("bad.dll"); File.WriteAllText(path, "not metadata");
        foreach (var references in new[] { Array.Empty<string>(), new[] { path }, new[] { directory.File("missing.dll") } })
        {
            var result = FpasCompiler.Analyze(new([new("input.fpas", "program Input; begin end; end program;")], "Input", references));
            Assert.False(result.Success); Assert.Equal("FP4001", Assert.Single(result.Diagnostics).Code);
        }
    }

    [Theory]
    [InlineData("public type Point = record X: int; end record;")]
    [InlineData("public type Choice = variant Empty; Value(Item: int); end variant;")]
    [InlineData("public type Id = distinct long;")]
    [InlineData("public type Number = long;")]
    public void Missing_support_metadata_is_diagnosed_before_emission(string declaration)
    {
        var sources = new[] { new CompilationSource("library.fpas", "unit Library; " + declaration + " end unit;") };
        var analyzed = FpasCompiler.Analyze(new(sources, "Library", TestEnvironment.References, CompilationTarget.Library));
        Assert.False(analyzed.Success); Assert.Equal("FP4001", Assert.Single(analyzed.Diagnostics).Code);
        Assert.Equal(FpasCompiler.Compile(new(sources, "Library", TestEnvironment.References, CompilationTarget.Library)).Diagnostics, analyzed.Diagnostics);
    }

    [Fact]
    public void Precancelled_analysis_does_not_start_reading_references()
    {
        using var cancellation = new CancellationTokenSource(); cancellation.Cancel();
        Assert.ThrowsAny<OperationCanceledException>(() => FpasCompiler.Analyze(
            new([new("input.fpas", "program Input; begin end; end program;")], "Input", ["missing.dll"]), cancellation.Token));
    }

    [Fact]
    public void Missing_generated_domain_helper_metadata_is_diagnosed_without_emission()
    {
        var sources = new[] { new CompilationSource("domain.fpas", "unit Domains; public type Id = distinct long; end unit;") };
        string[] references = [Path.Combine(TestEnvironment.ReferenceDirectory, "System.Runtime.dll"), typeof(Core.RecordAttribute).Assembly.Location];
        var analyzed = FpasCompiler.Analyze(new(sources, "Domains", references, CompilationTarget.Library));
        Assert.False(analyzed.Success);
        var diagnostic = Assert.Single(analyzed.Diagnostics);
        Assert.Equal("FP4001", diagnostic.Code); Assert.Contains("EqualityComparer", diagnostic.Message);
        Assert.Equal(FpasCompiler.Compile(new(sources, "Domains", references, CompilationTarget.Library)).Diagnostics, analyzed.Diagnostics);
    }
}
