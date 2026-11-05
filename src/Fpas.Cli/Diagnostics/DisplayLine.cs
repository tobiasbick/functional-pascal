using System.Globalization;
using System.Text;

namespace Fpas.Cli.Diagnostics;

internal static class DisplayLine
{
    internal sealed record Rendered(string Text, int Column, int Width);

    internal static Rendered Render(string line, int offset, int length)
    {
        var cells = new Queue<(string Text, int Column, int Width)>();
        var elements = StringInfo.GetTextElementEnumerator(line);
        var column = 0;
        int? marker = null;
        var extent = 0;
        var truncated = false;
        while (elements.MoveNext())
        {
            var value = elements.GetTextElement();
            var start = elements.ElementIndex;
            if (marker is null && start + value.Length > offset) marker = column;
            if (column >= 120)
            {
                var left = Math.Max(0, (marker ?? column) - 40);
                while (cells.Count > 0 && cells.Peek().Column + cells.Peek().Width <= left) cells.Dequeue();
                if (marker is not null && column >= left + 120) { truncated = true; break; }
            }
            var display = value.Length > 64 ? "[grapheme]" : value == "\t" ? new string(' ', 4 - column % 4) : DiagnosticRenderer.Safe(value);
            var width = value == "\t" || value.Length > 64 ? display.Length : Width(value, display);
            if (width == 0) { display = string.Concat(value.EnumerateRunes().Select(rune => $"\\u{rune.Value:X4}")); width = display.Length; }
            if (start + value.Length > offset && start < offset + Math.Max(1, length)) extent += width;
            cells.Enqueue((display, column, width)); column += width;
        }
        marker ??= column;
        var skipped = cells.Count > 0 ? cells.Peek().Column : 0;
        var prefix = skipped > 0 ? "…" : "";
        var text = prefix + string.Concat(cells.Select(cell => cell.Text)) + (truncated ? "…" : "");
        return new(text, prefix.Length + marker.Value - skipped, Math.Clamp(extent, 1, Math.Max(1, 120 - (marker.Value - skipped))));
    }

    private static int Width(string value, string display)
    {
        if (value != display) return display.Length;
        var width = 0;
        var emojiPresentation = value.Contains('\uFE0F') || value.Contains('\u20E3');
        foreach (var rune in value.EnumerateRunes())
        {
            if (Rune.GetUnicodeCategory(rune) is UnicodeCategory.NonSpacingMark or UnicodeCategory.EnclosingMark or UnicodeCategory.Format) continue;
            var wide = rune.Value is >= 0x1100 and <= 0x115F or 0x2329 or 0x232A or >= 0x2E80 and <= 0xA4CF
                or >= 0xAC00 and <= 0xD7A3 or >= 0xF900 and <= 0xFAFF or >= 0xFE10 and <= 0xFE19
                or >= 0xFE30 and <= 0xFE6F or >= 0xFF01 and <= 0xFF60 or >= 0xFFE0 and <= 0xFFE6
                or >= 0x1F000 and <= 0x1FAFF or >= 0x20000 and <= 0x3FFFD;
            width = Math.Max(width, wide ? 2 : 1);
        }
        return emojiPresentation ? 2 : width;
    }
}
