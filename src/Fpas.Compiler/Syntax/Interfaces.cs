namespace Fpas.Compiler.Syntax;

internal sealed record InterfaceSyntax(Token Name, bool IsPublic, IReadOnlyList<InterfacePropertySyntax> Properties,
    IReadOnlyList<RoutineSyntax> Methods, IReadOnlyList<Token> TypeParameters, IReadOnlyList<WhereSyntax> Constraints,
    IReadOnlyList<TypeSyntax> Bases, DocumentationComment? Documentation = null, IReadOnlyList<AttributeSyntax>? Attributes = null);

internal sealed record InterfacePropertySyntax(Token Name, TypeSyntax Type, DocumentationComment? Documentation = null,
    IReadOnlyList<AttributeSyntax>? Attributes = null);
