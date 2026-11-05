using Fpas.Compiler.Syntax;
using System.Text;

namespace Fpas.Compiler.Formatting;

internal sealed record RenderedLine(int Number, int Width, int Indent);
internal sealed record RenderedSource(string Text, int[] Line, int[] Column, IReadOnlyList<RenderedLine> Lines);

internal sealed class LayoutWriter(SourceText source, IReadOnlyList<Token> tokens, SourceTrivia trivia,
    SyntaxLayout layout, IReadOnlyDictionary<int, int> continuations)
{
    private readonly StringBuilder output = new();
    private readonly int[] lines = new int[tokens.Count];
    private readonly int[] columns = new int[tokens.Count];
    private readonly List<RenderedLine> renderedLines = [];
    private readonly int[] tokenIndents = new int[tokens.Count];
    private int column;
    private int line;
    private int lineIndent;
    private int codeWidth;
    private int newlines;
    private int depth;

    internal RenderedSource Write()
    {
        for (var index = 0; index < tokens.Count; index++)
        {
            source.CheckCancellation();
            depth += layout.Indents[index];
            var indent = depth * 2;
            if (continuations.TryGetValue(index, out var anchor))
                indent = Math.Max(indent, columns[anchor] == tokenIndents[anchor] ? columns[anchor] : tokenIndents[anchor] + 2);
            tokenIndents[index] = depth * 2;
            Gap(index, indent);
            if (tokens[index].Kind == TokenKind.End) break;
            if (column == 0) { output.Append(' ', indent); column = indent; lineIndent = indent; }
            else if (SpaceBefore(index)) { output.Append(' '); column++; }
            lines[index] = line; columns[index] = column;
            Append(source.Text.Substring(tokens[index].Position, tokens[index].Length));
            codeWidth = column;
        }
        Break(1);
        return new(output.ToString(), lines, columns, renderedLines);
    }

    private void Gap(int index, int indent)
    {
        var comments = trivia.Comments[index];
        var requested = layout.Breaks[index];
        if (comments.Count == 0) { Break(requested); return; }
        var previous = index == 0 ? 0 : tokens[index - 1].Position + tokens[index - 1].Length;
        foreach (var comment in comments)
        {
            if (comment.Standalone)
            {
                Break(Math.Max(requested, trivia.Newlines(previous, comment.Start) > 1 ? 2 : 1));
                var commentIndent = indent + layout.ClosingComments[index] * 2;
                if (column == 0) { output.Append(' ', commentIndent); column = commentIndent; lineIndent = commentIndent; }
                requested = 0;
            }
            else if (column != 0) { output.Append(' '); column++; }
            Append(source.Text[comment.Start..comment.End].TrimEnd());
            Break(1);
            previous = comment.End;
        }
        // A blank before the next token keeps Markdown documentation detached.
        Break(Math.Max(requested, trivia.Newlines(previous, tokens[index].Position) > 1 ? 2 : 1));
    }

    private bool SpaceBefore(int index)
    {
        if (index == 0) return false;
        var token = tokens[index]; var previous = tokens[index - 1];
        var current = token.Kind is TokenKind.Keyword or TokenKind.Symbol ? token.Text : "";
        var before = previous.Kind is TokenKind.Keyword or TokenKind.Symbol ? previous.Text : "";
        if (layout.Tight.Contains(index) || layout.Tight.Contains(index - 1) && before == "<") return false;
        if (current is ";" or "," or ":" or "." or ".." or ")" or "]" or "?" || before is "." or ".." or "(" or "[") return false;
        if (current == "(") return previous.Kind != TokenKind.Identifier && before is not (")" or "]" or "function" or "procedure" or "cast" or "typeof" or "unchecked" or "new") && !(before == ">" && layout.Tight.Contains(index - 1));
        if (current == "[") return previous.Kind != TokenKind.Identifier && before is not (")" or "]" or "?") && !(before == ">" && layout.Tight.Contains(index - 1));
        if (layout.Unary.Contains(index - 1) && before is "+" or "-") return false;
        return true;
    }

    private void Append(string text)
    {
        output.Append(text); column += TextColumns.Width(text, column, source.CancellationToken); newlines = 0;
    }

    private void Break(int count)
    {
        if (count == 0 || output.Length == 0) return;
        if (newlines == 0) { renderedLines.Add(new(line, codeWidth, lineIndent)); line++; }
        while (newlines < count) { output.Append('\n'); newlines++; }
        column = 0;
        codeWidth = 0;
    }
}
