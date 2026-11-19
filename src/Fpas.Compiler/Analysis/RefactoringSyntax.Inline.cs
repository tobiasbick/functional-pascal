// [Documentation](../../../docs/pascal/tools/refactoring.md#inline-a-local-variable)
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Analysis;

/// <summary>The source of one local initializer and the statement that stores it.</summary>
/// <param name="Statement">The whole declaration statement, including its semicolon.</param>
/// <param name="Initializer">The original expression text to move or copy.</param>
/// <param name="Literal">Whether the initializer is a literal that needs no new evaluation.</param>
/// <param name="HasSuspensionOrRoutine">Whether it contains an await or defines a routine.</param>
public sealed record RefactoringInitializer(SourceSpan Statement, SourceSpan Initializer, bool Literal, bool HasSuspensionOrRoutine);

public sealed partial class RefactoringSyntax
{
    /// <summary>Finds a local declaration with explicit storage and an initializer.</summary>
    /// <param name="offset">Start of the declared name token.</param>
    /// <returns>The initializer information, or null for bindings outside editable statement lists.</returns>
    public RefactoringInitializer? InlineInitializer(int offset)
    {
        if (FindBinding(offset) is not { } found) return null;
        var declaration = unit.Declarations[offset];
        var initializer = found.Binding.Initializer;
        var value = initializer;
        while (value is ParenthesizedSyntax parenthesized) value = parenthesized.Expression;
        var literal = value is LiteralSyntax ||
            value is UnarySyntax { Operator.Text: "-", Operand: LiteralSyntax { Token: { Kind: TokenKind.Integer } number } } &&
            !number.Text.EndsWith("u", StringComparison.OrdinalIgnoreCase);
        var special = tokens.Any(item => item.Position >= initializer.SpanStart && item.Position < initializer.SpanEnd &&
            item.Kind == TokenKind.Keyword && item.Text is "await" or "function" or "procedure");
        return new(document.Span(declaration.Start, declaration.End - declaration.Start),
            document.Span(initializer.SpanStart, initializer.SpanEnd - initializer.SpanStart), literal, special);
    }

    /// <summary>Checks whether the read is the first evaluated value of the immediately following statement in the same block.</summary>
    /// <param name="offset">Start of the local declaration's name token.</param>
    /// <param name="read">Span of the original local read.</param>
    /// <returns>Calls whose receivers need semantic verification, or null when evaluation cannot be moved here.</returns>
    public IReadOnlyList<(int Offset, bool RequiresStatic)>? FirstReadAfter(int offset, SourceSpan read)
    {
        if (FindBinding(offset) is not { Next: { } next }) return null;
        var expression = next is ExpressionStatementSyntax statement ? statement.Expression : FirstExpression(next);
        List<(int Offset, bool RequiresStatic)> calls = [];
        return expression is not null && EvaluatedFirst(expression, read.Offset, read.Offset + read.Length, calls, allowQualified: true)
            ? calls : null;
    }

    private (BindingSyntax Binding, StatementSyntax? Next)? FindBinding(int offset)
    {
        foreach (var statements in StatementLists())
            for (var index = 0; index < statements.Count; index++)
            {
                token.ThrowIfCancellationRequested();
                if (statements[index] is BindingSyntax binding && binding.Name.Position == offset)
                    return (binding, index + 1 < statements.Count ? statements[index + 1] : null);
            }
        return null;
    }
}
