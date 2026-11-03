using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed class UnionSymbol(SourceUnit owner, UnionSyntax syntax)
{
    internal SourceUnit Owner { get; } = owner;
    internal UnionSyntax Syntax { get; } = syntax;
    internal IReadOnlyList<BoundAttribute> Attributes { get; set; } = [];
    internal string Name => Syntax.Name.Text;
    internal bool IsPublic => Syntax.IsPublic;
    internal IReadOnlyList<GenericParameterSymbol> GenericParameters { get; set; } = [];
    internal TypeSymbol Type => new(TypeKind.Union, Union: this, Arguments: GenericParameters.Count == 0 ? null : new(GenericParameters.Select(parameter => parameter.Type)));
    internal List<UnionCaseSymbol> Cases { get; } = [];
    internal List<RoutineSymbol> Methods { get; } = [];
    internal Dictionary<RoutineSymbol, BoundRoutine> Bodies { get; } = [];
    internal RoutineSymbol Method(GeneratedMethodKind kind) => Methods.Single(method => method.Generated == kind);
    internal ImportedMethod Import(RoutineSymbol method) => ReferenceCatalog.SourceMethod(method, Type);
}

internal sealed class UnionCaseSymbol(UnionSymbol owner, UnionCaseSyntax syntax, int tag)
{
    internal UnionSymbol Owner { get; } = owner;
    internal UnionCaseSyntax Syntax { get; } = syntax;
    internal int Tag { get; } = tag;
    internal RecordSymbol Representation { get; } = new(owner.Owner, new(new(TokenKind.Identifier, syntax.Name.Text + "Case", syntax.Name.Position, syntax.Name.Length),
        owner.IsPublic, syntax.Fields.Select(field => new RecordFieldSyntax(field.Name, field.Type, null, true)).ToArray(), []));
    internal RoutineSymbol Factory { get; set; } = null!;
}

internal sealed record UnionPayload(string Name, TypeSymbol Type, ImportedMethod Getter);
internal sealed record UnionCaseInfo(string Name, int Tag, TypeSymbol Type, ImportedMethod Factory, IReadOnlyList<UnionPayload> Fields);
internal sealed record UnionInfo(TypeSymbol Type, IReadOnlyList<UnionCaseInfo> Cases);
internal sealed record UnionImport(UnionSymbol Union) : ImportTarget;
internal sealed record BoundTypeTest(BoundExpression Value, TypeSymbol Target) : BoundExpression(TypeSymbol.Boolean);
internal sealed record BoundFailure(TypeSymbol Result, string Message) : BoundExpression(Result);
