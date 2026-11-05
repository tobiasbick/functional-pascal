namespace Fpas.Compiler.Syntax.Parsing;

internal sealed partial class Parser
{
    private IReadOnlyList<T> ReadList<T>(string open, string close, string separator, Func<T> read,
        bool empty = true, bool wrap = true, bool tight = false)
    {
        var start = position;
        var list = layout?.List(start, wrap);
        Expect(open);
        var values = new List<T>();
        if (!empty || !At(close))
        {
            do
            {
                list?.Item(position);
                values.Add(read());
                if (!At(separator)) break;
                position++;
            } while (true);
        }
        Expect(close);
        list?.End(position - 1);
        if (tight && layout is not null) { layout.Tight.Add(start); layout.Tight.Add(position - 1); }
        return values;
    }
}
