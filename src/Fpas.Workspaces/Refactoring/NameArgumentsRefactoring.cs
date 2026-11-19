// [Documentation](../../../docs/pascal/tools/refactoring.md#name-arguments-at-a-call)
using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Refactoring;

public static partial class CheckedRefactoring
{
    private static IReadOnlyList<string> NameArguments(Draft draft, SourceDocument source, int offset)
    {
        IReadOnlyList<(int Offset, string Label)>? shared = null;
        foreach (var (project, snapshot) in draft.Owners(source.Path))
        {
            draft.Token.ThrowIfCancellationRequested();
            var call = snapshot.Operations.Where(operation => operation.Kind is "invoke" or "delegate_invoke" or "construct" &&
                Paths.Equals(operation.Span.Path, source.Path) && operation.Span.Contains(offset))
                .MinBy(operation => operation.Span.Length);
            if (call is null || call.SuggestedArgumentLabels() is not { Count: > 0 } labels)
                throw Refuse("Select a call with positional arguments and at least two supplied parameters of the same input type. The arguments must have distinct usable parameter names.");
            if (shared is not null && !shared.SequenceEqual(labels))
                throw Refuse("Different owners of the shared file require different parameter labels.");
            shared = labels;
            draft.Calls.Add(new(project, snapshot, call, 0));
        }
        if (shared is null) throw Refuse("The selected source has no checked owner.");
        foreach (var (at, label) in shared) draft.Edit(source, at, at, label + " := ");
        return ["The arguments keep their written order and the call binds to the same overload."];
    }
}
