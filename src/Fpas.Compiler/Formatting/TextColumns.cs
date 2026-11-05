using System.Globalization;
using System.Text;

namespace Fpas.Compiler.Formatting;

internal static class TextColumns
{
    internal static int Width(string text, int column, CancellationToken cancellationToken)
    {
        var width = 0;
        var elements = StringInfo.GetTextElementEnumerator(text);
        var count = 0;
        while (elements.MoveNext())
        {
            if ((count++ & 1023) == 0) cancellationToken.ThrowIfCancellationRequested();
            var element = elements.GetTextElement();
            if (element == "\t") { width += 4 - (column + width) % 4; continue; }
            var cells = element.Contains('\uFE0F') || element.Contains('\u20E3') ? 2 : 0;
            foreach (var rune in element.EnumerateRunes())
            {
                var category = Rune.GetUnicodeCategory(rune);
                if (category is UnicodeCategory.NonSpacingMark or UnicodeCategory.EnclosingMark or UnicodeCategory.Format) continue;
                var value = rune.Value;
                cells = Math.Max(cells, value is >= 0x1100 and <= 0x115F or >= 0x2329 and <= 0x232A or
                    >= 0x2E80 and <= 0xA4CF or >= 0xAC00 and <= 0xD7A3 or >= 0xF900 and <= 0xFAFF or
                    >= 0xFE10 and <= 0xFE19 or >= 0xFE30 and <= 0xFE6F or >= 0xFF00 and <= 0xFF60 or
                    >= 0xFFE0 and <= 0xFFE6 or >= 0x1F000 and <= 0x1FAFF or >= 0x20000 and <= 0x3FFFD ? 2 : 1);
            }
            width += cells;
        }
        return width;
    }
}
