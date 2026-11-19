// [Documentation](../../../docs/pascal/tools/refactoring.md#add-a-parameter)
using Fpas.Compiler.Analysis;
using Fpas.Compiler.Syntax;
using Fpas.Workspaces.Queries;

namespace Fpas.Workspaces.Refactoring;

/// <remarks>Parameter refactorings preserve existing direct-call bindings and argument evaluation order.</remarks>
public static partial class CheckedRefactoring
{
    private static IReadOnlyList<string> AddParameter(Draft draft, RefactoringRequest request, SourceSelection selection, SourceDocument source)
    {
        var name = IdentifierFacts.Spell(request.Name ?? "");
        var type = request.Type?.Trim();
        var argument = request.Argument?.Trim();
        if (type is not { Length: > 0 and <= 1024 } || argument is not { Length: > 0 and <= 8192 } ||
            !RefactoringSyntax.ParameterFragments(type, argument, draft.Token))
            throw Refuse("Supply one FPAS type and one positional value expression.");

        var definition = draft.Before.Query.Execute("definition", selection, cancellationToken: draft.Token).Definition;
        if (definition is not { Origin: "source", Symbol: { Kind: "routine", Declaration: { } declaration } })
            throw Refuse("Select a source FPAS routine or a checked call to it.");
        var declarationSource = Paths.Equals(source.Path, declaration.Path) ? source : draft.Source(declaration.Path);
        var routine = draft.Syntax(declarationSource).EditableRoutine(declaration.Offset);
        if (routine is null)
            throw Refuse("Add Parameter requires an editable FPAS routine without attributes.");
        if (routine.HasVariadicParameter) throw Refuse("A routine with a params parameter requires a manual signature change.");
        if (routine.Parameters.Any(parameter => parameter.Name == request.Name)) throw Refuse("The routine already declares that parameter.");
        var (sites, owners) = DirectCalls(draft, declarationSource, declaration.Offset);
        draft.Targets.AddRange(owners);

        var parameter = name + ": " + type;
        var insertion = routine.FirstOptional < routine.Parameters.Count ? parameter + "; " :
            (routine.Parameters.Count == 0 ? "" : "; ") + parameter;
        draft.Edit(declarationSource, routine.Insertion, routine.Insertion, insertion);
        foreach (var site in sites)
        {
            var call = site.Syntax;
            for (var index = routine.FirstOptional; index < call.Arguments.Count && call.Arguments[index].Name is null; index++)
            {
                var at = call.Arguments[index].Span.Offset;
                draft.Edit(site.Source, at, at, IdentifierFacts.Spell(routine.Parameters[index].Name!) + " := ");
            }
            draft.Edit(site.Source, call.Close, call.Close, (call.Arguments.Count > 0 ? ", " : "") + name + " := " + argument);
            draft.Calls.Add(new(site.Project, site.Snapshot, site.Operation, 1, routine.FirstOptional));
        }
        return ["The new value argument is evaluated after the existing explicit arguments. The routine body is preserved."];
    }

    private static IReadOnlyList<string> RemoveParameter(Draft draft, SourceSelection selection)
    {
        var definition = draft.Before.Query.Execute("definition", selection, cancellationToken: draft.Token).Definition;
        if (definition is not { Origin: "source", Symbol: { Kind: "parameter", Declaration: { } declaration } })
            throw Refuse("Select the unused value parameter of an editable FPAS routine.");
        var source = draft.Source(declaration.Path);
        var syntax = draft.Syntax(source);
        if (syntax.ValueParameterOwner(declaration.Offset) is not int owner || syntax.EditableRoutine(owner) is not { } routine || routine.HasVariadicParameter)
            throw Refuse("Remove Parameter requires an ordinary value parameter of an editable routine without attributes or params.");
        int slot = Enumerable.Range(0, routine.Parameters.Count).Single(index => routine.Parameters[index].Span.Offset == declaration.Offset);
        var (sites, owners) = DirectCalls(draft, source, owner);
        foreach (var (project, snapshot) in draft.Owners(source.Path).OrderBy(owner => owner.Project, StringComparer.Ordinal))
        {
            var parameter = snapshot.OccurrenceAt(source.Path, declaration.Offset);
            if (parameter?.Role != "declaration" || snapshot.Symbol(parameter.Symbol).Kind != "parameter" ||
                snapshot.Occurrences.Any(site => site.Symbol == parameter.Symbol && site.Role is not ("declaration" or "argument")))
                throw Refuse("The parameter is used by the routine, a contract or a nested body.");
            draft.Targets.Add(new(project, "parameter", definition.Symbol.Name, declaration,
                owners.Single(routineOwner => Paths.Equals(routineOwner.Project, project)).PublicApi));
        }
        Remove(source, routine.Parameters, slot);
        foreach (var site in sites)
        {
            if (!site.Operation.CanOmitArgument(slot))
                throw Refuse($"Cannot omit argument '{definition.Symbol.Name}' at {site.Source.Path}:{site.Operation.Span.Line}. Its evaluation or conversion may have effects or throw.");
            var args = site.Syntax.Arguments;
            for (int index = 0; index < args.Count; index++)
                if (args[index].Name is { } name ? name == definition.Symbol.Name : index == slot)
                {
                    Remove(site.Source, args, index);
                    break;
                }
            draft.Calls.Add(new(site.Project, site.Snapshot, site.Operation, -1, slot));
        }
        return [$"Remove unused parameter '{definition.Symbol.Name}'. Remaining arguments keep their evaluation order."];

        void Remove(SourceDocument document, IReadOnlyList<RefactoringListItem> items, int index)
        {
            var range = draft.Syntax(document).RemovalRange(items, index) ?? throw Refuse("Deleting this list item would erase a source comment.");
            draft.Removed.Add(range);
            draft.Edit(document, range.Offset, range.Offset + range.Length, "");
        }
    }

