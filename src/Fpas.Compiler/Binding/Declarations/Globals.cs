using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Declarations;

internal sealed record GlobalSymbol(SourceUnit Owner, GlobalSyntax Syntax, TypeSymbol Type)
{
    internal string Name => Syntax.Binding.Name.Text;
    internal ImportedField Field => new(Owner.ContainerType, Name, Type, true, !Syntax.Binding.Mutable);
}

internal sealed partial class SourceCatalog
{
    private void DeclareGlobals(ReferenceCatalog references)
    {
        foreach (var unit in Units)
            foreach (var syntax in unit.Syntax.Globals ?? [])
            {
                var name = syntax.Binding.Name;
                if (unit.HasType(name.Text) || unit.Syntax.Imports.Any(import => import.Alias.Text == name.Text) ||
                    unit.Syntax.IsProgram && name.Text == "args" || unit.Globals.ContainsKey(name.Text))
                    throw unit.Source.Error("FP3032", "A global name conflicts with a type, import, global, or program args.", name.Position);
                unit.Globals.Add(name.Text, new(unit, syntax, ResolveType(syntax.Binding.Type, unit, references)));
            }
    }
}
