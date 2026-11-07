using Fpas.Compiler.Diagnostics;
using Fpas.Compiler.Analysis;

namespace Fpas.Compiler.Syntax;

internal sealed class SourceText(string path, string text, CancellationToken cancellationToken = default, DiagnosticBag? diagnostics = null)
{
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
        var line = 1;
        var column = 1;
        var startLine = 1; var startColumn = 1;
        var nextCancellationCheck = 0;
        for (var index = 0; index <= position + length; index++)
        {
            if (index >= nextCancellationCheck) { CheckCancellation(); nextCancellationCheck = index + 1024; }
            if (index == position) { startLine = line; startColumn = column; }
            if (index == position + length) break;
            if (Text[index] == '\r')
            {
                line++;
                column = 1;
            }
            else if (Text[index] == '\n')
            {
                if (index == 0 || Text[index - 1] != '\r') line++;
                column = 1;
            }
            else
                column++;
        }
        return new(Path, startLine, startColumn, length, line, column, position);
    }

    internal SourceLocation StatementLocation(int position)
    {
        var end = Text.IndexOfAny(['\r', '\n'], position);
        return Location(position, Math.Max(1, (end < 0 ? Text.Length : end) - position));
    }

    internal CompilationFailure Error(string code, string message, int position, int length = 1,
        string? help = null, DiagnosticExpectation? expectation = null, IReadOnlyList<DiagnosticNote>? related = null,
        IReadOnlyList<DiagnosticFix>? fixes = null)
        => new(DiagnosticHints.Enrich(new(code, message, Location(position, length))
            { Help = help, Expectation = expectation, Related = related ?? [], Fixes = fixes ?? [] }, this, position));
}
