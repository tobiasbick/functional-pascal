using Fpas.Compiler.Analysis;
using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Analysis.Support;

internal static class SemanticAnalysis
{
    internal static SemanticSnapshot Analyze(string source)
    {
        var result = FpasCompiler.Analyze(new([new("input.fpas", source)], "Input", References));
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
        return Assert.IsType<SemanticSnapshot>(result.Snapshot);
    }

    internal static string[] References => [.. TestEnvironment.References, typeof(Core.RecordAttribute).Assembly.Location];
}
