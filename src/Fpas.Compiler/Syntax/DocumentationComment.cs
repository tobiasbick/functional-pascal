namespace Fpas.Compiler.Syntax;

internal sealed record DocumentationLine(string Text, int Position);
internal sealed record DocumentationComment(IReadOnlyList<DocumentationLine> Lines);
