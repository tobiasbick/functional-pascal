using System.Reflection;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

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
        method.Generated is GeneratedMethodKind.EqualsTyped or GeneratedMethodKind.EqualsObject or GeneratedMethodKind.Hash or GeneratedMethodKind.Tag,
        SourceMethod: method, ParameterInfo: method.Parameters);

    internal bool ValidSourceArguments(RoutineSymbol method, TypeList? arguments, TypeSymbol? owner = null)
        => ValidSourceArguments(method.GenericParameters, arguments ?? TypeList.Empty, owner);

    private bool ValidSourceArguments(IReadOnlyList<GenericParameterSymbol> parameters, IReadOnlyList<TypeSymbol> supplied, TypeSymbol? owner = null)
    {
        if (parameters.Count != supplied.Count) return false;
        foreach (var parameter in parameters)
        {
            var argument = supplied[parameter.Index];
            if (!Usable(argument) || argument.Kind is TypeKind.Void or TypeKind.Null) return false;
            if (parameter.IsReference && !argument.IsReference || parameter.Primary == "struct" && (!argument.IsValue || argument.NullableValue) ||
                parameter.IsNotNull && argument.NeedsNullCheck || parameter.HasConstructor && !HasDefaultConstructor(argument)) return false;
            foreach (var bound in parameter.Constraints)
            {
                var required = SubstituteSource(bound, parameters, supplied);
                if (owner is not null) required = SubstituteSource(required, SourceParameters(owner), owner.TypeArguments);
                if (!Derives(argument, required) || required.IsReference && !required.IsNullable && argument.NeedsNullCheck) return false;
            }
        }
        return true;
    }

    private bool HasDefaultConstructor(TypeSymbol type)
    {
        if (type.IsValue) return true;
        if (type.IsParameter) return type.Parameter!.HasConstructor;
        if (type.Record is { } record) return record.UnionCase is null && record.Fields.Count == 0;
        if (type.Kind == TypeKind.Union) return false;
        return !Definition(type).Definition.Attributes.HasFlag(TypeAttributes.Abstract) && Methods(type, ".ctor", false).Any(constructor => constructor.Parameters.Count == 0);
    }

    internal void ValidateGenericType(TypeSymbol type, SourceText source, int position)
    {
        if (type.IsParameter && type.IsNullable && !type.IsReference && !type.IsValue)
            throw source.Error("FP3026", "T? requires a reference or struct constraint.", position);
        if (type.Element is not null) ValidateGenericType(type.Element, source, position);
        foreach (var argument in type.TypeArguments) if (argument != type) ValidateGenericType(argument, source, position);
        if (type.Definition is not null && !ConstraintsSatisfied(type, type.Definition.Definition.GetGenericParameters(), type.TypeArguments, null))
            throw source.Error("FP3018", "A constructed constraint type violates CLR generic constraints.", position);
        if (type.Kind is TypeKind.Record or TypeKind.Union && !ValidSourceArguments(SourceParameters(type), type.TypeArguments))
            throw source.Error("FP3018", "A constructed source type violates generic constraints.", position);
    }

    internal bool HasStaticAbstractConstraint(TypeSymbol type)
        => Hierarchy(type).Where(candidate => candidate.Kind is not (TypeKind.Record or TypeKind.Union or TypeKind.Parameter)).Any(candidate =>
            Definition(candidate).Definition.GetMethods().Any(handle =>
            {
                var method = Definition(candidate).Assembly.Reader.GetMethodDefinition(handle);
                return method.Attributes.HasFlag(MethodAttributes.Static) && method.Attributes.HasFlag(MethodAttributes.Abstract);
            }));
}
