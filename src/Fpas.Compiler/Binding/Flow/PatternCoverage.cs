// [Documentation](../../../../docs/pascal/language/pattern-matching/exhaustiveness.md)
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using System.Globalization;
using System.Numerics;

namespace Fpas.Compiler.Binding.Flow;

// A pattern matrix is specialized one constructor at a time. Payload columns
// preserve correlations, so nested cases are checked without enumerating values.
internal sealed class PatternCoverage(ReferenceCatalog references, Syntax.SourceText source, int position)
{
    private sealed record Constructor(VariantCaseInfo? Case, object? Constant, TypeSymbol? ActualType, BigInteger? Lower = null, BigInteger? Upper = null, bool IsNull = false, bool IsOther = false)
    {
        internal IReadOnlyList<TypeSymbol> Fields => Case?.Fields.Select(payload => payload.Type).ToArray() ?? [];
    }

    private int steps;

    internal bool Useful(IReadOnlyList<MatchPattern> covered, MatchPattern candidate, TypeSymbol type)
    {
        steps = 0;
        return Useful(Rows(covered), [candidate], [type]);
    }

    internal void RequireExhaustive(IReadOnlyList<MatchPattern> covered, TypeSymbol type, Func<VariantCaseInfo, string?> caseName)
    {
        var matrix = Rows(covered);
        MatchPattern[] candidate = [new AnyMatch()];
        TypeSymbol[] types = [type];
        steps = 0;
        if (!Useful(matrix, candidate, types)) return;
        var witness = new CoverageWitness(references, caseName);
        if (!Useful(matrix, candidate, types, witness, chooseConstructor: true))
            throw source.Error("FP3025", "Case matching is not exhaustive; add the missing cases or a fallback.", position);
        var pattern = witness.Patterns.Pop();
        throw source.Error("FP3025", $"Case matching is not exhaustive. Missing pattern: {pattern}.", position,
            expectation: new("pattern", [pattern], "non-exhaustive case"));
    }

    private static MatchPattern[][] Rows(IReadOnlyList<MatchPattern> patterns) => patterns.Select(pattern => new[] { pattern }).ToArray();

    private bool Useful(IReadOnlyList<MatchPattern[]> matrix, MatchPattern[] candidate, TypeSymbol[] types,
        CoverageWitness? witness = null, bool chooseConstructor = false)
    {
        if (++steps > 100_000)
        {
            if (witness is not null) return false;
            throw source.Error("FP3025", "Pattern coverage is too complex; simplify the nested patterns.", position);
        }
        if (candidate.Length == 0) return matrix.Count == 0;
        if (matrix.Any(row => row.All(pattern => pattern is AnyMatch))) return false;
        if (candidate[0] is AnyMatch && matrix.All(row => row[0] is AnyMatch) && !(chooseConstructor && matrix.Count == 0))
        {
            if (!Useful(matrix.Select(row => row[1..]).ToArray(), candidate[1..], types[1..], witness)) return false;
            witness?.Patterns.Push("_");
            return true;
        }
        var heads = matrix.Select(row => row[0]).Append(candidate[0]).ToArray();
        foreach (var constructor in Constructors(types[0], heads))
        {
            var selected = Specialize(candidate[0], constructor);
            if (selected is null) continue;
            var head = witness?.Spell(constructor);
            if (witness is not null && head is null) continue;
            var rows = new List<MatchPattern[]>();
            foreach (var row in matrix)
                if (Specialize(row[0], constructor) is { } fields) rows.Add([.. fields, .. row.Skip(1)]);
            if (!Useful(rows, [.. selected, .. candidate.Skip(1)], [.. constructor.Fields, .. types.Skip(1)], witness)) continue;
            witness?.Rebuild(head!, selected.Length);
            return true;
        }
        return false;
    }

    // Each recursive success leaves its columns on the stack in source order.
    // Unwinding a constructor combines its field columns into a single pattern.
    private sealed class CoverageWitness(ReferenceCatalog references, Func<VariantCaseInfo, string?> caseName)
    {
        internal Stack<string> Patterns { get; } = new();

        internal string? Spell(Constructor constructor)
        {
            if (constructor.Case is { } item) return caseName(item);
            if (constructor.IsNull) return "null";
            if (constructor.Constant is bool value) return value ? "true" : "false";
            if (constructor.Lower is { } lower && constructor.Upper is { } upper && constructor.ActualType is { } type)
                return IntegerPattern(BigInteger.Clamp(BigInteger.Zero, lower, upper), references.ScalarInfo(type)?.Base ?? type);
            return null;
        }

        private static string IntegerPattern(BigInteger value, TypeSymbol type)
        {
            var text = value.ToString(CultureInfo.InvariantCulture);
            return type.Kind switch
            {
                TypeKind.Int32 => text,
                TypeKind.Int64 => text + "L",
                TypeKind.UInt32 => text + "U",
                TypeKind.UInt64 => text + "UL",
                _ => $"cast {text + (value > long.MaxValue ? "UL" : NumericFacts.Fits(value, TypeSymbol.Int32) ? "" : "L")} as {type}"
            };
        }

        internal void Rebuild(string head, int fieldCount)
        {
            var fields = new string[fieldCount];
            for (var index = 0; index < fields.Length; index++) fields[index] = Patterns.Pop();
            Patterns.Push(fieldCount == 0 ? head : $"{head}({string.Join(", ", fields)})");
        }
    }

    private IEnumerable<Constructor> Constructors(TypeSymbol type, IReadOnlyList<MatchPattern> patterns)
    {
        if (type.IsNullable || type == TypeSymbol.Null) yield return new(null, null, null, IsNull: true);
        if (type == TypeSymbol.Null) yield break;
        type = type.NonNullable;
        if (references.VariantInfo(type) is { } variant)
        {
            foreach (var member in variant.Cases) yield return new(member, null, member.Type);
            yield break;
        }
        if (type == TypeSymbol.Boolean)
        {
            yield return new(null, false, type); yield return new(null, true, type); yield break;
        }
        var domain = references.ScalarInfo(type);
        if (NumericFacts.Integral(type) || domain?.Category == Syntax.ScalarCategory.Range)
        {
            var minimum = domain?.Lower ?? NumericFacts.Min(type);
            var maximum = domain?.Upper ?? NumericFacts.Max(type);
            var cuts = new SortedSet<BigInteger> { minimum, maximum + 1 };
            foreach (var pattern in patterns)
            {
                if (pattern is RangeMatch range) { cuts.Add(range.Lower); cuts.Add(range.Upper + 1); }
                if (pattern is ConstantMatch { Value: not null } constant)
                {
                    var value = NumericValue.Integer(constant.Value);
                    cuts.Add(value); cuts.Add(value + 1);
                }
            }
            var points = cuts.ToArray();
            for (var index = 0; index < points.Length - 1; index++) yield return new(null, null, type, points[index], points[index + 1] - 1);
            // Open scalar selectors require a fallback even when ranges span all
            // currently representable integers; ranges are not subrange domains.
            if (domain?.Category != Syntax.ScalarCategory.Range) yield return new(null, null, type, IsOther: true);
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
        return selector.IsInterface || target.IsInterface ||
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
                return NumericValue.Integer(constant.Value) == low && constructor.Upper == low ? [] : null;
            return constant.Expression.Type == constructor.ActualType && Equals(constant.Value, constructor.Constant) ? [] : null;
        }
        return null;
    }
}
