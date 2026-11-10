using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Declarations;

internal sealed partial class SourceCatalog
{
    private void DeclareVariants(ReferenceCatalog references)
    {
        foreach (var unit in Units)
            foreach (var syntax in unit.Syntax.Variants)
            {
                if (TypeFacts.PredefinedName(syntax.Name.Text) || unit.Syntax.Imports.Any(import => import.Alias.Text == syntax.Name.Text) ||
                    !unit.Variants.TryAdd(syntax.Name.Text, new(unit, syntax)))
                    throw unit.Source.Error("FP3023", $"Duplicate or conflicting variant '{syntax.Name.Text}'.", syntax.Name.Position);
                var variant = unit.Variants[syntax.Name.Text];
                var names = new HashSet<string>(["Tag", "get_Tag", "Equals", "GetHashCode", "op_Equality", "op_Inequality", variant.Name], StringComparer.Ordinal);
                foreach (var member in syntax.Cases)
                {
                    if (!names.Add(member.Name.Text) || !names.Add(member.Name.Text + "Case") || member.Fields.Count == 0 && !names.Add("get_" + member.Name.Text))
                        throw unit.Source.Error("FP3023", "Variant cases collide with another case or generated member.", member.Name.Position);
                    var payloadNames = new HashSet<string>(["Tag", "get_Tag", "Equals", "GetHashCode", "op_Equality", "op_Inequality", member.Name.Text + "Case"], StringComparer.Ordinal);
                    foreach (var field in member.Fields)
                        if (!payloadNames.Add(field.Name.Text) || !payloadNames.Add("get_" + field.Name.Text))
                            throw unit.Source.Error("FP3023", "Variant payload names must be unique and cannot collide with generated members.", field.Name.Position);
                    var item = new VariantCaseSymbol(variant, member, variant.Cases.Count);
                    item.Representation.VariantCase = item;
                    variant.Cases.Add(item);
                }
                if (variant.Cases.Count == 0) throw unit.Source.Error("FP3023", "A variant requires at least one case.", syntax.Name.Position);
            }
        if (Units.Any(unit => unit.Variants.Count != 0)) references.Required("Fpas.Core.VariantAttribute");
    }

    private static void CompleteVariants(IEnumerable<SourceUnit> units)
    {
        foreach (var variant in units.SelectMany(unit => unit.Variants.Values))
        {
            RoutineSymbol Add(string name, TypeSymbol result, IEnumerable<(string Name, TypeSymbol Type)> parameters, bool instance, GeneratedMethodKind kind)
            {
                var method = new RoutineSymbol(name, result, parameters.Select((parameter, index) => new ParameterSymbol(parameter.Name, parameter.Type, index)).ToArray(),
                    kind != GeneratedMethodKind.Constructor, false, -1, instance, Generated: kind, VariantOwner: variant);
                variant.Methods.Add(method); return method;
            }
            Add(".ctor", TypeSymbol.Void, [], true, GeneratedMethodKind.Constructor);
            Add("get_Tag", TypeSymbol.Int32, [], true, GeneratedMethodKind.Tag);
            Add("Equals", TypeSymbol.Boolean, [("Other", variant.Type with { IsNullable = true })], true, GeneratedMethodKind.EqualsTyped);
            Add("Equals", TypeSymbol.Boolean, [("Other", TypeSymbol.Object with { IsNullable = true })], true, GeneratedMethodKind.EqualsObject);
            Add("GetHashCode", TypeSymbol.Int32, [], true, GeneratedMethodKind.Hash);
            Add("op_Equality", TypeSymbol.Boolean, [("Left", variant.Type with { IsNullable = true }), ("Right", variant.Type with { IsNullable = true })], false, GeneratedMethodKind.Equality);
            Add("op_Inequality", TypeSymbol.Boolean, [("Left", variant.Type with { IsNullable = true }), ("Right", variant.Type with { IsNullable = true })], false, GeneratedMethodKind.Inequality);
            foreach (var member in variant.Cases)
                member.Factory = Add(member.Syntax.Fields.Count == 0 ? "get_" + member.Syntax.Name.Text : member.Syntax.Name.Text, variant.Type,
                    member.Representation.Fields.Select(field => (field.Name, field.Type)), false, GeneratedMethodKind.Factory);
        }
    }
}
