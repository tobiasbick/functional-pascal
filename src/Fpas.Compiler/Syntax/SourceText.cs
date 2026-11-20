// [Documentation](../../../docs/architecture/compiler.md#semantic-snapshots)
using Fpas.Compiler.Diagnostics;
using Fpas.Compiler.Analysis;

namespace Fpas.Compiler.Syntax;

internal sealed class SourceText(string path, string text, CancellationToken cancellationToken = default, DiagnosticBag? diagnostics = null)
{
    // Start offsets of each line, computed once so locations need no rescan of the text.
    private int[]? lineStarts;

    internal string Path { get; } = path;
    internal string Text { get; } = text;
    internal CancellationToken CancellationToken => cancellationToken;
    internal DiagnosticBag Diagnostics { get; } = diagnostics ?? new();
    internal EditorProbe? Editor { get; set; }
    internal void CheckCancellation() => cancellationToken.ThrowIfCancellationRequested();

    internal SourceLocation Location(int position, int length = 1)
    {
        position = Math.Clamp(position, 0, Text.Length);
        length = Math.Clamp(length, 0, Text.Length - position);
        var (startLine, startColumn) = Coordinates(position);
        var (endLine, endColumn) = Coordinates(position + length);
        return new(Path, startLine, startColumn, length, endLine, endColumn, position);
    }

    private (int Line, int Column) Coordinates(int position)
    {
        var starts = lineStarts ??= LineStarts();
        // The LF of a CRLF pair already belongs to the following line at column 1.
        if (position > 0 && position < Text.Length && Text[position] == '\n' && Text[position - 1] == '\r') position++;
        var line = Array.BinarySearch(starts, position);
        if (line < 0) line = ~line - 1;
        return (line + 1, position - starts[line] + 1);
    }

    private int[] LineStarts()
    {
        var starts = new List<int> { 0 };
        var nextCancellationCheck = 0;
        for (var index = 0; index < Text.Length; index++)
        {
            if (index >= nextCancellationCheck) { CheckCancellation(); nextCancellationCheck = index + 1024; }
            if (Text[index] == '\r')
            {
                if (index + 1 < Text.Length && Text[index + 1] == '\n') index++;
                starts.Add(index + 1);
            }
            else if (Text[index] == '\n') starts.Add(index + 1);
        }
        return starts.ToArray();
    }

    internal SourceLocation StatementLocation(int position)
    {
        var end = Text.IndexOfAny(['\r', '\n'], position);
        return Location(position, Math.Max(1, (end < 0 ? Text.Length : end) - position));
    }

    internal CompilationFailure Error(string code, string message, int position, int length = 1,
        string? help = null, DiagnosticExpectation? expectation = null, IReadOnlyList<DiagnosticNote>? related = null,
        IReadOnlyList<DiagnosticFix>? fixes = null, DiagnosticArgument? argument = null)
        => new(DiagnosticHints.Enrich(new(code, message, Location(position, length))
            { Help = help, Expectation = expectation, Argument = argument, Related = related ?? [], Fixes = fixes ?? [] }, this, position));
}
