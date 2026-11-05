using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Formatting;

internal sealed class SourceFormatter
{
    private const int TargetWidth = 100;
    private readonly SourceText source;
    private readonly IReadOnlyList<Token> tokens;
    private readonly SyntaxLayout layout;
    private readonly SourceTrivia trivia;
    private readonly Dictionary<int, int> continuations = [];
    private readonly HashSet<SyntaxList> expanded = [];
    private readonly HashSet<SyntaxConditional> conditionals = [];

    internal SourceFormatter(SourceText source, IReadOnlyList<Token> tokens, IReadOnlyList<SourceComment> comments, SyntaxLayout layout)
    {
        this.source = source; this.tokens = tokens; this.layout = layout;
        trivia = new(source, tokens, comments);
    }

    internal string Format()
    {
        PreserveSections();
        foreach (var binary in layout.Binary)
        {
            var op = binary.Operator;
            if (trivia.HasBreak(End(op - 1), tokens[op].Position) || trivia.HasBreak(End(binary.Right - 1), tokens[binary.Right].Position))
                BreakBinary(binary);
        }
        foreach (var conditional in layout.Conditionals.OrderBy(value => value.End - value.Start))
            if (trivia.HasBreak(tokens[conditional.Start].Position, End(conditional.End - 1))) Expand(conditional);
        foreach (var list in layout.Lists.OrderBy(value => value.Close - value.Open))
            if (trivia.HasBreak(End(list.Open), tokens[list.Close].Position)) Expand(list, allItems: false);

        // Layout only gains breaks. Each pass either makes progress or finishes.
        while (true)
        {
            source.CheckCancellation();
            var changed = ExpandContainers();
            foreach (var (start, clause) in layout.Conditions)
                if (Enumerable.Range(start + 1, clause - start - 1).Any(index => layout.Breaks[index] > 0) && layout.Breaks[clause] == 0)
                { layout.Line(clause); changed = true; }
            var rendered = Render();
            foreach (var line in rendered.Lines.Where(line => line.Width > TargetWidth))
            {
                source.CheckCancellation();
                var list = layout.Lists.Where(list => list.Wrap && list.Items.Count != 0 &&
                    (!expanded.Contains(list) && rendered.Line[list.Open] == line.Number && rendered.Line[list.Close] == line.Number ||
                     expanded.Contains(list) && list.Items.Skip(1).Any(item => rendered.Line[item] == line.Number && layout.Breaks[item] == 0)))
                    .OrderByDescending(list => list.Close - list.Open).FirstOrDefault();
                if (list is not null) { Expand(list, allItems: true); changed = true; continue; }
                var conditional = layout.Conditionals.FirstOrDefault(value => !conditionals.Contains(value) && rendered.Line[value.Start] == line.Number && rendered.Line[value.End - 1] == line.Number);
                if (conditional is not null) { Expand(conditional); changed = true; continue; }
                var candidates = layout.Binary.Where(binary =>
                    rendered.Line[binary.Operator] == line.Number && layout.Breaks[BreakIndex(binary)] == 0 &&
                    rendered.Column[binary.Operator] > line.Indent && binary.Right < tokens.Count).OrderBy(binary => binary.Operator).ToArray();
                var selected = candidates.LastOrDefault(binary => rendered.Column[binary.Operator] < TargetWidth)
                    ?? candidates.FirstOrDefault();
                if (selected is not null) { BreakBinary(selected); changed = true; }
            }
            if (!changed) return rendered.Text;
        }
    }

    private void PreserveSections()
    {
        for (var index = 1; index < tokens.Count - 1; index++)
        {
            source.CheckCancellation();
            if (layout.Breaks[index] != 0 && trivia.BlankBefore(index) && tokens[index].Text is not
                ("end" or "elsif" or "else" or "when" or "until" or "catch" or "finally")) layout.Section(index);
        }
    }

    private bool ExpandContainers()
    {
        var changed = false;
        foreach (var list in layout.Lists.OrderBy(value => value.Close - value.Open))
        {
            source.CheckCancellation();
            if (expanded.Contains(list) || list.Items.Count == 0) continue;
            if (Enumerable.Range(list.Open + 1, list.Close - list.Open - 1).Any(index => layout.Breaks[index] > 0))
            { Expand(list, allItems: false); changed = true; }
        }
        return changed;
    }

    private void Expand(SyntaxList list, bool allItems)
    {
        if (list.Items.Count == 0) return;
        var first = expanded.Add(list);
        layout.Line(list.Items[0]);
        foreach (var item in list.Items.Skip(1))
            if (allItems || trivia.HasBreak(End(item - 1), tokens[item].Position)) layout.Line(item);
        if (list.CloseLine) layout.Line(list.Close);
        if (first)
        {
            layout.Indent(list.Open + 1, list.CloseLine ? list.Close : list.Close + 1);
            if (list.CloseLine) layout.ClosingComments[list.Close]++;
        }
    }

    private void Expand(SyntaxConditional conditional)
    {
        if (!conditionals.Add(conditional)) return;
        foreach (var branch in conditional.Branches) layout.Line(branch);
        layout.Indent(conditional.Start + 1, conditional.End);
    }

    private void BreakBinary(SyntaxBinary binary)
    {
        var index = BreakIndex(binary);
        layout.Line(index);
        if (!continuations.TryGetValue(index, out var previous) || binary.Start < previous) continuations[index] = binary.Start;
    }

    private int BreakIndex(SyntaxBinary binary) => tokens[binary.Operator].Text is "and" or "or" or "xor" ? binary.Operator : binary.Right;
    private int End(int index) => tokens[index].Position + tokens[index].Length;
    private RenderedSource Render() => new LayoutWriter(source, tokens, trivia, layout, continuations).Write();
}
