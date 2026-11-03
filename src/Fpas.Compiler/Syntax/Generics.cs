namespace Fpas.Compiler.Syntax;

internal sealed record ConstraintSyntax(int Position, string? Kind, TypeSyntax? Type);
internal sealed record WhereSyntax(Token Parameter, IReadOnlyList<ConstraintSyntax> Constraints);
