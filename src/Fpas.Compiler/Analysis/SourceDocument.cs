namespace Fpas.Compiler.Analysis;

public sealed record SourceSpan(string Path, int Offset, int Length, int Line, int Column)
{
    public bool Contains(int offset) => offset >= Offset && offset < Offset + Length;
}

public sealed class SourceDocument
{
    public string Path { get; }
    public string Text { get; }
    private readonly int[] lines;

    internal SourceDocument(string path, string text, CancellationToken token)
    {
        Path = path; Text = text;
        var starts = new List<int> { 0 };
        var nextCheck = 0;
        for (var index = 0; index < text.Length; index++)
        {
            if (index >= nextCheck) { token.ThrowIfCancellationRequested(); nextCheck = index + 1024; }
            if (text[index] == '\r')
            {
                if (index + 1 < text.Length && text[index + 1] == '\n') index++;
                starts.Add(index + 1);
            }
            else if (text[index] == '\n') starts.Add(index + 1);
        }
        lines = starts.ToArray();
    }

    public int Offset(int line, int column)
    {
        if (line < 1 || line > lines.Length || column < 1) throw new ArgumentOutOfRangeException(nameof(line), "Coordinates must identify a source line and a one-based UTF-16 column.");
        var start = lines[line - 1]; var end = line == lines.Length ? Text.Length : lines[line];
        while (end > start && Text[end - 1] is '\r' or '\n') end--;
        if ((long)start + column - 1 > end) throw new ArgumentOutOfRangeException(nameof(column), "Column lies beyond the source line.");
        return start + column - 1;
    }

    internal SourceSpan Span(int offset, int length)
    {
        if (offset < 0 || length < 0 || (long)offset + length > Text.Length) throw new ArgumentOutOfRangeException(nameof(offset));
        var line = Array.BinarySearch(lines, offset); if (line < 0) line = ~line - 1;
        return new(Path, offset, length, line + 1, offset - lines[line] + 1);
    }
}
