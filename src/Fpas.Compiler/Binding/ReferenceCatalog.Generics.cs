using System.Reflection;
using System.Reflection.Metadata;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class ReferenceCatalog
{
    internal TypeSymbol Construct(ImportedType definition, IReadOnlyList<TypeSymbol> arguments, SourceText source, int position, bool validate = true)
    {
        var parameters = definition.Definition.GetGenericParameters();
        if (parameters.Count != arguments.Count || arguments.Any(argument => !Usable(argument) || argument.Kind is TypeKind.Void or TypeKind.Null))
            throw source.Error("FP3018", $"Invalid type arguments for '{definition.FullName}'.", position);
        var result = Symbol(definition) with { Arguments = new(arguments) };
        if (validate && !ConstraintsSatisfied(result, parameters, arguments, null))
            throw source.Error("FP3018", $"Type arguments violate constraints on '{definition.FullName}'.", position);
        return Canonical(result);
    }

    private static TypeSymbol Canonical(TypeSymbol type)
        => type.Definition?.FullName == "System.Nullable`1" && type.Arguments is { Count: 1 } args && args[0].IsValue && !args[0].IsNullable
            ? args[0] with { IsNullable = true } : type;

    internal TypeSymbol Substitute(TypeSymbol type, TypeSymbol owner, TypeList? methodArguments = null, bool keepMethodParameters = false)
    {
        if (keepMethodParameters && type.Kind == TypeKind.MethodParameter) return type;
        if (type.Kind is TypeKind.TypeParameter or TypeKind.MethodParameter)
        {
            var arguments = type.Kind == TypeKind.TypeParameter ? owner.TypeArguments : methodArguments ?? TypeList.Empty;
            if (type.ParameterIndex >= arguments.Count) return TypeSymbol.Unsupported;
            var argument = arguments[type.ParameterIndex];
            return type.IsNullable && argument.CanHaveNull ? argument with { IsNullable = true } : argument;
        }
        return Canonical(type with
        {
            Element = type.Element is null ? null : Substitute(type.Element, owner, methodArguments, keepMethodParameters),
            Arguments = type.Arguments is null ? null : new(type.Arguments.Select(argument => Substitute(argument, owner, methodArguments, keepMethodParameters)))
        });
    }

    internal static bool Usable(TypeSymbol type) => type.Kind is not (TypeKind.Unsupported or TypeKind.TypeParameter or TypeKind.MethodParameter) &&
        (type.Element is null || Usable(type.Element)) && (type.Arguments is null || type.Arguments.All(Usable));

    private IEnumerable<TypeSymbol> Parents(TypeSymbol type)
    {
        if (type.Scalar is { } scalar)
        {
            yield return scalar.Syntax.Category == Syntax.ScalarCategory.Enum ? Symbol(Required("System.Enum")) : type.IsValue ? Symbol(Required("System.ValueType")) : TypeSymbol.Object;
            if (scalar.Syntax.Category != Syntax.ScalarCategory.Enum) yield return Symbol(Required("System.IEquatable`1")) with { Arguments = new([type.NonNullable]) };
            yield break;
        }
        if (type.Kind == TypeKind.Async)
        {
            yield return Symbol(Required("System.Runtime.CompilerServices.IAsyncStateMachine"));
            yield return TypeSymbol.Object;
            yield break;
        }
        if (type.IsParameter)
        {
            foreach (var bound in type.Parameter!.Constraints) yield return bound.NonNullable;
            yield return type.IsValue ? Symbol(Required("System.ValueType")) : TypeSymbol.Object;
            yield break;
        }
        if (type.Kind is TypeKind.Record or TypeKind.Union)
        {
            if (type.Record is { } record)
                foreach (var contract in record.Interfaces)
                    yield return SubstituteSource(contract, record.GenericParameters, type.TypeArguments);
            yield return type.Record?.UnionCase is { } member ? member.Owner.Type with { Arguments = type.Arguments } : TypeSymbol.Object;
            yield return Symbol(Required("System.IEquatable`1")) with { Arguments = new([type.NonNullable]) };
            yield break;
        }
        if (type.Kind == TypeKind.Array)
        {
            yield return Symbol(Required("System.Array"));
            if (type.Rank == 1)
                foreach (var name in new[] { "IEnumerable", "ICollection", "IList", "IReadOnlyCollection", "IReadOnlyList" })
                    yield return Symbol(Required("System.Collections.Generic." + name + "`1")) with { Arguments = new([type.Element!]) };
            yield break;
        }
        var definition = Definition(type);
        yield return Decode(definition, definition.Definition.BaseType, type);
        foreach (var handle in definition.Definition.GetInterfaceImplementations())
            yield return Decode(definition, definition.Assembly.Reader.GetInterfaceImplementation(handle).Interface, type);
    }

    private bool SameAssignableDefinition(TypeSymbol from, TypeSymbol to)
    {
        if (from == to) return true;
        if (from.Definition is null || from.Definition != to.Definition || from.Arguments is null || to.Arguments is null || from.Arguments.Count != to.Arguments.Count) return false;
        foreach (var handle in from.Definition.Definition.GetGenericParameters())
        {
            var parameter = from.Definition.Assembly.Reader.GetGenericParameter(handle);
            var left = from.Arguments[parameter.Index]; var right = to.Arguments[parameter.Index];
            if (left == right) continue;
            if (!left.IsReference || !right.IsReference) return false;
            var variance = parameter.Attributes & GenericParameterAttributes.VarianceMask;
            if (variance == GenericParameterAttributes.Covariant && CanAssign(left, right)) continue;
            if (variance == GenericParameterAttributes.Contravariant && CanAssign(right, left)) continue;
            return false;
        }
        return true;
    }

    internal IReadOnlyList<TypeSymbol> EnumerableViews(TypeSymbol type)
    {
        var pending = new Stack<TypeSymbol>(); var seen = new HashSet<TypeSymbol>(); var result = new List<TypeSymbol>();
        pending.Push(type.NonNullable);
        while (pending.TryPop(out var current))
        {
            if (current == TypeSymbol.Unsupported || !seen.Add(current)) continue;
            if (current.Definition?.FullName == "System.Collections.Generic.IEnumerable`1" && current.Arguments is { Count: 1 }) result.Add(current);
            foreach (var parent in Parents(current)) pending.Push(parent);
        }
        return result;
    }
}
