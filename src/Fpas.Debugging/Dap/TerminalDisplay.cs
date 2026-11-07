using System.Globalization;
using System.Text;

namespace Fpas.Debugging.Dap;

// Tracks the cursor and cell contents of the VT stream independently of the TUI frame.
internal sealed class TerminalDisplay(int columns, int rows)
{
    private readonly Decoder decoder = Encoding.UTF8.GetDecoder();
    private string?[,] cells = new string?[rows, columns];
    private readonly StringBuilder sequence = new();
    private int x, y;
    private char state;
    private bool wrap = true;

    internal void Resize(int width, int height)
    {
        if (cells.GetLength(1) == width && cells.GetLength(0) == height) return;
        var next = new string?[height, width];
        for (var row = 0; row < Math.Min(height, cells.GetLength(0)); row++)
            for (var col = 0; col < Math.Min(width, cells.GetLength(1)); col++)
                next[row, col] = cells[row, col];
        cells = next;
        x = Math.Clamp(x, 0, width - 1);
        y = Math.Clamp(y, 0, height - 1);
    }

    internal void Read(byte[] bytes)
    {
        var decoded = new char[Encoding.UTF8.GetMaxCharCount(bytes.Length)];
        var length = decoder.GetChars(bytes, 0, bytes.Length, decoded, 0);
        var elements = StringInfo.GetTextElementEnumerator(new string(decoded, 0, length));
        while (elements.MoveNext()) Read(elements.GetTextElement());
    }

    internal string[] Lines()
    {
        var lines = new string[cells.GetLength(0)];
        for (var row = 0; row < lines.Length; row++)
        {
            var line = new StringBuilder();
            for (var col = 0; col < cells.GetLength(1); col++) line.Append(cells[row, col] ?? " ");
            lines[row] = line.ToString();
        }
        return lines;
    }

    private void Read(string glyph)
    {
        var c = glyph[0];
        if (state == ']')
        {
            if (c == '\a') state = (char)0;
            else if (c == 27) state = '\\';
            return;
        }
        if (state == '\\') { state = c == '\\' ? (char)0 : ']'; return; }
        if (state == '[')
        {
            if (c is >= '@' and <= '~')
            {
                state = (char)0;
                Apply(c, sequence.ToString());
                sequence.Clear();
            }
            else if (sequence.Length < 96) sequence.Append(c);
            return;
        }
        if (state == 27)
        {
            state = c is '[' or ']' ? c : (char)0;
            return;
        }
        if (c == 27) { state = c; return; }
        if (glyph == "\r\n") { x = 0; NewLine(); return; }
        if (c == '\r') { x = 0; return; }
        if (c == '\n') { NewLine(); return; }
        if (c == '\b') { x = Math.Max(0, x - 1); return; }
        if (char.IsControl(c)) return;
        var width = cells.GetLength(1);
        if (x >= width)
        {
            if (wrap) { x = 0; NewLine(); }
            else x = width - 1;
        }
        cells[y, x] = glyph;
        var rune = Rune.GetRuneAt(glyph, 0).Value;
        var wide = rune is >= 4352 and <= 4447 or >= 11904 and <= 42191 or >= 44032 and <= 55203
            or >= 63744 and <= 64255 or >= 65040 and <= 65376 or >= 65504 and <= 65510
            or >= 127744 and <= 129791 or >= 131072 and <= 196605 || glyph.Contains('\ufe0f');
        if (wide && x + 1 < width) cells[y, ++x] = "";
        x++;
    }

    private void NewLine()
    {
        if (y + 1 < cells.GetLength(0)) { y++; return; }
        for (var row = 1; row < cells.GetLength(0); row++)
            for (var col = 0; col < cells.GetLength(1); col++) cells[row - 1, col] = cells[row, col];
        for (var col = 0; col < cells.GetLength(1); col++) cells[y, col] = null;
    }

    private void Apply(char command, string arguments)
    {
        if (arguments == "?7" && command is 'h' or 'l') { wrap = command == 'h'; return; }
        if (arguments == "?1049" && command is 'h' or 'l') { Array.Clear(cells); x = 0; y = 0; return; }
        var parts = arguments.Split(';');
        int Value(int position, int fallback) => position < parts.Length && int.TryParse(parts[position], out var value) ? value : fallback;
        var width = cells.GetLength(1);
        var height = cells.GetLength(0);
        switch (command)
        {
            case 'H' or 'f':
                y = Math.Clamp(Value(0, 1) - 1, 0, height - 1);
                x = Math.Clamp(Value(1, 1) - 1, 0, width - 1);
                break;
            case 'A': y = Math.Max(0, y - Value(0, 1)); break;
            case 'B': y = Math.Min(height - 1, y + Value(0, 1)); break;
            case 'C': x = Math.Min(width - 1, x + Value(0, 1)); break;
            case 'D': x = Math.Max(0, x - Value(0, 1)); break;
            case 'G': x = Math.Clamp(Value(0, 1) - 1, 0, width - 1); break;
            case 'd': y = Math.Clamp(Value(0, 1) - 1, 0, height - 1); break;
            case 'J' when Value(0, 0) is 2 or 3: Array.Clear(cells); break;
            case 'K':
                var mode = Value(0, 0);
                for (var col = mode == 0 ? x : 0; col < (mode == 1 ? Math.Min(width, x + 1) : width); col++)
                    cells[y, col] = null;
                break;
            case 't' when Value(0, 0) == 8 && Value(1, 0) is >= 1 and <= 300 && Value(2, 0) is >= 1 and <= 500:
                Resize(Value(2, 0), Value(1, 0));
                break;
        }
    }
}
