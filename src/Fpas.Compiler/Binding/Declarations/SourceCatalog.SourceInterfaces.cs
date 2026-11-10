using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;

namespace Fpas.Compiler.Binding.Declarations;

internal sealed partial class SourceCatalog
{
    private void DeclareInterfaceNames()
    {
        foreach (var unit in Units)
            foreach (var syntax in unit.Syntax.Interfaces ?? [])
            {
                if (TypeFacts.PredefinedName(syntax.Name.Text) || unit.HasType(syntax.Name.Text) ||
                    unit.Syntax.Records.Any(type => type.Name.Text == syntax.Name.Text) || unit.Syntax.Variants.Any(type => type.Name.Text == syntax.Name.Text) ||
                    unit.Syntax.Imports.Any(import => import.Alias.Text == syntax.Name.Text))
                    throw unit.Source.Error("FP3029", $"Duplicate or conflicting interface '{syntax.Name.Text}'.", syntax.Name.Position);
                unit.Interfaces.Add(syntax.Name.Text, new(unit, syntax));
            }
    }

    private void DeclareInterfaceBases(ReferenceCatalog references)
    {
        var contracts = Units.SelectMany(unit => unit.Interfaces.Values).ToArray();
        foreach (var contract in contracts)
            foreach (var syntax in contract.Syntax.Bases)
            {
                var type = ResolveType(syntax, contract.Owner, references,
                    contract.GenericParameters.ToDictionary(parameter => parameter.Name, StringComparer.Ordinal), validate: false);
                if (!type.IsInterface || type.IsNullable || contract.Bases.Any(other => other.Erased == type.Erased))
                    throw contract.Owner.Source.Error("FP3029", "extends requires distinct non-nullable interface types.", syntax.Position);
                contract.Bases.Add(type);
            }
        var completed = new HashSet<InterfaceSymbol>();
        var pending = new HashSet<InterfaceSymbol>();
        void Visit(InterfaceSymbol contract)
        {
            if (completed.Contains(contract)) return;
            if (!pending.Add(contract)) throw contract.Owner.Source.Error("FP3029", "Interface inheritance cannot form a cycle.", contract.Syntax.Name.Position);
            foreach (var parent in contract.Bases)
                if (parent.Interface is { } source) Visit(source);
            pending.Remove(contract);
            completed.Add(contract);
        }
        foreach (var contract in contracts) Visit(contract);
    }

    private void DeclareInterfaceMembers(ReferenceCatalog references)
    {
        foreach (var contract in Units.SelectMany(unit => unit.Interfaces.Values))
        {
            var unit = contract.Owner;
            foreach (var parent in contract.Bases) references.ValidateGenericType(parent, unit.Source, contract.Syntax.Name.Position);
            var names = new HashSet<string>(StringComparer.Ordinal);
            var overloads = new Dictionary<string, List<RoutineSymbol>>(StringComparer.Ordinal);
            foreach (var property in contract.Syntax.Properties)
            {
                if (!names.Add(property.Name.Text) || !names.Add("get_" + property.Name.Text))
                    throw unit.Source.Error("FP3029", "Interface properties and accessors cannot collide.", property.Name.Position);
                var type = ResolveType(property.Type, unit, references, contract.GenericParameters.ToDictionary(parameter => parameter.Name, StringComparer.Ordinal));
                var getter = new RoutineSymbol("get_" + property.Name.Text, type, [], true, false, -1,
                    IsInstance: true, Generated: GeneratedMethodKind.Getter, InterfaceOwner: contract);
                contract.Methods.Add(getter);
                contract.Properties.Add(new(property, getter));
            }
            foreach (var syntax in contract.Syntax.Methods)
            {
                if (names.Contains(syntax.Name.Text) || syntax.Parameters.Any(parameter => parameter.Name.Text == "self"))
                    throw unit.Source.Error("FP3029", "Interface methods cannot collide with properties/accessors or use a self parameter.", syntax.Name.Position);
                var method = DeclareRoutine(unit, syntax, references, -1, enclosing: contract.GenericParameters) with { IsInstance = true, InterfaceOwner = contract };
                AddOverload(overloads, method, unit.Source, syntax.Name.Position);
                contract.Methods.Add(method);
            }
        }
    }
}
