// [Documentation](../../../docs/architecture/compiler.md#semantic-snapshots)
namespace Fpas.Compiler.Analysis;

/// <summary>A UTF-16 range in one source with the one-based coordinates of its start.</summary>
/// <param name="Path">The logical source path.</param>
/// <param name="Offset">The zero-based start offset.</param>
/// <param name="Length">The length in UTF-16 code units.</param>
/// <param name="Line">The one-based start line.</param>
/// <param name="Column">The one-based start column.</param>
public sealed record SourceSpan(string Path, int Offset, int Length, int Line, int Column)
{
    /// <summary>Tests whether an offset lies inside the half-open range.</summary>
    /// <param name="offset">The zero-based offset to test.</param>
    /// <returns>Whether <paramref name="offset"/> is at least the start and before the end.</returns>
    public bool Contains(int offset) => offset >= Offset && offset < Offset + Length;
}

/// <summary>Source text with a line index for converting between offsets and coordinates.</summary>
public sealed class SourceDocument
{
    /// <summary>The logical source path.</summary>
    public string Path { get; }
    /// <summary>The complete source text.</summary>
    public string Text { get; }
    private readonly int[] lines;

    /// <summary>Indexes the lines of a source; CR, LF, and CRLF each end a line.</summary>
    /// <param name="path">The logical source path.</param>
    /// <param name="text">The complete source text.</param>
    /// <param name="token">Cancels indexing with an <see cref="OperationCanceledException"/>.</param>
    public SourceDocument(string path, string text, CancellationToken token = default)
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

    /// <summary>Converts one-based coordinates to an offset.</summary>
    /// <param name="line">The one-based line.</param>
    /// <param name="column">The one-based UTF-16 column; the position after the last character of the line is allowed.</param>
    /// <returns>The zero-based offset.</returns>
    /// <exception cref="ArgumentOutOfRangeException">The coordinates lie outside the source.</exception>
    public int Offset(int line, int column)
    {
        if (line < 1 || line > lines.Length || column < 1) throw new ArgumentOutOfRangeException(nameof(line), "Coordinates must identify a source line and a one-based UTF-16 column.");
        var start = lines[line - 1]; var end = line == lines.Length ? Text.Length : lines[line];
        while (end > start && Text[end - 1] is '\r' or '\n') end--;
        if ((long)start + column - 1 > end) throw new ArgumentOutOfRangeException(nameof(column), "Column lies beyond the source line.");
        return start + column - 1;
    }

    /// <summary>Creates a span with the coordinates of its start.</summary>
    /// <param name="offset">The zero-based start offset.</param>
    /// <param name="length">The length in UTF-16 code units.</param>
    /// <returns>The span inside this source.</returns>
    /// <exception cref="ArgumentOutOfRangeException">The range lies outside the source.</exception>
    public SourceSpan Span(int offset, int length)
    {
        if (offset < 0 || length < 0 || (long)offset + length > Text.Length) throw new ArgumentOutOfRangeException(nameof(offset));
        var line = Array.BinarySearch(lines, offset); if (line < 0) line = ~line - 1;
        return new(Path, offset, length, line + 1, offset - lines[line] + 1);
    }
}
