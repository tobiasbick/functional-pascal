using Fpas.Compiler.Analysis;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Analysis;

public sealed class AnalysisDocumentationTests
{
    private const string Source = """
        unit Docs;
        // Read a value.
        // ## Parameters
        // - `Wrong`: Not a parameter.
        public function Read(Value: int): int;
        begin return Value; end function;
        end unit;
        """;

    [Theory]
    [InlineData(false, false, false, true, 0)]
    [InlineData(false, true, false, true, 0)]
    [InlineData(true, false, false, true, 1)]
    [InlineData(true, true, false, false, 1)]
    [InlineData(true, true, true, true, 0)]
    public void Analysis_documentation_policy_matches_compile_without_serializing_XML(bool validate, bool promote, bool suppress, bool success, int count)
    {
        var sources = new[] { new CompilationSource("docs.fpas", Source) };
        var policy = new DiagnosticOptions(TreatWarningsAsErrors: promote, NoWarn: suppress ? ["FP6001"] : []);
        var analyzed = FpasCompiler.Analyze(new(sources, "Docs", TestEnvironment.References, CompilationTarget.Library, validate, policy));
        var compiled = FpasCompiler.Compile(new(sources, "Docs", TestEnvironment.References, CompilationTarget.Library,
            EmitDocumentation: validate, DiagnosticOptions: policy));
        Assert.Equal(success, analyzed.Success); Assert.Equal(success, analyzed.IsComplete);
        Assert.Equal(count, analyzed.Diagnostics.Count); Assert.Equal(compiled.Diagnostics, analyzed.Diagnostics);
        foreach (var diagnostic in analyzed.Diagnostics)
        {
            Assert.Equal("FP6001", diagnostic.Code);
            Assert.Equal(promote ? DiagnosticSeverity.Error : DiagnosticSeverity.Warning, diagnostic.Severity);
            Assert.Equal("docs.fpas", diagnostic.Location.Path); Assert.Equal(4, diagnostic.Location.Line);
        }
    }
}
