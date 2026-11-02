using System.Text;
using System.Text.RegularExpressions;
using System.Xml.Linq;

namespace Fpas.Compiler.Documentation;

internal static class MarkdownXml
{
    private static readonly Regex ListItem = new("^( *)([-+*] |[0-9]+[.)] )(.*)$", RegexOptions.CultureInvariant);

    internal static IReadOnlyList<XNode> Render(IReadOnlyList<string> lines)
    {
        var result = new List<XNode>();
        for (var index = 0; index < lines.Count;)
        {
            if (string.IsNullOrWhiteSpace(lines[index])) { index++; continue; }
            char fence = '\0'; var length = 0;
            if (Fence(lines[index], ref fence, ref length))
            {
                index++; var code = new List<string>();
                while (index < lines.Count)
                {
                    var line = lines[index++];
                    if (Fence(line, ref fence, ref length) && fence == '\0') break;
                    code.Add(line);
                }
                result.Add(new XElement("code", Clean(string.Join('\n', code)))); continue;
            }
            var list = ListItem.Match(lines[index]);
            if (list.Success)
            {
                var ordered = char.IsAsciiDigit(list.Groups[2].Value[0]);
                var level = list.Groups[1].Length;
                var element = new XElement("list", new XAttribute("type", ordered ? "number" : "bullet"));
                while (index < lines.Count)
                {
                    var item = ListItem.Match(lines[index]);
                    if (!item.Success || item.Groups[1].Length != level || char.IsAsciiDigit(item.Groups[2].Value[0]) != ordered) break;
                    index++; var body = new List<string> { item.Groups[3].Value };
                    var indent = item.Groups[1].Length + item.Groups[2].Length;
                    while (index < lines.Count)
                    {
                        var line = lines[index];
                        if (string.IsNullOrWhiteSpace(line))
                        {
                            if (index + 1 >= lines.Count || !lines[index + 1].StartsWith(new string(' ', indent), StringComparison.Ordinal)) break;
                            body.Add(""); index++; continue;
                        }
                        if (!line.StartsWith(new string(' ', indent), StringComparison.Ordinal)) break;
                        body.Add(line[indent..]); index++;
                    }
                    element.Add(new XElement("item", new XElement("description", Render(body))));
                }
                result.Add(element); continue;
            }
            var paragraph = new List<string> { lines[index++].Trim() };
            while (index < lines.Count && !string.IsNullOrWhiteSpace(lines[index]) && !ListItem.IsMatch(lines[index]))
            {
                char nextFence = '\0'; var nextLength = 0;
                if (Fence(lines[index], ref nextFence, ref nextLength)) break;
                paragraph.Add(lines[index++].Trim());
            }
            result.Add(new XElement("para", Inline(string.Join(' ', paragraph))));
        }
        if (result.Count == 1 && result[0] is XElement { Name.LocalName: "para" } only) return only.Nodes().ToArray();
        return result;
    }

    internal static bool Fence(string line, ref char fence, ref int length)
    {
        var indentation = 0; while (indentation < line.Length && line[indentation] == ' ') indentation++;
        if (indentation > 3 || indentation == line.Length || line[indentation] is not ('`' or '~')) return false;
        var marker = line[indentation]; var end = indentation;
        while (end < line.Length && line[end] == marker) end++;
        var count = end - indentation;
        if (fence == '\0')
        {
            if (count < 3) return false;
            fence = marker; length = count; return true;
        }
        if (marker != fence || count < length || !string.IsNullOrWhiteSpace(line[end..])) return false;
        fence = '\0'; length = 0; return true;
    }

    private static IEnumerable<XNode> Inline(string text)
    {
        var plain = new StringBuilder();
        var nodes = new List<XNode>();
        void Flush() { if (plain.Length != 0) { nodes.Add(new XText(Clean(plain.ToString()))); plain.Clear(); } }
        for (var index = 0; index < text.Length;)
        {
            if (text[index] == '`')
            {
                var count = 1; while (index + count < text.Length && text[index + count] == '`') count++;
                var end = text.IndexOf(new string('`', count), index + count, StringComparison.Ordinal);
                if (end >= 0)
                {
                    Flush(); nodes.Add(new XElement("c", Clean(text[(index + count)..end]))); index = end + count; continue;
                }
            }
            if (text[index] == '[')
            {
                var labelEnd = text.IndexOf("](", index, StringComparison.Ordinal);
                if (labelEnd > index)
                {
                    var end = labelEnd + 2; var depth = 1;
                    while (end < text.Length && depth != 0)
                    {
                        if (text[end] == '(') depth++; else if (text[end] == ')') depth--;
                        if (depth != 0) end++;
                    }
                    if (depth == 0)
                    {
                        Flush(); nodes.Add(new XElement("see", new XAttribute("href", Clean(text[(labelEnd + 2)..end])), Inline(text[(index + 1)..labelEnd])));
                        index = end + 1; continue;
                    }
                }
            }
            plain.Append(text[index++]);
        }
        Flush(); return nodes;
    }

    private static string Clean(string text)
    {
        var result = new StringBuilder();
        foreach (var rune in text.EnumerateRunes())
            result.Append(rune.Value is 9 or 10 or 13 or >= 0x20 and <= 0xD7FF or >= 0xE000 and <= 0xFFFD or >= 0x10000 and <= 0x10FFFF ? rune.ToString() : "\uFFFD");
        return result.ToString();
    }
}