    private sealed record DirectCall(string Project, SemanticSnapshot Snapshot, SourceOperation Operation, SourceDocument Source, RefactoringCall Syntax);

    private static (List<DirectCall> Sites, List<RefactorTarget> Owners) DirectCalls(Draft draft, SourceDocument declarationSource, int routineOffset)
    {
        var targets = new HashSet<string>(StringComparer.Ordinal);
        var owners = new List<RefactorTarget>();
        foreach (var (project, snapshot) in draft.Owners(declarationSource.Path).OrderBy(owner => owner.Project, StringComparer.Ordinal))
        {
            var owner = snapshot.Symbols.SingleOrDefault(symbol => symbol.Kind == "routine" &&
                symbol.Declaration is { } at && Paths.Equals(at.Path, declarationSource.Path) && at.Offset == routineOffset);
            if (owner?.Declaration is not { } declaration || owner.Context?.Accessibility is not ("private" or "public"))
                throw Refuse("Every linked owner must provide an editable routine declaration.");
            targets.Add(draft.Before.Key(project, owner.Id));
            owners.Add(new(project, owner.Kind, owner.Name, declaration, RenameBindings.IsPublic(snapshot, owner)));
        }
        if (targets.Count == 0 || draft.Before.Relations(draft.Token).Any(relation =>
            targets.Contains(relation.Implementation) || targets.Contains(relation.Contract)))
            throw Refuse("The selected signature participates in an interface or external contract.");
        if (owners.Any(owner => owner.PublicApi))
            VerifyPublicParameterContext(draft, owners);

        var sites = new List<DirectCall>();
        foreach (var (project, snapshot) in draft.Projects)
        {
            var calls = new Dictionary<(string Path, int Offset), SourceOperation>();
            foreach (var operation in snapshot.Operations)
            {
                draft.Token.ThrowIfCancellationRequested();
                if (operation.Target is not { } id || !targets.Contains(draft.Before.Key(project, id))) continue;
                if (operation is not { Kind: "invoke", Dispatch: "direct", Anchor: { } anchor } ||
                    !calls.TryAdd((anchor.Path, anchor.Offset), operation))
                    throw Refuse("A delegate, indirect invocation or ambiguous call refers to the selected routine.");
            }
            foreach (var occurrence in snapshot.Occurrences)
            {
                draft.Token.ThrowIfCancellationRequested();
                if (occurrence.Role == "declaration" || !targets.Contains(draft.Before.Key(project, occurrence.Symbol))) continue;
                if (!calls.ContainsKey((occurrence.Span.Path, occurrence.Span.Offset)))
                    throw Refuse("A reference to the routine is outside the checked direct calls.");
            }
            foreach (var ((path, offset), operation) in calls)
            {
                var caller = Paths.Equals(path, declarationSource.Path) ? declarationSource : draft.Source(path);
                var call = draft.Syntax(caller).Call(operation.Span, offset)
                    ?? throw Refuse("A call could not be mapped back to its original argument list.");
                sites.Add(new(project, snapshot, operation, caller, call));
            }
        }
        foreach (var group in sites.GroupBy(site => (site.Source.Path, site.Operation.Anchor!.Offset)))
            if (group.Count() != draft.Owners(group.Key.Path).Count())
                throw Refuse("A call binds to a different routine in another owner of the shared file.");
        return (sites, owners);
    }
}
