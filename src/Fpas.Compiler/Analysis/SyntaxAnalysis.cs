using Fpas.Compiler.Syntax;
using Fpas.Compiler.Syntax.Parsing;

namespace Fpas.Compiler.Analysis;

public static partial class SyntaxAnalysis
{
    public static IReadOnlyList<CompilerDiagnostic> Check(CompilationSource input, CancellationToken token = default)
    {
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
