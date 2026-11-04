using Fpas.Compiler;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Documentation;

public sealed class SemanticDocumentationTests
{
    private static IReadOnlyList<MarkdownExample> Examples() => MarkdownExamples.Read(Path.Combine(AppContext.BaseDirectory, "Fixtures", "specification"));
    public static IEnumerable<object[]> CompleteExamples() => Examples().Select(example => new object[] { example.Document, example.Fence });

    [Theory]
    [MemberData(nameof(CompleteExamples))]
    public void Documented_language_constructs_have_detached_consistent_semantic_indexes(string document, int fence)
    {
        var examples = Examples(); var example = examples.Single(example => example.Document == document && example.Fence == fence);
        var sources = MarkdownExamples.Sources(example, examples).Select(source => new CompilationSource(source.Name + ".fpas", source.Source)).ToArray();
        var core = Path.Combine(Path.GetDirectoryName(TestEnvironment.Metadata("FpasCliPath"))!, "Fpas.Core.dll");
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        var result = FpasCompiler.Analyze(new(sources, example.Name, [.. TestEnvironment.References, core],
            example.Library ? CompilationTarget.Library : CompilationTarget.Exe, ValidateDocumentation: true), deadline.Token);
        Assert.True(result.Success, document + ": " + string.Join(';', result.Diagnostics));
        var snapshot = result.Snapshot!;
        Assert.Equal(sources.Length, snapshot.Documents.Count);
        var symbolIds = snapshot.Symbols.Select(symbol => symbol.Id).ToHashSet(StringComparer.Ordinal);
        var typeIds = snapshot.Types.Select(type => type.Id).ToHashSet(StringComparer.Ordinal);
        Assert.Equal(snapshot.Symbols.Count, symbolIds.Count); Assert.Equal(snapshot.Types.Count, typeIds.Count);
        foreach (var occurrence in snapshot.Occurrences)
        {
            Assert.Contains(occurrence.Symbol, symbolIds);
            var text = snapshot.Documents.Single(source => source.Path == occurrence.Span.Path).Text;
            Assert.InRange(occurrence.Span.Offset, 0, text.Length - 1);
            Assert.InRange(occurrence.Span.Length, 1, text.Length - occurrence.Span.Offset);
            Assert.DoesNotContain('\n', text.Substring(occurrence.Span.Offset, occurrence.Span.Length));
        }
        foreach (var expression in snapshot.Expressions)
        {
            Assert.Contains(expression.Type, typeIds);
            if (expression.DeclaredType is not null) Assert.Contains(expression.DeclaredType, typeIds);
        }
        foreach (var type in snapshot.Types)
        {
            if (type.Element is not null) Assert.Contains(type.Element, typeIds);
            foreach (var argument in type.Arguments) Assert.Contains(argument, typeIds);
            if (type.Callable is { } callable)
            {
                Assert.Contains(callable.Result, typeIds);
                foreach (var parameter in callable.Parameters) Assert.Contains(parameter.Type, typeIds);
            }
        }
    }
}
