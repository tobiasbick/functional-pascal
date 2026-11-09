namespace Fpas.Workspaces.Documentation;

internal static class DocumentationMarkdown
{
    internal static string Source(string comments)
        => string.Join('\n', comments.Replace("\r\n", "\n", StringComparison.Ordinal).Replace('\r', '\n').Split('\n').Select(line =>
        {
            var start = line.TrimStart();
            if (!start.StartsWith("//", StringComparison.Ordinal)) return line;
            var text = start[2..];
            return text.StartsWith(' ') ? text[1..] : text;
        }));

    internal static string PlainText(string text)
        => string.Concat(text.Select(character =>
            "\\`*_{}[]<>()#+-.!|~&".Contains(character) ? "\\" + character : character.ToString()));
}
