using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class Binder
{
    private TypeSymbol PatternOwner(ExpressionSyntax path, TypeSymbol selector)
    {
        if (path is not MemberSyntax member) return TypeSymbol.Unsupported;
        var owner = selector.NonNullable;
        if (owner.Record?.UnionCase is { } sourceCase) owner = sourceCase.Owner.Type with { Arguments = owner.Arguments };
        else if (owner.Definition?.DeclaringType is { } parent && references.UnionInfo(references.Symbol(parent) with { Arguments = owner.Arguments }) is not null)
            owner = references.Symbol(parent) with { Arguments = owner.Arguments };
        var selected = ImportedValueType(StaticTarget(member.Receiver, owner.TypeArguments.Count), path.Position, allowOpen: true);
        if (ReferenceCatalog.SourceParameters(selected).Count != 0)
        {
            if (selected.Union != owner.Union) throw source.Error("FP3024", "The generic union pattern must name the selector's union.", path.Position);
            return selected with { Arguments = owner.Arguments };
        }
        if (selected.Definition?.Definition.GetGenericParameters().Count > 0 && selected.Arguments is null)
        {
            if (selected.Definition != owner.Definition) throw source.Error("FP3024", "The generic union pattern must name the selector's union.", path.Position);
            return selected with { Arguments = owner.Arguments };
        }
        return selected;
    }
}
