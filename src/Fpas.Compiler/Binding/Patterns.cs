namespace Fpas.Compiler.Binding;

internal abstract record MatchPattern;
internal sealed record AnyMatch(LocalSymbol? Binding = null) : MatchPattern;
internal sealed record ConstantMatch(BoundExpression Expression, object? Value) : MatchPattern;
internal sealed record RangeMatch(long Lower, long Upper) : MatchPattern;
internal sealed record CaseMatch(UnionCaseInfo Case, IReadOnlyList<MatchPattern> Fields) : MatchPattern;
internal sealed record TypeMatch(TypeSymbol Target, LocalSymbol Binding) : MatchPattern;
