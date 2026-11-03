namespace Fpas.Compiler.Syntax;

internal sealed record NameSyntax(IReadOnlyList<Token> Parts)
{
    internal string Text => string.Join('.', Parts.Select(part => part.Text));
    internal int Position => Parts[0].Position;
}

internal sealed record TypeSyntax(NameSyntax Name, bool Nullable = false, IReadOnlyList<TypeSyntax>? Arguments = null, TypeSyntax? Element = null, int Rank = 0, IReadOnlyList<TypeSyntax>? CallableParameters = null, TypeSyntax? CallableResult = null)
{
    internal int Position => Name.Position;
    internal string Text => Name.Text;
}

internal sealed record CompilationUnitSyntax(NameSyntax Name, bool IsProgram, IReadOnlyList<ImportSyntax> Imports,
    IReadOnlyList<RoutineSyntax> Routines, IReadOnlyList<StatementSyntax> Statements, IReadOnlyList<RecordSyntax> Records, IReadOnlyList<UnionSyntax> Unions, bool IsAsync = false, DocumentationComment? Documentation = null,
    IReadOnlyList<AliasSyntax>? Aliases = null, IReadOnlyList<ScalarSyntax>? Scalars = null, IReadOnlyList<GlobalSyntax>? Globals = null);
internal sealed record GlobalSyntax(BindingSyntax Binding, bool IsPublic);
internal sealed record RoutineSyntax(Token Name, bool IsPublic, IReadOnlyList<ParameterSyntax> Parameters,
    TypeSyntax? ResultType, IReadOnlyList<StatementSyntax> Statements, bool IsStatic = true, IReadOnlyList<Token>? TypeParameters = null, IReadOnlyList<WhereSyntax>? Constraints = null, IReadOnlyList<ContractSyntax>? Contracts = null, bool IsAsync = false, DocumentationComment? Documentation = null);
internal sealed record ContractSyntax(Token Keyword, ParameterSyntax? Result, ExpressionSyntax Condition, string Text);
internal sealed record ParameterSyntax(Token Name, TypeSyntax Type, string Mode = "", ExpressionSyntax? Default = null);
internal sealed record ArgumentModeSyntax(int Start, string Mode, ExpressionSyntax Value) : ExpressionSyntax(Start);
internal sealed record ImportSyntax(NameSyntax Target, Token Alias);
internal abstract record StatementSyntax(int Position);
internal sealed record NestedRoutineSyntax(RoutineSyntax Routine) : StatementSyntax(Routine.Name.Position);
internal sealed record BindingSyntax(Token Name, TypeSyntax Type, ExpressionSyntax Initializer, bool Mutable, DocumentationComment? Documentation = null) : StatementSyntax(Name.Position);
internal sealed record AssignmentSyntax(ExpressionSyntax Target, ExpressionSyntax Value) : StatementSyntax(Target.Position);
internal sealed record ExpressionStatementSyntax(ExpressionSyntax Expression, bool Discard = false) : StatementSyntax(Expression.Position);
internal sealed record ReturnSyntax(int Start, ExpressionSyntax? Expression) : StatementSyntax(Start);
internal sealed record BlockSyntax(int Start, IReadOnlyList<StatementSyntax> Statements) : StatementSyntax(Start);
internal sealed record IfArmSyntax(ExpressionSyntax Condition, IReadOnlyList<StatementSyntax> Statements);
internal sealed record IfSyntax(int Start, IReadOnlyList<IfArmSyntax> Arms, IReadOnlyList<StatementSyntax> Else) : StatementSyntax(Start);
internal sealed record WhileSyntax(int Start, ExpressionSyntax Condition, IReadOnlyList<StatementSyntax> Body) : StatementSyntax(Start);
internal sealed record ForSyntax(int Start, Token Name, TypeSyntax Type, ExpressionSyntax Lower, ExpressionSyntax Upper,
    bool Descending, IReadOnlyList<StatementSyntax> Body) : StatementSyntax(Start);
