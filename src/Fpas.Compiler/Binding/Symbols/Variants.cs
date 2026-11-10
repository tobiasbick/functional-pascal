using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Symbols;

internal sealed class VariantSymbol(SourceUnit owner, VariantSyntax syntax)
{
    internal SourceUnit Owner { get; } = owner;
    internal VariantSyntax Syntax { get; } = syntax;
    internal IReadOnlyList<BoundAttribute> Attributes { get; set; } = [];
    internal string Name => Syntax.Name.Text;
    internal bool IsPublic => Syntax.IsPublic;
    internal IReadOnlyList<GenericParameterSymbol> GenericParameters { get; set; } = [];
    internal TypeSymbol Type => new(TypeKind.Variant, Variant: this, Arguments: GenericParameters.Count == 0 ? null : new(GenericParameters.Select(parameter => parameter.Type)));
    internal List<VariantCaseSymbol> Cases { get; } = [];
    internal List<RoutineSymbol> Methods { get; } = [];
    internal Dictionary<RoutineSymbol, BoundRoutine> Bodies { get; } = [];
    internal RoutineSymbol Method(GeneratedMethodKind kind) => Methods.Single(method => method.Generated == kind);
    internal ImportedMethod Import(RoutineSymbol method) => ReferenceCatalog.SourceMethod(method, Type);
}

internal sealed class VariantCaseSymbol(VariantSymbol owner, VariantCaseSyntax syntax, int tag)
{
    internal VariantSymbol Owner { get; } = owner;
    internal VariantCaseSyntax Syntax { get; } = syntax;
    internal int Tag { get; } = tag;
    internal RecordSymbol Representation { get; } = new(owner.Owner, new(new(TokenKind.Identifier, syntax.Name.Text + "Case", syntax.Name.Position, syntax.Name.Length),
        owner.IsPublic, syntax.Fields.Select(field => new RecordFieldSyntax(field.Name, field.Type, null, true)).ToArray(), []));
    internal RoutineSymbol Factory { get; set; } = null!;
}

internal sealed record VariantPayload(string Name, TypeSymbol Type, ImportedMethod Getter);
internal sealed record VariantCaseInfo(string Name, int Tag, TypeSymbol Type, ImportedMethod Factory, IReadOnlyList<VariantPayload> Fields);
internal sealed record VariantInfo(TypeSymbol Type, IReadOnlyList<VariantCaseInfo> Cases);
