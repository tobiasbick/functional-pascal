using Fpas.Compiler.Syntax;
using Fpas.Compiler.Syntax.Parsing;

namespace Fpas.Compiler.Formatting;

/// <summary>The outcome of formatting one source.</summary>
/// <param name="Text">The formatted text, or <see langword="null"/> when the source could not be formatted.</param>
/// <param name="Diagnostics">The syntax or preservation errors that prevented formatting.</param>
public sealed record FormatResult(string? Text, IReadOnlyList<CompilerDiagnostic> Diagnostics)
{
    /// <summary>Whether formatted text was produced.</summary>
    public bool Success => Text is not null;
}

/// <summary>Formats one complete source without references, binding, or code generation.</summary>
public static class FpasFormatter
{
    /// <summary>Formats a source and verifies that tokens, comments, and documentation are preserved.</summary>
    /// <param name="input">The source to format.</param>
    /// <param name="cancellationToken">Cancels formatting with an <see cref="OperationCanceledException"/>.</param>
    /// <returns>The formatted text, or the diagnostics that prevented formatting.</returns>
    public static FormatResult Format(CompilationSource input, CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(input);
        ArgumentNullException.ThrowIfNull(input.Text);
        var source = new SourceText(input.Path, input.Text, cancellationToken);
        try
        {
            var comments = new List<SourceComment>();
            var tokens = new Lexer(source, comments).ReadTokens();
            var layout = new SyntaxLayout(tokens.Count);
            new Parser(source, tokens, layout).ParseCompilationUnit();
            source.Diagnostics.ThrowIfErrors();
            var output = new SourceFormatter(source, tokens, comments, layout).Format();
            Validate(source, tokens, comments, output, layout.CompilationEnd);
            return new(output, []);
        }
        catch (CompilationFailure failure) { return new(null, failure.Diagnostics); }
    }

    private static void Validate(SourceText original, IReadOnlyList<Token> before, IReadOnlyList<SourceComment> comments,
        string output, int compilationEnd)
    {
        var formatted = new SourceText(original.Path, output, original.CancellationToken);
        var afterComments = new List<SourceComment>();
        IReadOnlyList<Token> after;
        try
        {
            after = new Lexer(formatted, afterComments).ReadTokens();
            new Parser(formatted, after).ParseCompilationUnit();
            formatted.Diagnostics.ThrowIfErrors();
        }
        catch (CompilationFailure) { throw PreservationFailure(original.Path); }
        // The compilation terminator cannot own declaration documentation.
        if (before.Count == after.Count && Enumerable.Range(0, before.Count)
            .All(index => SameToken(before[index], after[index], compareDocumentation: index != compilationEnd)) &&
            comments.Select(comment => original.Text[comment.Start..comment.End].TrimEnd())
            .SequenceEqual(afterComments.Select(comment => output[comment.Start..comment.End].TrimEnd()))) return;
        throw PreservationFailure(original.Path);
    }

    private static CompilationFailure PreservationFailure(string path)
        => new(new("FP7001", "Formatting could not preserve valid syntax, tokens, comments and documentation attachment.", null)
        { InputPath = path, Help = "The source was not modified. Please report this formatter error." });

    private static bool SameToken(Token before, Token after, bool compareDocumentation)
        => before.Kind == after.Kind && before.Text == after.Text &&
           (!compareDocumentation || (before.Documentation?.Lines.Select(line => line.Text.TrimEnd()) ?? [])
               .SequenceEqual(after.Documentation?.Lines.Select(line => line.Text.TrimEnd()) ?? []));
}
