using Fpas.Compiler.Tests.Support;
using Fpas.Compiler.Analysis;

namespace Fpas.Compiler.Tests.Analysis.Support;

internal static class SemanticAnalysis
{
    internal static SemanticSnapshot Analyze(string source)
    {
        var result = FpasCompiler.Analyze(new([new("input.fpas", source)], "Input", TestCompilation.References));
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
        return Assert.IsType<SemanticSnapshot>(result.Snapshot);
    }

}
