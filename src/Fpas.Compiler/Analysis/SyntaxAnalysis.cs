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
}
