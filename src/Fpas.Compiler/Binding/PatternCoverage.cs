namespace Fpas.Compiler.Binding;

// A pattern matrix is specialized one constructor at a time. Payload columns
// preserve correlations, so nested cases are checked without enumerating values.
internal sealed class PatternCoverage(ReferenceCatalog references, Syntax.SourceText source, int position)
{
    private sealed record Constructor(UnionCaseInfo? Case, object? Constant, TypeSymbol? ActualType, long? Lower = null, long? Upper = null, bool IsNull = false, bool IsOther = false)
    {
        internal IReadOnlyList<TypeSymbol> Fields => Case?.Fields.Select(payload => payload.Type).ToArray() ?? [];
    }

    private int steps;

    internal bool Useful(IReadOnlyList<MatchPattern> covered, MatchPattern candidate, TypeSymbol type)
    {
        steps = 0;
        return Useful(covered.Select(pattern => new[] { pattern }).ToArray(), [candidate], [type]);
    }

    private bool Useful(IReadOnlyList<MatchPattern[]> matrix, MatchPattern[] candidate, TypeSymbol[] types)
    {
        if (++steps > 100_000) throw source.Error("FP3025", "Pattern coverage is too complex; simplify the nested patterns.", position);
        if (candidate.Length == 0) return matrix.Count == 0;
        if (matrix.Any(row => row.All(pattern => pattern is AnyMatch))) return false;
        if (candidate[0] is AnyMatch && matrix.All(row => row[0] is AnyMatch))
            return Useful(matrix.Select(row => row[1..]).ToArray(), candidate[1..], types[1..]);
        var heads = matrix.Select(row => row[0]).Append(candidate[0]).ToArray();
        foreach (var constructor in Constructors(types[0], heads))
        {
            var selected = Specialize(candidate[0], constructor);
            if (selected is null) continue;
            var rows = new List<MatchPattern[]>();
            foreach (var row in matrix)
                if (Specialize(row[0], constructor) is { } fields) rows.Add([.. fields, .. row.Skip(1)]);
            if (Useful(rows, [.. selected, .. candidate.Skip(1)], [.. constructor.Fields, .. types.Skip(1)])) return true;
        }
        return false;
    }

    private IEnumerable<Constructor> Constructors(TypeSymbol type, IReadOnlyList<MatchPattern> patterns)
    {
        if (type.IsNullable || type == TypeSymbol.Null) yield return new(null, null, null, IsNull: true);
        if (type == TypeSymbol.Null) yield break;
        type = type.NonNullable;
        if (references.UnionInfo(type) is { } union)
        {
            foreach (var member in union.Cases) yield return new(member, null, member.Type);
            yield break;
        }
        if (type == TypeSymbol.Boolean)
        {
            yield return new(null, false, type); yield return new(null, true, type); yield break;
        }
        if (type.Kind is TypeKind.Int32 or TypeKind.Char)
        {
            var minimum = type == TypeSymbol.Char ? char.MinValue : (long)int.MinValue;
            var maximum = type == TypeSymbol.Char ? char.MaxValue : (long)int.MaxValue;
            var cuts = new SortedSet<long> { minimum, maximum + 1 };
            foreach (var pattern in patterns)
            {
                if (pattern is RangeMatch range) { cuts.Add(range.Lower); cuts.Add(range.Upper + 1); }
                if (pattern is ConstantMatch { Value: not null } constant)
                {
                    var value = System.Convert.ToInt64(constant.Value, System.Globalization.CultureInfo.InvariantCulture);
                    cuts.Add(value); cuts.Add(value + 1);
                }
            }
            var points = cuts.ToArray();
            for (var index = 0; index < points.Length - 1; index++) yield return new(null, null, type, points[index], points[index + 1] - 1);
            // Open scalar selectors require a fallback even when ranges span all
            // currently representable integers; ranges are not subrange domains.
            yield return new(null, null, type, IsOther: true);
            yield break;
        }
        var constants = new HashSet<(TypeSymbol, object)>();
        foreach (var pattern in patterns.OfType<ConstantMatch>().Where(pattern => pattern.Value is not null))
            if (constants.Add((pattern.Expression.Type, pattern.Value!))) yield return new(null, pattern.Value, pattern.Expression.Type);
        var cases = new HashSet<TypeSymbol>();
        foreach (var pattern in patterns.OfType<CaseMatch>())
            if (cases.Add(pattern.Case.Type)) yield return new(pattern.Case, null, pattern.Case.Type);
        foreach (var pattern in patterns.OfType<TypeMatch>().DistinctBy(pattern => pattern.Target).Where(pattern => PossibleType(type, pattern.Target)))
            yield return new(null, null, pattern.Target);
        yield return new(null, null, type.IsValue ? type : null, IsOther: true);
    }

    private bool RuntimeAssignable(TypeSymbol actual, TypeSymbol target)
        => actual.IsValue && target.IsValue ? actual == target : references.CanAssign(actual, target);

    private bool PossibleType(TypeSymbol selector, TypeSymbol target)
    {
        static bool Sealed(TypeSymbol type) => type.Kind is TypeKind.Record or TypeKind.String || type.Definition?.Definition.Attributes.HasFlag(System.Reflection.TypeAttributes.Sealed) == true;
        if (selector.IsValue || Sealed(selector)) return RuntimeAssignable(selector, target);
        if (target.IsValue || Sealed(target)) return RuntimeAssignable(target, selector);
        return selector.Definition?.Definition.Attributes.HasFlag(System.Reflection.TypeAttributes.Interface) == true ||
            target.Definition?.Definition.Attributes.HasFlag(System.Reflection.TypeAttributes.Interface) == true ||
            RuntimeAssignable(selector, target) || RuntimeAssignable(target, selector);
    }

    private MatchPattern[]? Specialize(MatchPattern pattern, Constructor constructor)
    {
        static MatchPattern[] Wildcards(Constructor constructor) => constructor.Fields.Select(_ => (MatchPattern)new AnyMatch()).ToArray();
        if (pattern is AnyMatch) return Wildcards(constructor);
        if (pattern is TypeMatch tested)
            return !constructor.IsNull && constructor.ActualType is { } actual && RuntimeAssignable(actual, tested.Target) ? Wildcards(constructor) : null;
        if (constructor.IsNull) return pattern is ConstantMatch { Value: null } ? [] : null;
        if (pattern is CaseMatch member) return constructor.Case?.Type == member.Case.Type ? member.Fields.ToArray() : null;
        if (constructor.Case is not null || constructor.IsOther) return null;
        if (pattern is RangeMatch range) return constructor.Lower >= range.Lower && constructor.Upper <= range.Upper ? [] : null;
        if (pattern is ConstantMatch constant)
        {
            if (constant.Value is null) return null;
            if (constructor.Lower is { } low)
                return System.Convert.ToInt64(constant.Value, System.Globalization.CultureInfo.InvariantCulture) == low && constructor.Upper == low ? [] : null;
            return constant.Expression.Type == constructor.ActualType && Equals(constant.Value, constructor.Constant) ? [] : null;
        }
        return null;
    }
}
