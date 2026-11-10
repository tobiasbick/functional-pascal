using Fpas.Compiler.Binding.References;
namespace Fpas.Compiler.Binding.Declarations;

internal sealed partial class SourceCatalog
{
    private void DeclareTypeGenerics(ReferenceCatalog references)
    {
        foreach (var unit in Units)
        {
            foreach (var variant in unit.Variants.Values)
            {
                variant.GenericParameters = CreateTypeParameters(unit, variant.Syntax.TypeParameters, []);
                foreach (var member in variant.Cases) member.Representation.GenericParameters = variant.GenericParameters;
            }
            foreach (var record in unit.Records.Values)
                record.GenericParameters = CreateTypeParameters(unit, record.Syntax.TypeParameters, []);
            foreach (var contract in unit.Interfaces.Values)
                contract.GenericParameters = CreateTypeParameters(unit, contract.Syntax.TypeParameters, []);
        }
        foreach (var unit in Units)
        {
            foreach (var variant in unit.Variants.Values)
                BindTypeParameterConstraints(unit, variant.Syntax.Name, variant.GenericParameters, variant.Syntax.Constraints, references, validate: false);
            foreach (var record in unit.Records.Values)
                BindTypeParameterConstraints(unit, record.Syntax.Name, record.GenericParameters, record.Syntax.Constraints, references, validate: false);
            foreach (var contract in unit.Interfaces.Values)
                BindTypeParameterConstraints(unit, contract.Syntax.Name, contract.GenericParameters, contract.Syntax.Constraints, references, validate: false);
        }
    }

    private void ValidateTypeGenerics(ReferenceCatalog references)
    {
        foreach (var unit in Units)
        {
            foreach (var variant in unit.Variants.Values) ValidateTypeParameters(unit, variant.Syntax.Name, variant.GenericParameters, references);
            foreach (var record in unit.Records.Values) ValidateTypeParameters(unit, record.Syntax.Name, record.GenericParameters, references);
            foreach (var contract in unit.Interfaces.Values) ValidateTypeParameters(unit, contract.Syntax.Name, contract.GenericParameters, references);
        }
    }
}
