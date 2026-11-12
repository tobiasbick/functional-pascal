namespace Fpas.Compiler.Syntax;

/// <summary>Optional concrete layout facts collected by the parser, using token indices.</summary>
internal sealed class SyntaxLayout(int tokenCount)
{
    internal int[] Breaks { get; } = new int[tokenCount];
    internal int[] Indents { get; } = new int[tokenCount];
    internal int[] ClosingComments { get; } = new int[tokenCount];
    internal HashSet<int> Tight { get; } = [];
    internal HashSet<int> Unary { get; } = [];
    internal List<SyntaxList> Lists { get; } = [];
    internal List<SyntaxBinary> Binary { get; } = [];
    internal List<(int Start, int Clause)> Conditions { get; } = [];
    internal Dictionary<int, int> ConditionalEnds { get; } = [];
    internal HashSet<int> SingleElementBranchArrays { get; } = [];
    internal List<(int Start, int End)> Bodies { get; } = [];

    internal void Line(int index) => Breaks[index] = Math.Max(Breaks[index], 1);
    internal void Section(int index) => Breaks[index] = 2;
    internal void Indent(int start, int end) { Indents[start]++; Indents[end]--; }
    internal void Body(int start, int end)
    {
        IndentBody(start, end);
        Bodies.Add((start, end));
    }

    internal void ConditionalValue(int start, int end, bool singleElementArray)
    {
        IndentBody(start, end);
        if (singleElementArray) SingleElementBranchArrays.Add(start);
    }

    private void IndentBody(int start, int end)
    {
        Line(start);
        Line(end);
        Indent(start, end);
        ClosingComments[end]++;
    }
    internal ListBuilder List(int open, bool wrap = true, bool closeLine = true) => new(this, open, wrap, closeLine);

    internal sealed class ListBuilder(SyntaxLayout layout, int open, bool wrap, bool closeLine)
    {
        private readonly List<int> items = [];
        internal void Item(int index) => items.Add(index);
        internal void End(int close) => layout.Lists.Add(new(open, close, items.ToArray(), wrap, closeLine));
    }
}

internal sealed record SyntaxList(int Open, int Close, IReadOnlyList<int> Items, bool Wrap, bool CloseLine);
internal sealed record SyntaxBinary(int Start, int End, int Operator, int Right);
