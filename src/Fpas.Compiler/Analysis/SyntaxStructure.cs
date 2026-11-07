using Fpas.Compiler.Diagnostics;
using Fpas.Compiler.Syntax;
using Fpas.Compiler.Syntax.Parsing;

namespace Fpas.Compiler.Analysis;

public sealed record SyntaxDeclaration(string Name, string Kind, int? Owner, SourceSpan NameSpan, SourceSpan Span);

public sealed record SyntaxStructure(SourceDocument Document, IReadOnlyList<SyntaxDeclaration> Declarations,
    IReadOnlyList<SourceSpan> Folds, IReadOnlyList<SourceSpan> Selections);

public static partial class SyntaxAnalysis
{
    public static SyntaxStructure Structure(CompilationSource input, CancellationToken token = default)
    {
        var source = new SourceText(input.Path, input.Text, token);
        var document = new SourceDocument(input.Path, input.Text, token);
        var comments = new List<SourceComment>();
        IReadOnlyList<Token> tokens;
        try { tokens = new Lexer(source, comments).ReadTokens(); }
        catch (CompilationFailure) { return new(document, [], [], []); }
        var layout = new SyntaxLayout(tokens.Count);
        var parser = new Parser(source, tokens, layout, recover: true);
        try { _ = parser.ParseCompilationUnit(); }
        catch (CompilationFailure) { /* Keep declarations already parsed. */ }

        var declarations = parser.Declarations.Values.Where(item => item.End >= item.Name.Position + item.Name.Length)
            .Select(item => new SyntaxDeclaration(item.SourceName, item.Kind, item.Owner,
                document.Span(item.Name.Position, item.Name.Length), Span(item.Start, item.End))).ToArray();
        var bodies = layout.Bodies.Where(pair => pair.Start < pair.End)
            .Select(pair => Span(tokens[pair.Start].Position, tokens[pair.End].Position));
        var folds = declarations.Select(item => item.Span).Concat(bodies).Distinct()
            .Where(span => document.Span(span.Offset + span.Length, 0).Line > span.Line).ToArray();
        var selections = folds.Concat(declarations.Select(item => item.Span))
            .Concat(layout.Lists.Select(list => Span(tokens[list.Open].Position, tokens[list.Close].Position + tokens[list.Close].Length)))
            .Concat(tokens.Where(item => item.Length > 0).Select(item => document.Span(item.Position, item.Length)))
            .Concat(comments.Select(comment => Span(comment.Start, comment.End))).Distinct().ToArray();
        token.ThrowIfCancellationRequested();
        return new(document, Array.AsReadOnly(declarations), Array.AsReadOnly(folds), Array.AsReadOnly(selections));

        SourceSpan Span(int start, int end) => document.Span(start, Math.Max(0, end - start));
    }
}
