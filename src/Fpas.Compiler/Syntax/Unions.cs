namespace Fpas.Compiler.Syntax;

internal sealed record UnionSyntax(Token Name, bool IsPublic, IReadOnlyList<UnionCaseSyntax> Cases,
    IReadOnlyList<Token>? TypeParameters = null, IReadOnlyList<WhereSyntax>? Constraints = null, DocumentationComment? Documentation = null, IReadOnlyList<AttributeSyntax>? Attributes = null);
internal sealed record UnionCaseSyntax(Token Name, IReadOnlyList<ParameterSyntax> Fields, DocumentationComment? Documentation = null);
internal sealed record CaseArmSyntax(int Position, IReadOnlyList<PatternSyntax> Patterns, ExpressionSyntax? Guard,
    IReadOnlyList<StatementSyntax>? Statements, ExpressionSyntax? Value);
internal sealed record CaseStatementSyntax(int Start, ExpressionSyntax Selector, IReadOnlyList<CaseArmSyntax> Arms) : StatementSyntax(Start);
internal sealed record CaseExpressionSyntax(int Start, ExpressionSyntax Selector, IReadOnlyList<CaseArmSyntax> Arms) : ExpressionSyntax(Start);
internal sealed record TypeTestSyntax(ExpressionSyntax Value, TypeSyntax Target) : ExpressionSyntax(Value.Position);
internal abstract record PatternSyntax(int Position);
internal sealed record AnyPatternSyntax(int Start) : PatternSyntax(Start);
internal sealed record BindingPatternSyntax(Token Name, TypeSyntax Type) : PatternSyntax(Name.Position);
internal sealed record TypePatternSyntax(int Start, TypeSyntax Type, Token Name) : PatternSyntax(Start);
internal sealed record ConstantPatternSyntax(ExpressionSyntax Value) : PatternSyntax(Value.Position);
internal sealed record RangePatternSyntax(ExpressionSyntax Lower, ExpressionSyntax Upper) : PatternSyntax(Lower.Position);
internal sealed record NamedPatternSyntax(NameSyntax Name, IReadOnlyList<PayloadPatternSyntax>? Fields) : PatternSyntax(Name.Position);
internal sealed record PayloadPatternSyntax(Token? Name, PatternSyntax Pattern);
