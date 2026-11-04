namespace Fpas.Compiler.Syntax;

internal sealed record DocumentationLine(string Text, int Position, int Start, int End);
internal sealed record DocumentationComment(IReadOnlyList<DocumentationLine> Lines);
