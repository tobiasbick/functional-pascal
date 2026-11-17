using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Snapshots;
using System.Text;

namespace Fpas.Workspaces.Live;

/// <summary>One-based UTF-16 coordinates; protocol adapters convert at their boundary.</summary>
/// <param name="Line">One-based source line.</param>
/// <param name="Column">One-based UTF-16 column; tabs count as one code unit.</param>
public sealed record TextPosition(int Line, int Column);
/// <summary>A half-open range in one document state.</summary>
/// <param name="Start">Inclusive start position.</param>
/// <param name="End">Exclusive end position.</param>
public sealed record TextRange(TextPosition Start, TextPosition End);
/// <summary>Replaces a range, or the whole document when the range is absent.</summary>
/// <param name="Text">Replacement text.</param>
/// <param name="Range">Range in the state produced by preceding changes, or null for full replacement.</param>
public sealed record TextChange(string Text, TextRange? Range = null);
/// <summary>Bounds live text, retained analysis and operation duration.</summary>
/// <param name="DocumentBytes">Maximum UTF-8 bytes in one document.</param>
/// <param name="OpenBytes">Maximum aggregate UTF-8 bytes in open documents.</param>
/// <param name="RetainedBytes">Maximum accounted bytes in analysis and cache data.</param>
/// <param name="WorkTimeout">Operation deadline, or null for thirty seconds.</param>
public sealed record LiveWorkspaceLimits(int DocumentBytes = 4_194_304, int OpenBytes = 67_108_864,
    long RetainedBytes = 134_217_728, TimeSpan? WorkTimeout = null);

internal static class DocumentChanges
{
    internal static TextDocumentSnapshot Apply(TextDocumentSnapshot original, int version, IReadOnlyList<TextChange> changes, LiveWorkspaceLimits limits)
    {
        if (version <= original.Version) throw new ContextFailure("invalid_version", "Document versions must increase.");
        if (changes.Count == 0 || changes.Count > 1000) throw new ContextFailure("invalid_change", "Supply 1..1,000 ordered text changes.");
        var text = original.Text;
        foreach (var change in changes)
        {
            if (change.Range is not { } range) text = change.Text;
            else
            {
                var start = Offset(text, range.Start); var end = Offset(text, range.End);
                if (end < start) throw new ContextFailure("invalid_change", "A text range must not end before its start.");
                if ((long)text.Length - (end - start) + change.Text.Length > 2L * limits.DocumentBytes)
                    throw new ContextFailure("workspace_limit", "Document text exceeds its budget.");
                text = string.Concat(text.AsSpan(0, start), change.Text, text.AsSpan(end));
            }
            CheckSize(text, limits);
        }
        return new(original.Path, text, version);
    }

    internal static int Bytes(string text) => Encoding.UTF8.GetByteCount(text);
    internal static void CheckSize(string text, LiveWorkspaceLimits limits)
    {
        if (text.Length > 2L * limits.DocumentBytes || Bytes(text) > limits.DocumentBytes)
            throw new ContextFailure("workspace_limit", "Document text exceeds its UTF-8 budget.");
    }

    private static int Offset(string text, TextPosition position)
    {
        if (position.Line < 1 || position.Column < 1) throw new ContextFailure("invalid_change", "Text coordinates must be positive.");
        var line = 1; var start = 0;
        for (var index = 0; index < text.Length && line < position.Line; index++)
        {
            if (text[index] == '\r') { if (index + 1 < text.Length && text[index + 1] == '\n') index++; line++; start = index + 1; }
            else if (text[index] == '\n') { line++; start = index + 1; }
        }
        if (line != position.Line) throw new ContextFailure("invalid_change", "Text line is outside the document.");
        var end = start; while (end < text.Length && text[end] is not ('\r' or '\n')) end++;
        if ((long)start + position.Column - 1 > end) throw new ContextFailure("invalid_change", "Text column is outside the line.");
        return start + position.Column - 1;
    }
}
