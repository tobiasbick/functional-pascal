namespace Fpas.Compiler.Syntax;

internal enum ScalarCategory { Distinct, Range, Enum }
internal sealed record AliasSyntax(Token Name, bool IsPublic, TypeSyntax Target, DocumentationComment? Documentation);
internal sealed record ScalarSyntax(Token Name, bool IsPublic, ScalarCategory Category, TypeSyntax Base,
    ExpressionSyntax? Lower, ExpressionSyntax? Upper, IReadOnlyList<EnumMemberSyntax> Members, DocumentationComment? Documentation, IReadOnlyList<AttributeSyntax>? Attributes = null);
internal sealed record EnumMemberSyntax(Token Name, ExpressionSyntax? Value, DocumentationComment? Documentation);
