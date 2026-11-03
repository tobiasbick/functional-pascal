using Fpas.Compiler.Binding.Types;
namespace Fpas.Compiler.Binding.References;

internal sealed partial class ReferenceCatalog
{
    internal static bool ArrayInterface(TypeSymbol type) => type.Definition?.FullName is
        "System.Collections.Generic.IEnumerable`1" or "System.Collections.Generic.ICollection`1" or "System.Collections.Generic.IList`1" or
        "System.Collections.Generic.IReadOnlyCollection`1" or "System.Collections.Generic.IReadOnlyList`1";
    internal ImportedType FindOpenType(string name, int position, Syntax.SourceText source)
    {
        var family = types.Where(entry => entry.Key.StartsWith(name + "`", StringComparison.Ordinal) &&
            int.TryParse(entry.Key[(name.Length + 1)..], out _)).SelectMany(entry => entry.Value).ToArray();
        if (family.Length != 1) throw source.Error(family.Length == 0 ? "FP4001" : "FP4002", "An inferred type owner must resolve to exactly one accessible generic definition; use explicit type arguments.", position);
        return family[0];
    }

    internal IEnumerable<TypeSymbol> InferenceViews(TypeSymbol type)
    {
        var seen = new HashSet<TypeSymbol>(); var pending = new Stack<TypeSymbol>(); pending.Push(type);
        while (pending.TryPop(out var current))
        {
            if (current == TypeSymbol.Unsupported || !seen.Add(current)) continue;
            yield return current;
            foreach (var parent in Parents(current)) pending.Push(parent);
        }
    }

    internal static bool UsableTemplate(TypeSymbol type)
        => type.Kind is not (TypeKind.Unsupported or TypeKind.TypeParameter) && (type.Element is null || UsableTemplate(type.Element)) && type.TypeArguments.All(UsableTemplate);

}
