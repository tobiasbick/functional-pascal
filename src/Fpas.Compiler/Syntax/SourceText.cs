namespace Fpas.Compiler.Syntax;

internal sealed class SourceText(string path, string text, CancellationToken cancellationToken = default)
{
    internal string Path { get; } = path;
    internal string Text { get; } = text;
    internal CancellationToken CancellationToken => cancellationToken;
    internal void CheckCancellation() => cancellationToken.ThrowIfCancellationRequested();

    internal SourceLocation Location(int position, int length = 1)
    {
        var line = 1;
        var column = 1;
        var nextCancellationCheck = 0;
        for (var index = 0; index < Math.Min(position, Text.Length); index++)
        {
            if (index >= nextCancellationCheck) { CheckCancellation(); nextCancellationCheck = index + 1024; }
            if (Text[index] == '\r')
            {
                if (index + 1 < position && Text[index + 1] == '\n')
                    index++;
                line++;
                column = 1;
            }
            else if (Text[index] == '\n')
            {
                line++;
                column = 1;
            }
            else
                column++;
        }
        return new(Path, line, column, Math.Min(length, Text.Length - position));
    }

    internal SourceLocation StatementLocation(int position)
    {
        var end = Text.IndexOfAny(['\r', '\n'], position);
        return Location(position, Math.Max(1, (end < 0 ? Text.Length : end) - position));
    }

    internal CompilationFailure Error(string code, string message, int position, int length = 1)
        => new(new(code, message, Location(position, length)));
}
