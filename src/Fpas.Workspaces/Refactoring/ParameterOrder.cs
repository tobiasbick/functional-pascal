// [Documentation](../../../docs/pascal/tools/refactoring.md#reorder-parameters)
using Fpas.Compiler.Analysis;
using Fpas.Compiler.Syntax;
using Fpas.Workspaces.Queries;

namespace Fpas.Workspaces.Refactoring;

public static partial class CheckedRefactoring
{
    private static IReadOnlyList<string> ReorderParameters(Draft draft, SourceSelection selection, IReadOnlyList<string> order)
    {
        var definition = draft.Before.Query.Execute("definition", selection, cancellationToken: draft.Token).Definition;
        if (definition is not { Origin: "source", Symbol: { Kind: "routine", Declaration: { } declaration } })
            throw Refuse("Select an editable FPAS routine declaration or a direct call to it.");
        var source = draft.Source(declaration.Path);
        var syntax = draft.Syntax(source);
        if (syntax.EditableRoutine(declaration.Offset) is not { HasVariadicParameter: false } routine)
            throw Refuse("Reordering requires an editable FPAS routine without attributes or params.");

        var parameters = routine.Parameters;
        var slots = parameters.Select((parameter, index) => (parameter.Name, index))
            .ToDictionary(parameter => parameter.Name!, parameter => parameter.index, StringComparer.Ordinal);
        if (order.Count != parameters.Count || order.Any(name => !slots.ContainsKey(name)))
            throw Refuse("List each parameter of the selected routine exactly once in the new order.");
        var oldSlots = order.Select(name => slots[name]).ToArray();
        if (oldSlots.Take(routine.FirstOptional).Any(slot => slot >= routine.FirstOptional))
            throw Refuse("Keep required parameters before optional parameters.");

        var (sites, owners) = DirectCalls(draft, source, declaration.Offset);
        draft.Targets.AddRange(owners);
        var changed = Enumerable.Range(0, oldSlots.Length).Where(index => index != oldSlots[index]).ToArray();
        if (changed.Length == 0) return ["The parameters already have the requested order."];

        for (var gap = 0; gap <= parameters.Count; gap++)
        {
            var leftMoves = gap > 0 && gap - 1 != oldSlots[gap - 1];
            var rightMoves = gap < parameters.Count && gap != oldSlots[gap];
            if (!leftMoves && !rightMoves) continue;
            var start = gap == 0 ? routine.ParameterList.Offset : End(parameters[gap - 1].Span);
            var end = gap == parameters.Count ? End(routine.ParameterList) : parameters[gap].Span.Offset;
            if (syntax.ContainsComment(source.Span(start, end - start)))
                throw Refuse("A comment next to reordered parameters makes its intended attachment ambiguous.");
        }
        foreach (var destination in changed)
        {
            var original = parameters[oldSlots[destination]].Span;
            var target = parameters[destination].Span;
            draft.Edit(source, target.Offset, End(target), source.Text.Substring(original.Offset, original.Length));
            draft.Moves.Add(new(original, target.Offset));
        }

        var newSlots = new int[oldSlots.Length];
        for (var index = 0; index < oldSlots.Length; index++) newSlots[oldSlots[index]] = index;
        foreach (var site in sites)
        {
            var arguments = site.Syntax.Arguments;
            for (var index = changed[0]; index < arguments.Count && arguments[index].Name is null; index++)
            {
                var offset = arguments[index].Span.Offset;
                draft.Edit(site.Source, offset, offset, IdentifierFacts.Spell(parameters[index].Name!) + " := ");
            }
            draft.Calls.Add(new(site.Project, site.Snapshot, site.Operation, 0, NewSlots: newSlots));
        }
        draft.CheckOverloads = true;
        return ["Reorder parameters to " + string.Join(", ", order) + ".",
            "Affected positional arguments receive labels in place. Explicit arguments retain their written evaluation order."];

        static int End(SourceSpan span) => span.Offset + span.Length;
    }
}
