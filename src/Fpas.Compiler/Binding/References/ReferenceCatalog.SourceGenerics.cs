// [Documentation](../../../../docs/pascal/language/types/generics.md)
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using System.Reflection;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.References;

internal sealed partial class ReferenceCatalog
{
    internal TypeSymbol SubstituteSource(TypeSymbol type, IReadOnlyList<GenericParameterSymbol> parameters, IReadOnlyList<TypeSymbol> arguments)
    {
        if (type.IsParameter)
        {
            var index = parameters.ToList().IndexOf(type.Parameter!);
            if (index < 0) return type;
            var argument = arguments[index];
            return type.IsNullable && (type.IsValue || argument.CanHaveNull) ? argument with { IsNullable = true } : argument;
        }
        return Canonical(type with
        {
            Element = type.Element is null ? null : SubstituteSource(type.Element, parameters, arguments),
            Arguments = type.Arguments is null ? null : new(type.Arguments.Select(argument => SubstituteSource(argument, parameters, arguments)))
        });
    }

    internal ImportedMethod InstantiateSource(RoutineSymbol method, TypeSymbol owner, TypeList? arguments)
    {
        var supplied = arguments ?? TypeList.Empty;
        TypeSymbol Apply(TypeSymbol type) => SubstituteSource(SubstituteSource(type, SourceParameters(owner), owner.TypeArguments), method.GenericParameters, supplied);
        return SourceMethod(method, owner) with { Result = Apply(method.Result), Parameters = method.Parameters.Select(parameter => Apply(parameter.Type)).ToArray(), MethodArguments = arguments,
            ParameterInfo = method.Parameters.Select(parameter => parameter with { Type = Apply(parameter.Type) }).ToArray() };
    }

    internal static ImportedMethod SourceMethod(RoutineSymbol method, TypeSymbol owner) => new(owner, method.Name, method.Result,
        method.Parameters.Select(parameter => parameter.Type).ToArray(), !method.IsInstance,
        method.InterfaceOwner is not null || method.Generated is GeneratedMethodKind.EqualsTyped or GeneratedMethodKind.EqualsObject or GeneratedMethodKind.Hash or GeneratedMethodKind.Tag,
        SourceMethod: method, ParameterInfo: method.Parameters);

    private GenericConstraintFailure? SourceConstraintFailure(IReadOnlyList<GenericParameterSymbol> parameters, IReadOnlyList<TypeSymbol> supplied, TypeSymbol? owner = null)
    {
        if (parameters.Count != supplied.Count) return new("type arguments", TypeSymbol.Unsupported, $"{parameters.Count} type arguments");
        foreach (var parameter in parameters)
        {
            var argument = supplied[parameter.Index];
            GenericConstraintFailure Fail(string requirement) => new(parameter.Name, argument, requirement);
            if (!Usable(argument) || argument.Kind is TypeKind.Void or TypeKind.Null) return Fail("supported type");
            if (parameter.IsReference && !argument.IsReference) return Fail("class");
            if (parameter.Primary == "struct" && (!argument.IsValue || argument.NullableValue)) return Fail("struct");
            if (parameter.IsNotNull && argument.NeedsNullCheck) return Fail("notnull");
            if (parameter.HasConstructor && !HasDefaultConstructor(argument)) return Fail("new()");
            foreach (var bound in parameter.Constraints)
            {
                var required = SubstituteSource(bound, parameters, supplied);
                if (owner is not null) required = SubstituteSource(required, SourceParameters(owner), owner.TypeArguments);
                if (!Derives(argument, required) || required.IsReference && !required.IsNullable && argument.NeedsNullCheck)
                    return Fail(TypeFacts.Display(required));
            }
        }
        return null;
    }

    private bool HasDefaultConstructor(TypeSymbol type)
    {
        if (type.IsValue) return true;
        if (type.Kind == TypeKind.Scalar) return false;
        if (type.IsParameter) return type.Parameter!.HasConstructor;
        if (type.Record is { } record) return record.VariantCase is null && record.Fields.Count == 0;
        if (type.Kind == TypeKind.Variant) return false;
        if (type.IsInterface) return false;
        return !Definition(type).Definition.Attributes.HasFlag(TypeAttributes.Abstract) && Methods(type, ".ctor", false).Any(constructor => constructor.Parameters.Count == 0);
    }

    internal void ValidateGenericType(TypeSymbol type, SourceText source, int position)
    {
        if (type.IsParameter && type.IsNullable && !type.IsReference && !type.IsValue)
            throw source.Error("FP3026", "T? requires a reference or struct constraint.", position);
        if (type.Element is not null) ValidateGenericType(type.Element, source, position);
        foreach (var argument in type.TypeArguments) if (argument != type) ValidateGenericType(argument, source, position);
        if (type.Definition is not null && ConstraintFailure(type, type.Definition.Definition.GetGenericParameters(), type.TypeArguments, null) is { } clr)
            throw ConstraintError(source, position, clr);
        if (type.Kind is TypeKind.Record or TypeKind.Variant or TypeKind.Interface && SourceConstraintFailure(SourceParameters(type), type.TypeArguments) is { } sourceType)
            throw ConstraintError(source, position, sourceType);
    }

}
