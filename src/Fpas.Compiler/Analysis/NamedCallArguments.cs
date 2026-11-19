// [Documentation](../../../docs/pascal/tools/refactoring.md)
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Analysis;

internal static class NamedCallArguments
{
    internal static TypeSymbol InputType(TypeSymbol type) => type.IsSpan ? TypeSymbol.Array(type.TypeArguments[0]) : type;

    internal static bool Useful(ImportedMethod method, IReadOnlyList<ExpressionSyntax>? supplied = null)
    {
        if ((supplied?.Count ?? method.Parameters.Count) < 2) return false;
        var names = new HashSet<string>(StringComparer.Ordinal);
        var types = new HashSet<TypeSymbol>();
        var named = supplied?.OfType<NamedArgumentSyntax>().Select(argument => argument.Name.Text).ToHashSet(StringComparer.Ordinal);
        var positional = (supplied?.Count ?? method.Parameters.Count) - (named?.Count ?? 0);
        bool repeated = false;
        for (int i = 0; i < method.Parameters.Count; i++)
        {
            var name = method.Parameter(i).Name;
            if (!IdentifierFacts.IsValid(name) || !names.Add(name)) return false;
            if ((i < positional || named?.Contains(name) == true) && !types.Add(InputType(method.Parameters[i]).Erased)) repeated = true;
        }
        return repeated;
    }

    internal static IReadOnlyList<(int Offset, string Label)> Suggestions(ImportedMethod method,
        IReadOnlyList<ExpressionSyntax> arguments, bool expandedParams)
    {
        if (arguments.Count < 2 || arguments.Count > method.Parameters.Count || arguments[0] is NamedArgumentSyntax)
            return [];
        // Expansion with no elements can still label the fixed arguments.
        if (expandedParams && arguments.Count == method.Parameters.Count)
            return [];
        if (!Useful(method, arguments)) return [];
        return Array.AsReadOnly(arguments.TakeWhile(argument => argument is not NamedArgumentSyntax)
            .Select((argument, slot) => (argument.SpanStart, IdentifierFacts.Spell(method.Parameter(slot).Name))).ToArray());
    }
}
