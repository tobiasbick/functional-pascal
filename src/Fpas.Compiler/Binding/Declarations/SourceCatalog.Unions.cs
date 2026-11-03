using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Declarations;

internal sealed partial class SourceCatalog
{
    private void DeclareUnions(ReferenceCatalog references)
    {
        foreach (var unit in Units)
            foreach (var syntax in unit.Syntax.Unions)
            {
                if (TypeFacts.PredefinedName(syntax.Name.Text) || unit.Syntax.Imports.Any(import => import.Alias.Text == syntax.Name.Text) ||
                    !unit.Unions.TryAdd(syntax.Name.Text, new(unit, syntax)))
                    throw unit.Source.Error("FP3023", $"Duplicate or conflicting union '{syntax.Name.Text}'.", syntax.Name.Position);
                var union = unit.Unions[syntax.Name.Text];
                var names = new HashSet<string>(["Tag", "get_Tag", "Equals", "GetHashCode", "op_Equality", "op_Inequality", union.Name], StringComparer.Ordinal);
                foreach (var member in syntax.Cases)
                {
                    if (!names.Add(member.Name.Text) || !names.Add(member.Name.Text + "Case") || member.Fields.Count == 0 && !names.Add("get_" + member.Name.Text))
                        throw unit.Source.Error("FP3023", "Union cases collide with another case or generated member.", member.Name.Position);
                    var payloadNames = new HashSet<string>(["Tag", "get_Tag", "Equals", "GetHashCode", "op_Equality", "op_Inequality", member.Name.Text + "Case"], StringComparer.Ordinal);
                    foreach (var field in member.Fields)
                        if (!payloadNames.Add(field.Name.Text) || !payloadNames.Add("get_" + field.Name.Text))
                            throw unit.Source.Error("FP3023", "Union payload names must be unique and cannot collide with generated members.", field.Name.Position);
                    var item = new UnionCaseSymbol(union, member, union.Cases.Count);
                    item.Representation.UnionCase = item;
                    union.Cases.Add(item);
                }
                if (union.Cases.Count == 0) throw unit.Source.Error("FP3023", "A union requires at least one case.", syntax.Name.Position);
            }
        if (Units.Any(unit => unit.Unions.Count != 0)) references.Required("Fpas.Core.UnionAttribute");
    }

    private static void CompleteUnions(IEnumerable<SourceUnit> units)
    {
        foreach (var union in units.SelectMany(unit => unit.Unions.Values))
        {
            RoutineSymbol Add(string name, TypeSymbol result, IEnumerable<(string Name, TypeSymbol Type)> parameters, bool instance, GeneratedMethodKind kind)
            {
                var method = new RoutineSymbol(name, result, parameters.Select((parameter, index) => new ParameterSymbol(parameter.Name, parameter.Type, index)).ToArray(),
                    kind != GeneratedMethodKind.Constructor, false, -1, instance, Generated: kind, UnionOwner: union);
                union.Methods.Add(method); return method;
            }
            Add(".ctor", TypeSymbol.Void, [], true, GeneratedMethodKind.Constructor);
            Add("get_Tag", TypeSymbol.Int32, [], true, GeneratedMethodKind.Tag);
            Add("Equals", TypeSymbol.Boolean, [("Other", union.Type with { IsNullable = true })], true, GeneratedMethodKind.EqualsTyped);
            Add("Equals", TypeSymbol.Boolean, [("Other", TypeSymbol.Object with { IsNullable = true })], true, GeneratedMethodKind.EqualsObject);
            Add("GetHashCode", TypeSymbol.Int32, [], true, GeneratedMethodKind.Hash);
            Add("op_Equality", TypeSymbol.Boolean, [("Left", union.Type with { IsNullable = true }), ("Right", union.Type with { IsNullable = true })], false, GeneratedMethodKind.Equality);
            Add("op_Inequality", TypeSymbol.Boolean, [("Left", union.Type with { IsNullable = true }), ("Right", union.Type with { IsNullable = true })], false, GeneratedMethodKind.Inequality);
            foreach (var member in union.Cases)
                member.Factory = Add(member.Syntax.Fields.Count == 0 ? "get_" + member.Syntax.Name.Text : member.Syntax.Name.Text, union.Type,
                    member.Representation.Fields.Select(field => (field.Name, field.Type)), false, GeneratedMethodKind.Factory);
        }
    }
}
