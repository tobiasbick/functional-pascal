// [Documentation](../../../docs/architecture/compiler.md#semantic-snapshots)
using Fpas.Compiler.Diagnostics;
using Fpas.Compiler.Syntax;
using Fpas.Compiler.Syntax.Parsing;

namespace Fpas.Compiler.Analysis;

/// <summary>A declaration recognized by syntax alone.</summary>
/// <param name="Name">The declared source name.</param>
/// <param name="Kind">The declaration kind, such as container, routine, constant, or alias.</param>
/// <param name="Owner">The name offset of the enclosing declaration, when nested.</param>
/// <param name="NameSpan">The span of the declared name.</param>
/// <param name="Span">The span of the whole declaration.</param>
public sealed record SyntaxDeclaration(string Name, string Kind, int? Owner, SourceSpan NameSpan, SourceSpan Span);

/// <summary>The editor structure of one source.</summary>
/// <param name="Document">The analyzed source.</param>
/// <param name="Declarations">The recognized declarations.</param>
/// <param name="Folds">Multi-line ranges that can be folded.</param>
/// <param name="Selections">Syntactic ranges for expanding a selection.</param>
public sealed record SyntaxStructure(SourceDocument Document, IReadOnlyList<SyntaxDeclaration> Declarations,
    IReadOnlyList<SourceSpan> Folds, IReadOnlyList<SourceSpan> Selections)
{
    /// <summary>Gets lexer and recovered parser diagnostics for the source.</summary>
    public IReadOnlyList<CompilerDiagnostic> Diagnostics { get; init; } = [];
}

public static partial class SyntaxAnalysis
{
    /// <summary>Collects declarations, folds, and selection ranges, recovering from syntax errors where possible.</summary>
    /// <param name="input">The source to analyze.</param>
    /// <param name="token">Cancels the analysis with an <see cref="OperationCanceledException"/>.</param>
    /// <returns>The structure that could be recognized.</returns>
    public static SyntaxStructure Structure(CompilationSource input, CancellationToken token = default)
    {
        ArgumentNullException.ThrowIfNull(input);
        var source = new SourceText(input.Path, input.Text, token);
        var document = new SourceDocument(input.Path, input.Text, token);
        var comments = new List<SourceComment>();
        var tokens = new Lexer(source, comments).ReadPrefix(out var lexicalError);
        var layout = new SyntaxLayout(tokens.Count);
        var parser = new Parser(source, tokens, layout, recover: true);
        IReadOnlyList<CompilerDiagnostic> diagnostics;
        try { _ = parser.ParseCompilationUnit(); diagnostics = source.Diagnostics.Snapshot; }
        catch (CompilationFailure failure)
        {
            diagnostics = source.Diagnostics.Snapshot.Concat(failure.Diagnostics).Distinct().Take(DiagnosticBag.Limit).ToArray();
        }
        if (lexicalError is not null)
            diagnostics = diagnostics.Where(item => item.Location?.Offset < tokens[^1].Position)
                .Take(DiagnosticBag.Limit - 1).Append(lexicalError).ToArray();

        var declarations = parser.Declarations.Values.Where(item => item.End >= item.Name.Position + item.Name.Length)
            .Select(item => new SyntaxDeclaration(item.SourceName, item.Kind, item.Owner,
                document.Span(item.Name.Position, item.Name.Length), Span(item.Start, item.End))).ToArray();
        var bodies = layout.Bodies.Where(pair => pair.Start < pair.End)
            .Select(pair => Span(tokens[pair.Start].Position, tokens[pair.End].Position));
        var folds = declarations.Select(item => item.Span).Concat(bodies).Distinct()
            .Where(span => document.Span(span.Offset + span.Length, 0).Line > span.Line).ToArray();
        var selections = folds.Concat(declarations.Select(item => item.Span))
            .Concat(layout.Imports.Select(import => TokenSpan(import.Keyword, import.Semicolon)))
            .Concat(layout.Imports.Count > 1 ? [TokenSpan(layout.Imports[0].Keyword, layout.Imports[^1].Semicolon)] : [])
            .Concat(layout.Lists.Select(list => TokenSpan(list.Open, list.Close)))
            .Concat(tokens.Where(item => item.Length > 0).Select(item => document.Span(item.Position, item.Length)))
            .Concat(comments.Select(comment => Span(comment.Start, comment.End))).Distinct().ToArray();
        token.ThrowIfCancellationRequested();
        return new(document, Array.AsReadOnly(declarations), Array.AsReadOnly(folds), Array.AsReadOnly(selections)) { Diagnostics = diagnostics };

        SourceSpan TokenSpan(int first, int last) => Span(tokens[first].Position, tokens[last].Position + tokens[last].Length);
        SourceSpan Span(int start, int end) => document.Span(start, Math.Max(0, end - start));
    }
}
