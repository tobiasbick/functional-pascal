// [Documentation](../../../../docs/pascal/language/types/generics.md)
using Fpas.Compiler.Binding.Types;
using System.Reflection;
using System.Reflection.Metadata;

namespace Fpas.Compiler.Binding.References;

internal sealed partial class ReferenceCatalog
{
    internal readonly record struct GenericConstraintFailure(string Parameter, TypeSymbol Argument, string Requirement);

    internal bool ValidMethodArguments(ImportedMethod method) => MethodConstraintFailure(method) is null;

    internal GenericConstraintFailure? MethodConstraintFailure(ImportedMethod method)
    {
        if (method.SourceMethod is { } source)
            return SourceConstraintFailure(source.GenericParameters, method.MethodArguments ?? TypeList.Empty, method.Owner);
        if (method.MethodArguments is null) return null;
        var definition = Definition(method.Owner);
        var declaration = definition.Assembly.Reader.GetMethodDefinition(method.DefinitionHandle);
        return ConstraintFailure(method.Owner, declaration.GetGenericParameters(), method.MethodArguments, method.MethodArguments, declaration);
    }

    private GenericConstraintFailure? ConstraintFailure(TypeSymbol owner, GenericParameterHandleCollection parameters, IReadOnlyList<TypeSymbol> arguments,
        TypeList? methodArguments, MethodDefinition? method = null)
    {
        var definition = Definition(owner);
        var reader = definition.Assembly.Reader;
        foreach (var handle in parameters)
        {
            var parameter = reader.GetGenericParameter(handle);
            var argument = arguments[parameter.Index];
            GenericConstraintFailure Fail(string requirement) => new(reader.GetString(parameter.Name), argument, requirement);
            if (!Usable(argument) || argument.Kind is TypeKind.Void or TypeKind.Null) return Fail("supported type");
            var flags = parameter.Attributes;
            if (flags.HasFlag(GenericParameterAttributes.ReferenceTypeConstraint) && !argument.IsReference) return Fail("class");
            var unmanaged = MetadataAttributes.Has(reader, parameter.GetCustomAttributes(), "System.Runtime.CompilerServices.IsUnmanagedAttribute");
            if (flags.HasFlag(GenericParameterAttributes.NotNullableValueTypeConstraint) && (!argument.IsValue || argument.NullableValue))
                return Fail(unmanaged ? "unmanaged" : "struct");
            if (NullabilityMetadata.GenericNotNull(definition, parameter, method) && argument.NeedsNullCheck) return Fail("notnull");
            if (unmanaged && !Unmanaged(argument, [])) return Fail("unmanaged");
            if (flags.HasFlag(GenericParameterAttributes.DefaultConstructorConstraint) && !HasDefaultConstructor(argument)) return Fail("new()");
            foreach (var constraintHandle in parameter.GetConstraints())
            {
                var constraint = reader.GetGenericParameterConstraint(constraintHandle);
                var required = Decode(definition, constraint.Type, owner, methodArguments);
                if (required == TypeSymbol.Unsupported) return Fail("supported CLR constraint type");
                if (!Derives(argument, required) || required.IsReference && !required.IsNullable && argument.NeedsNullCheck)
                    return Fail(TypeFacts.Display(required));
            }
        }
        return null;
    }

    // Mirrors C#'s unmanaged constraint for concrete arguments: primitives, enums,
    // value domains over them, and imported structs whose instance fields are all
    // unmanaged. Nullable values, references and generic parameters are rejected.
    private bool Unmanaged(TypeSymbol type, HashSet<TypeSymbol> visiting)
    {
        if (type.IsNullable || type.IsParameter) return false;
        if (NumericFacts.NumericKind(type.Kind) || type.Kind == TypeKind.Boolean) return true;
        if (type.Kind == TypeKind.Scalar) return type.IsValue && Unmanaged(type.Scalar!.Base, visiting);
        if (type.Kind != TypeKind.Named || type.Definition is not { IsValueType: true } imported || type.IsSpan) return false;
        if (imported.IsEnum) return true;
        if (!visiting.Add(type)) return false;
        var reader = imported.Assembly.Reader;
        foreach (var handle in imported.Definition.GetFields())
        {
            var field = reader.GetFieldDefinition(handle);
            if (field.Attributes.HasFlag(FieldAttributes.Static)) continue;
            var fieldType = Substitute(field.DecodeSignature(new SignatureTypes(this, imported.Assembly, includePrivate: true), null), type, null);
            if (!Unmanaged(fieldType, visiting)) return false;
        }
        visiting.Remove(type);
        return true;
    }
}
