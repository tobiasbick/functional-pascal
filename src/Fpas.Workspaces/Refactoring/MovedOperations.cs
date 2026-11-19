// [Documentation](../../../docs/pascal/tools/refactoring.md#validation-and-plans)
using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Refactoring;

public static partial class CheckedRefactoring
{
    private static void VerifyMovedOperations(string project, string path, SemanticSnapshot before, SemanticSnapshot after,
        RenameBindings oldBindings, RenameBindings newBindings, Func<SourceSpan, (int Start, int End)> map, SourceOperation? introduced = null)
    {
        var prior = before.Operations.Where(InFile).ToLookup(operation => (map(operation.Span), operation.Kind));
        var current = after.Operations.Where(operation => InFile(operation) && operation != introduced)
            .ToLookup(operation => ((operation.Span.Offset, End(operation.Span)), operation.Kind));
        if (prior.Count != current.Count || prior.Any(group =>
        {
            var matches = current[group.Key].ToArray();
            return group.Count() != matches.Length || !group.Zip(matches).All(pair => SameOperation(pair.First, pair.Second));
        }))
            throw Conflict("Moving code changes an operation's binding, dispatch or generic substitution.");

        bool InFile(SourceOperation operation) => Paths.Equals(operation.Span.Path, path);

        bool SameOperation(SourceOperation left, SourceOperation right)
        {
            if (!left.HasSameDispatch(right) || left.Phase != right.Phase ||
                !TypesMatch(left.OwnerType, right.OwnerType) || !TypesMatch(left.ResultType, right.ResultType) ||
                (left.Target is { } old ? oldBindings.Key(project, old) : null) !=
                (right.Target is { } edited ? newBindings.Key(project, edited) : null))
                return false;
            if (left.Signature is not { } original || right.Signature is not { } replacement)
                return left.Signature is null && right.Signature is null;
            return TypesMatch(original.Result, replacement.Result) && original.TypeParameters.Count == replacement.TypeParameters.Count &&
                original.TypeParameters.Zip(replacement.TypeParameters).All(pair => TypesMatch(pair.First, pair.Second)) &&
                original.Parameters.Count == replacement.Parameters.Count && original.Parameters.Zip(replacement.Parameters).All(pair =>
                    pair.First.Name == pair.Second.Name && pair.First.Mode == pair.Second.Mode && TypesMatch(pair.First.Type, pair.Second.Type));
        }

        bool TypesMatch(string? left, string? right) => left is null || right is null ? left == right :
            SameType(before.Types.Single(type => type.Id == left), after.Types.Single(type => type.Id == right));
    }
}
