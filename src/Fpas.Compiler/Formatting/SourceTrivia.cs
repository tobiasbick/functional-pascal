using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Formatting;

internal sealed class SourceTrivia
{
    internal IReadOnlyList<SourceComment>[] Comments { get; }
    private readonly SourceText source;
    private readonly IReadOnlyList<Token> tokens;

    internal SourceTrivia(SourceText source, IReadOnlyList<Token> tokens, IReadOnlyList<SourceComment> comments)
    {
        this.source = source; this.tokens = tokens;
        Comments = new IReadOnlyList<SourceComment>[tokens.Count];
        var commentIndex = 0;
        for (var index = 0; index < tokens.Count; index++)
        {
            source.CheckCancellation();
            var start = commentIndex;
            while (commentIndex < comments.Count && comments[commentIndex].Start < tokens[index].Position) commentIndex++;
            Comments[index] = comments.Skip(start).Take(commentIndex - start).ToArray();
        }
    }

    internal bool HasBreak(int start, int end) => Newlines(start, end) != 0;
    internal bool BlankBefore(int index)
    {
        var start = index == 0 ? 0 : tokens[index - 1].Position + tokens[index - 1].Length;
        foreach (var comment in Comments[index])
        {
            if (comment.Standalone) return Newlines(start, comment.Start) > 1;
            start = comment.End;
        }
        return Newlines(start, tokens[index].Position) > 1;
    }
    internal int Newlines(int start, int end)
    {
        var count = 0;
        for (var index = start; index < end; index++)
        {
            if ((index & 1023) == 0) source.CheckCancellation();
            if (source.Text[index] == '\r') { count++; if (index + 1 < end && source.Text[index + 1] == '\n') index++; }
            else if (source.Text[index] == '\n') count++;
        }
        return count;
    }
}
