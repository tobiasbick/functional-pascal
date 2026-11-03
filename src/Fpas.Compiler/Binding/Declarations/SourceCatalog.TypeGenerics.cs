using Fpas.Compiler.Binding.References;
namespace Fpas.Compiler.Binding.Declarations;

internal sealed partial class SourceCatalog
{
    private void DeclareTypeGenerics(ReferenceCatalog references)
    {
        foreach (var unit in Units)
        {
            foreach (var union in unit.Unions.Values)
            {
                union.GenericParameters = CreateTypeParameters(unit, union.Syntax.TypeParameters, []);
                foreach (var member in union.Cases) member.Representation.GenericParameters = union.GenericParameters;
            }
            foreach (var record in unit.Records.Values)
                record.GenericParameters = CreateTypeParameters(unit, record.Syntax.TypeParameters, []);
            foreach (var contract in unit.Interfaces.Values)
                contract.GenericParameters = CreateTypeParameters(unit, contract.Syntax.TypeParameters, []);
        }
        foreach (var unit in Units)
        {
            foreach (var union in unit.Unions.Values)
                BindTypeParameterConstraints(unit, union.Syntax.Name, union.GenericParameters, union.Syntax.Constraints, references, validate: false);
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
            foreach (var union in unit.Unions.Values) ValidateTypeParameters(unit, union.Syntax.Name, union.GenericParameters, references);
            foreach (var record in unit.Records.Values) ValidateTypeParameters(unit, record.Syntax.Name, record.GenericParameters, references);
            foreach (var contract in unit.Interfaces.Values) ValidateTypeParameters(unit, contract.Syntax.Name, contract.GenericParameters, references);
        }
    }
}
