using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;

namespace Fpas.Compiler.Emission.Metadata;

internal sealed partial class AssemblyEmitter
{
    private IReadOnlyList<GenericParameterSymbol>? genericTypeContext;
    private readonly List<(EntityHandle Owner, IReadOnlyList<GenericParameterSymbol> Parameters, IReadOnlyList<GenericParameterSymbol>? Context)> genericOwners = [];

    private void RegisterGenerics(EntityHandle owner, IReadOnlyList<GenericParameterSymbol> parameters, IReadOnlyList<GenericParameterSymbol>? context = null)
    {
        if (parameters.Count != 0) genericOwners.Add((owner, parameters, context));
    }

    private void EmitGenericParameters(AssemblyIdentity runtime)
    {
        // GenericParam rows are sorted by the TypeOrMethodDef coded owner index.
        foreach (var entry in genericOwners.OrderBy(entry => MetadataTokens.GetRowNumber(entry.Owner) * 2 + (entry.Owner.Kind == HandleKind.MethodDefinition ? 1 : 0)))
        {
            genericTypeContext = entry.Context;
            for (var index = 0; index < entry.Parameters.Count; index++)
            {
                var parameter = entry.Parameters[index];
                var attributes = GenericParameterAttributes.None;
                if (parameter.Primary is "class" or "class?") attributes |= GenericParameterAttributes.ReferenceTypeConstraint;
                if (parameter.Primary == "struct") attributes |= GenericParameterAttributes.NotNullableValueTypeConstraint | GenericParameterAttributes.DefaultConstructorConstraint;
                if (parameter.HasConstructor) attributes |= GenericParameterAttributes.DefaultConstructorConstraint;
                var handle = metadata.AddGenericParameter(entry.Owner, attributes, metadata.GetOrAddString(parameter.Name), index);
                AddNullableFlags(handle, runtime, [parameter.Primary == "struct" ? (byte)0 : parameter.IsNotNull ? (byte)1 : (byte)2]);
                if (parameter.Primary == "struct") metadata.AddGenericParameterConstraint(handle, TypeHandle(catalog.Symbol(catalog.Required("System.ValueType"))));
                foreach (var bound in parameter.Constraints)
                {
                    var constraint = metadata.AddGenericParameterConstraint(handle, TypeHandle(bound));
                    if (NullableFlags(bound).Contains((byte)2)) AddNullableAnnotation(constraint, runtime, bound);
                }
            }
        }
        genericTypeContext = null;
    }

}