internal sealed record LoopExitSyntax(int Start, bool IsBreak) : StatementSyntax(Start);
internal abstract record ExpressionSyntax(int Position);
internal sealed record LiteralSyntax(Token Token) : ExpressionSyntax(Token.Position);
internal sealed record ParenthesizedSyntax(int Start, ExpressionSyntax Expression) : ExpressionSyntax(Start);
internal sealed record NameExpressionSyntax(Token Name) : ExpressionSyntax(Name.Position);
internal sealed record CallSyntax(ExpressionSyntax Target, IReadOnlyList<ExpressionSyntax> Arguments) : ExpressionSyntax(Target.Position);
internal sealed record UnarySyntax(Token Operator, ExpressionSyntax Operand) : ExpressionSyntax(Operator.Position);
internal sealed record BinarySyntax(ExpressionSyntax Left, Token Operator, ExpressionSyntax Right) : ExpressionSyntax(Operator.Position);
internal sealed record CastSyntax(int Start, TypeSyntax Type, ExpressionSyntax Operand) : ExpressionSyntax(Start);
internal sealed record UncheckedSyntax(int Start, ExpressionSyntax Operand) : ExpressionSyntax(Start);
internal sealed record ConditionalArmSyntax(ExpressionSyntax Condition, ExpressionSyntax Value);
internal sealed record ConditionalSyntax(int Start, IReadOnlyList<ConditionalArmSyntax> Arms, ExpressionSyntax Else) : ExpressionSyntax(Start);

internal sealed record MemberSyntax(ExpressionSyntax Receiver, Token Name) : ExpressionSyntax(Receiver.Position);
internal sealed record NewSyntax(int Start, TypeSyntax Type, IReadOnlyList<ExpressionSyntax> Arguments) : ExpressionSyntax(Start);
internal sealed record UsingSyntax(BindingSyntax Binding, bool IsAsync = false) : StatementSyntax(Binding.Position);
internal sealed record RaiseSyntax(int Start, ExpressionSyntax? Exception) : StatementSyntax(Start);
internal sealed record CatchSyntax(int Position, Token? Name, TypeSyntax? Type, ExpressionSyntax? Filter, IReadOnlyList<StatementSyntax> Body);
internal sealed record TrySyntax(int Start, IReadOnlyList<StatementSyntax> Body, IReadOnlyList<CatchSyntax> Catches,
    IReadOnlyList<StatementSyntax>? Finally) : StatementSyntax(Start);

internal sealed record GenericNameSyntax(ExpressionSyntax Name, IReadOnlyList<TypeSyntax> Arguments) : ExpressionSyntax(Name.Position);
internal sealed record NewArraySyntax(int Start, TypeSyntax Element, IReadOnlyList<ExpressionSyntax> Lengths) : ExpressionSyntax(Start);
internal sealed record ArrayLiteralSyntax(int Start, IReadOnlyList<ExpressionSyntax> Elements) : ExpressionSyntax(Start);
internal sealed record IndexSyntax(ExpressionSyntax Receiver, IReadOnlyList<ExpressionSyntax> Indices) : ExpressionSyntax(Receiver.Position);
internal sealed record ForEachSyntax(int Start, Token Name, TypeSyntax Type, ExpressionSyntax Source, IReadOnlyList<StatementSyntax> Body) : StatementSyntax(Start);

internal sealed record LambdaSyntax(int Start, IReadOnlyList<ParameterSyntax> Parameters, TypeSyntax? Result, ExpressionSyntax? Expression, IReadOnlyList<StatementSyntax>? Body, IReadOnlyList<ContractSyntax>? Contracts = null, bool IsAsync = false) : ExpressionSyntax(Start);
internal sealed record AwaitSyntax(int Start, ExpressionSyntax Operand) : ExpressionSyntax(Start);

internal sealed record RecordSyntax(Token Name, bool IsPublic, IReadOnlyList<RecordFieldSyntax> Fields, IReadOnlyList<RoutineSyntax> Methods,
    IReadOnlyList<Token>? TypeParameters = null, IReadOnlyList<WhereSyntax>? Constraints = null, IReadOnlyList<TypeSyntax>? Interfaces = null, DocumentationComment? Documentation = null);
internal sealed record RecordFieldSyntax(Token Name, TypeSyntax Type, ExpressionSyntax? Default, bool IsPublic, DocumentationComment? Documentation = null);
internal sealed record NamedArgumentSyntax(Token Name, ExpressionSyntax Value) : ExpressionSyntax(Name.Position);
internal sealed record RecordUpdateSyntax(Token Name, ExpressionSyntax Value);
internal sealed record WithSyntax(ExpressionSyntax Receiver, IReadOnlyList<RecordUpdateSyntax> Updates) : ExpressionSyntax(Receiver.Position);
