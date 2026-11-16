using Fpas.Compiler.Tests.Support;
using Fpas.Compiler.Analysis;

namespace Fpas.Compiler.Tests.Analysis;

public sealed class EditorLexicalRecoveryTests
{
    // A string literal that is still being typed makes the whole source untokenizable.
    private const string Source = "program Sample;\nbegin\n  Show('partial\nend;\nend program;";

    [Fact]
    public void Editor_queries_return_empty_results_while_the_source_cannot_be_tokenized()
    {
        var request = new AnalysisRequest([new("sample.fpas", Source)], "Sample", TestCompilation.References);
        var offset = Source.IndexOf("partial", StringComparison.Ordinal);

        var completion = EditorAnalysis.Complete(request, "sample.fpas", offset);
        Assert.Equal("none", completion.Context);
        Assert.Empty(completion.Items);

        var signatures = EditorAnalysis.Signatures(request, "sample.fpas", offset);
        Assert.Empty(signatures.Items);

        Assert.Null(EditorImports.Suggest(request, "sample.fpas", Source.IndexOf("Show", StringComparison.Ordinal)));
    }
}
