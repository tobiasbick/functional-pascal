using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Bound;
using System.Reflection;

namespace Fpas.Compiler.Binding.Types;

internal enum InferenceBound { Exact, Lower, Upper }

internal sealed class TypeInference(ReferenceCatalog references, IReadOnlyList<TypeSymbol> variables)
{
    private readonly List<(TypeSymbol Type, InferenceBound Kind)>[] bounds = variables.Select(_ => new List<(TypeSymbol, InferenceBound)>()).ToArray();
    private readonly bool[] nullBounds = new bool[variables.Count];
    private bool ambiguous;

    private int Variable(TypeSymbol type)
    {
        for (var index = 0; index < variables.Count; index++)
            if (type.IsParameter && type.Parameter == variables[index].Parameter ||
                type.Kind == TypeKind.MethodParameter && variables[index].Kind == type.Kind && type.ParameterIndex == variables[index].ParameterIndex) return index;
        return -1;
    }

    internal bool ContainsVariable(TypeSymbol type) => Variable(type) >= 0 || type.Element is not null && ContainsVariable(type.Element) || type.TypeArguments.Any(ContainsVariable);

    internal void Add(TypeSymbol formal, TypeSymbol actual, InferenceBound kind = InferenceBound.Lower)
    {
        if (actual.Kind == TypeKind.Null)
        {
            var variable = Variable(formal);
            if (variable >= 0 && !formal.IsNullable) nullBounds[variable] = true;
            return;
        }
        if (actual.Kind is TypeKind.Void or TypeKind.Unsupported) return;
        if (formal.NullableValue)
        {
            if (actual.NullableValue) Add(formal.NonNullable, actual.NonNullable, kind);
            else if (actual.IsValue) Add(formal.NonNullable, actual, kind);
            return;
        }
        if (formal.Definition?.FullName == "System.Nullable`1" && formal.TypeArguments.Count == 1)
        {
            if (actual.NullableValue) Add(formal.TypeArguments[0], actual.NonNullable, kind);
            else if (actual.IsValue) Add(formal.TypeArguments[0], actual, kind);
            return;
        }
        var slot = Variable(formal);
        if (slot >= 0)
        {
            if (formal.IsNullable && actual.CanHaveNull) actual = actual with { IsNullable = false };
            bounds[slot].Add((actual with { NullValidated = false }, kind));
            return;
        }
        if (formal.Kind == TypeKind.Array && actual.Kind == TypeKind.Array && formal.Rank == actual.Rank)
        {
            Add(formal.Element!, actual.Element!, actual.Element!.IsReference ? kind : InferenceBound.Exact);
            return;
        }
        if (formal.TypeArguments.Count == 0 || !ContainsVariable(formal)) return;
        var views = references.InferenceViews(actual).Where(view => SameDefinition(formal, view) && view.TypeArguments.Count == formal.TypeArguments.Count).Distinct().ToArray();
        if (views.Length > 1) { ambiguous = true; return; }
        if (views.Length == 0) return;
        var match = views[0];
        var variance = formal.Definition?.Definition.GetGenericParameters().Select(handle => formal.Definition.Assembly.Reader.GetGenericParameter(handle).Attributes & GenericParameterAttributes.VarianceMask).ToArray();
        for (var index = 0; index < formal.TypeArguments.Count; index++)
        {
            var nested = variance?[index] switch
            {
                GenericParameterAttributes.Covariant => kind,
                GenericParameterAttributes.Contravariant => Reverse(kind),
                _ => InferenceBound.Exact
            };
            Add(formal.TypeArguments[index], match.TypeArguments[index], nested);
        }
    }

    internal void Add(TypeSymbol formal, BoundExpression value)
    {
        if (value is BoundMethodGroup) return;
        if (value is BoundArrayLiteral array && formal.Kind == TypeKind.Array && formal.Rank == 1)
        {
            foreach (var element in array.Elements) Add(formal.Element!, element);
            return;
        }
        if (value is BoundDelegate callable && references.DelegateSignature(formal, allowOpen: true) is { } invoke)
        {
            var (parameters, result) = callable switch
            {
                BoundLambda lambda => (lambda.Closure.Body.Symbol.Parameters.Select(parameter => parameter.Type).ToArray(), lambda.Closure.Body.Symbol.Result),
                BoundMethodDelegate method => (method.Method.Parameters.ToArray(), method.Method.Result),
                _ => throw new InvalidOperationException("Unknown callable inference source.")
            };
            if (parameters.Length != invoke.Parameters.Count) return;
            for (var index = 0; index < parameters.Length; index++) Add(invoke.Parameters[index], parameters[index], InferenceBound.Upper);
            Add(invoke.Result, result);
        }
        else Add(formal, value.Type);
    }

    internal TypeList? Fix()
    {
        if (ambiguous) return null;
        var result = new List<TypeSymbol>();
        for (var index = 0; index < bounds.Length; index++)
        {
            var set = bounds[index];
            var candidates = set.Select(bound => nullBounds[index] && bound.Type.IsReference ? bound.Type with { IsNullable = true } : bound.Type).Distinct().Where(candidate => set.All(bound => bound.Kind switch
            {
                InferenceBound.Exact => candidate == bound.Type,
                InferenceBound.Lower => references.CanAssign(bound.Type, candidate),
                _ => references.CanAssign(candidate, bound.Type)
            })).ToArray();
            var best = candidates.Where(candidate => candidates.All(other => references.CanAssign(other, candidate))).ToArray();
            if (best.Length != 1) return null;
            result.Add(best[0]);
        }
        return new(result);
    }

    private static InferenceBound Reverse(InferenceBound kind) => kind == InferenceBound.Exact ? kind : kind == InferenceBound.Lower ? InferenceBound.Upper : InferenceBound.Lower;

    private static bool SameDefinition(TypeSymbol left, TypeSymbol right)
        => left.Kind == right.Kind && (left.Interface is not null ? left.Interface == right.Interface : left.Definition is not null ? left.Definition == right.Definition : left.Record is not null ? left.Record == right.Record : left.Variant is not null && left.Variant == right.Variant);
}
