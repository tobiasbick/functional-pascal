// [Documentation](../../../docs/architecture/compiler.md#compiler-api-and-output)
using Fpas.Compiler.Syntax;
using Fpas.Compiler.Syntax.Parsing;

namespace Fpas.Compiler.Analysis;

/// <summary>Syntax-only queries over one source, without references or binding.</summary>
public static partial class SyntaxAnalysis
{
    /// <summary>Lexes and parses one source.</summary>
    /// <param name="input">The source to check.</param>
    /// <param name="token">Cancels the check with an <see cref="OperationCanceledException"/>.</param>
    /// <returns>The syntax diagnostics, or an empty list for valid syntax.</returns>
    public static IReadOnlyList<CompilerDiagnostic> Check(CompilationSource input, CancellationToken token = default)
    {
        ArgumentNullException.ThrowIfNull(input);
        token.ThrowIfCancellationRequested();
        var source = new SourceText(input.Path, input.Text, token);
        try
        {
            new Parser(source, new Lexer(source).ReadTokens()).ParseCompilationUnit();
            source.Diagnostics.ThrowIfErrors(); return [];
        }
        catch (CompilationFailure failure) { token.ThrowIfCancellationRequested(); return failure.Diagnostics; }
    }

    /// <summary>Finds the first control-flow construct on each path that exceeds the allowed nesting depth.</summary>
    /// <param name="document">The source to analyze.</param>
    /// <param name="limit">The maximum allowed control-flow depth within a routine.</param>
    /// <param name="token">Cancels the analysis with an <see cref="OperationCanceledException"/>.</param>
    /// <returns>The keyword spans that cross the limit, or an empty list for invalid syntax.</returns>
    public static IReadOnlyList<SourceSpan> FindDeepNesting(SourceDocument document, int limit, CancellationToken token = default)
    {
        ArgumentNullException.ThrowIfNull(document);
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(limit);
        token.ThrowIfCancellationRequested();
        var source = new SourceText(document.Path, document.Text, token);
        var nesting = new ControlNesting(limit);
        try
        {
            new Parser(source, new Lexer(source).ReadTokens(), controlNesting: nesting).ParseCompilationUnit();
            source.Diagnostics.ThrowIfErrors();
            token.ThrowIfCancellationRequested();
            return nesting.Exceeded.Select(keyword => document.Span(keyword.Position, keyword.Length)).ToArray();
        }
        catch (CompilationFailure) { token.ThrowIfCancellationRequested(); return []; }
    }
}
