// [Documentation](../../../docs/pascal/interop/attributes.md)
namespace Fpas.Compiler.Syntax;

internal sealed record AttributeSyntax(TypeSyntax Type, IReadOnlyList<ExpressionSyntax> Arguments);
