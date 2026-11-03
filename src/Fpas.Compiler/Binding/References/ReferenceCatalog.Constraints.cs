using Fpas.Compiler.Binding.Types;
using System.Reflection;
using System.Reflection.Metadata;

namespace Fpas.Compiler.Binding.References;

internal sealed partial class ReferenceCatalog
{
    internal bool ValidMethodArguments(ImportedMethod method)
    {
        if (method.SourceMethod is { } source) return ValidSourceArguments(source, method.MethodArguments, method.Owner);
        if (method.MethodArguments is null) return true;
        var definition = Definition(method.Owner);
        var declaration = definition.Assembly.Reader.GetMethodDefinition(method.DefinitionHandle);
        return ConstraintsSatisfied(method.Owner, declaration.GetGenericParameters(), method.MethodArguments, method.MethodArguments, declaration);
    }

    private bool ConstraintsSatisfied(TypeSymbol owner, GenericParameterHandleCollection parameters, IReadOnlyList<TypeSymbol> arguments,
        TypeList? methodArguments, MethodDefinition? method = null)
    {
        var definition = Definition(owner);
        foreach (var handle in parameters)
        {
            var parameter = definition.Assembly.Reader.GetGenericParameter(handle);
            var argument = arguments[parameter.Index];
            if (!Usable(argument) || argument.Kind is TypeKind.Void or TypeKind.Null) return false;
            var flags = parameter.Attributes;
            if (flags.HasFlag(GenericParameterAttributes.ReferenceTypeConstraint) && !argument.IsReference ||
                flags.HasFlag(GenericParameterAttributes.NotNullableValueTypeConstraint) && (!argument.IsValue || argument.NullableValue) ||
                NullabilityMetadata.GenericNotNull(definition, parameter, method) && argument.NeedsNullCheck) return false;
            if (MetadataAttributes.Has(definition.Assembly.Reader, parameter.GetCustomAttributes(), "System.Runtime.CompilerServices.IsUnmanagedAttribute")) return false;
            if (flags.HasFlag(GenericParameterAttributes.DefaultConstructorConstraint) && !HasDefaultConstructor(argument)) return false;
            foreach (var constraintHandle in parameter.GetConstraints())
            {
                var constraint = definition.Assembly.Reader.GetGenericParameterConstraint(constraintHandle);
                var required = Decode(definition, constraint.Type, owner, methodArguments);
                if (required == TypeSymbol.Unsupported || !Derives(argument, required) || required.IsReference && !required.IsNullable && argument.NeedsNullCheck) return false;
            }
        }
        return true;
    }
}
