// [Documentation](../../../../docs/pascal/language/pattern-matching/syntax.md)
using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private TypeSymbol PatternOwner(ExpressionSyntax path, TypeSymbol selector)
    {
        if (path is not MemberSyntax member) return TypeSymbol.Unsupported;
        var owner = selector.NonNullable;
        if (owner.Record?.VariantCase is { } sourceCase) owner = sourceCase.Owner.Type with { Arguments = owner.Arguments };
        else if (owner.Definition?.DeclaringType is { } parent && references.VariantInfo(references.Symbol(parent) with { Arguments = owner.Arguments }) is not null)
            owner = references.Symbol(parent) with { Arguments = owner.Arguments };
        var selected = ImportedValueType(StaticTarget(member.Receiver, owner.TypeArguments.Count), path.Position, allowOpen: true);
        if (ReferenceCatalog.SourceParameters(selected).Count != 0)
        {
            if (selected.Variant != owner.Variant) throw source.Error("FP3024", "The generic variant pattern must name the selector's variant.", path.Position);
            return selected with { Arguments = owner.Arguments };
        }
        if (selected.Definition?.Definition.GetGenericParameters().Count > 0 && selected.Arguments is null)
        {
            if (selected.Definition != owner.Definition) throw source.Error("FP3024", "The generic variant pattern must name the selector's variant.", path.Position);
            return selected with { Arguments = owner.Arguments };
        }
        return selected;
    }

    private string? CoverageCaseName(VariantCaseInfo item)
    {
        if (!IdentifierFacts.IsValid(item.Name)) return null;
        var owner = item.Factory.Owner;
        List<string[]> paths = [];
        if (owner.Variant is { } variant)
        {
            if (variant.Owner == unit) paths.Add([variant.Name]);
            foreach (var (alias, target) in imports)
                if (target is VariantImport direct && direct.Variant == variant) paths.Add([alias]);
                else if (target is SourceImport container && container.Unit == variant.Owner &&
                    !container.Unit.Globals.ContainsKey(variant.Name) && !container.Unit.Routines.ContainsKey(variant.Name))
                    paths.Add([alias, variant.Name]);
        }
        else if (owner.Definition is { } definition)
        {
            foreach (var (alias, target) in imports)
                if (MetadataPatternPath(alias, target, definition) is { } path) paths.Add(path);
        }
        var selected = paths.Where(path => !HasValue(path[0]) && !TypeParameters.ContainsKey(path[0]) && path.All(IdentifierFacts.IsValid))
            .OrderBy(path => path.Length).FirstOrDefault();
        return selected is null ? null : string.Join('.', selected.Append(item.Name).Select(IdentifierFacts.Spell));
    }

    private static string[]? MetadataPatternPath(string alias, ImportTarget target, ImportedType definition)
    {
        var names = new Stack<string>();
        for (ImportedType? current = definition; current is not null; current = current.DeclaringType)
        {
            if (target is ClrImport direct && direct.Type == current) return [alias, .. names];
            if (current != definition && current.Name.Contains('`')) return null;
            names.Push(current.Name.Split('`')[0]);
            if (current.DeclaringType is null && target is NamespaceImport ns)
            {
                if (current.Namespace == ns.Namespace) return [alias, .. names];
                if (current.Namespace.StartsWith(ns.Namespace + ".", StringComparison.Ordinal))
                    return [alias, .. current.Namespace[(ns.Namespace.Length + 1)..].Split('.'), .. names];
            }
        }
        return null;
    }
}
