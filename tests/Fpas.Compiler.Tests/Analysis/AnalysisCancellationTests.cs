using Fpas.Compiler.Analysis;
using Fpas.Tests.Support;
using System.Collections;

namespace Fpas.Compiler.Tests.Analysis;

public sealed class AnalysisCancellationTests
{
    [Fact]
    public void Cancellation_after_binding_does_not_publish_success_or_poison_a_later_analysis()
    {
        const string text = """
            unit Docs;
            // Reads a value.
            // ## Parameters
            // - `Wrong`: Invalid parameter.
            public function Read(Value: int): int;
            begin return Value; end function;
            end unit;
            """;
        var sources = new[] { new CompilationSource("docs.fpas", text) };
        using var cancellation = new CancellationTokenSource();
        // User-supplied warning options cancel once documentation diagnostics are processed.
        var options = new DiagnosticOptions(NoWarn: new CancelOnRead(cancellation));
        var request = new AnalysisRequest(sources, "Docs", TestEnvironment.References, CompilationTarget.Library, true, options);
        var failure = Assert.ThrowsAny<OperationCanceledException>(() => FpasCompiler.Analyze(request, cancellation.Token));
        Assert.Equal(cancellation.Token, failure.CancellationToken);
        var next = FpasCompiler.Analyze(request with { DiagnosticOptions = null });
        Assert.True(next.Success); Assert.Equal("FP6001", Assert.Single(next.Diagnostics).Code);
    }

    private sealed class CancelOnRead(CancellationTokenSource cancellation) : IReadOnlyList<string>
    {
        public int Count => 1;
        public string this[int index] => index == 0 ? "FP0000" : throw new ArgumentOutOfRangeException(nameof(index));
        public IEnumerator<string> GetEnumerator() { cancellation.Cancel(); yield return "FP0000"; }
        IEnumerator IEnumerable.GetEnumerator() => GetEnumerator();
    }
}
