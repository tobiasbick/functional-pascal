using Fpas.Compiler.Binding.Types;
namespace Fpas.Compiler.Binding.References;

internal sealed partial class ReferenceCatalog
{
    internal static bool MoreSpecific(ImportedMethod left, ImportedMethod right)
    {
        static bool Variable(TypeSymbol type, ImportedMethod method) => type.Kind == TypeKind.MethodParameter ||
            type.IsParameter && method.SourceMethod?.GenericParameters.Contains(type.Parameter!) == true;
        int Compare(TypeSymbol l, TypeSymbol r)
        {
            var lv = Variable(l, left); var rv = Variable(r, right);
            if (lv || rv) return lv == rv ? 0 : lv ? -1 : 1;
            if (l.Kind == TypeKind.Array && r.Kind == TypeKind.Array && l.Rank == r.Rank) return Compare(l.Element!, r.Element!);
            if (l.Definition != r.Definition || l.Record != r.Record || l.Union != r.Union || l.TypeArguments.Count != r.TypeArguments.Count) return 0;
            var nested = l.TypeArguments.Select((type, index) => Compare(type, r.TypeArguments[index])).ToArray();
            return nested.Any(value => value < 0) ? -1 : nested.Any(value => value > 0) ? 1 : 0;
        }
        var l = left.SignatureParameters ?? left.SourceMethod!.Parameters.Select(parameter => parameter.Type).ToArray();
        var r = right.SignatureParameters ?? right.SourceMethod!.Parameters.Select(parameter => parameter.Type).ToArray();
        var comparisons = l.Select((type, index) => Compare(type, r[index])).ToArray();
        return comparisons.Any(value => value > 0) && comparisons.All(value => value >= 0);
    }
}
