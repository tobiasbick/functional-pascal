using Fpas.Compiler.Analysis;
using Fpas.Compiler.Syntax;
using Fpas.Workspaces.Queries;

namespace Fpas.Workspaces.Refactoring;

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
            throw Refuse("Select a private FPAS routine or a checked call to it.");
        var declarationSource = Paths.Equals(source.Path, declaration.Path) ? source : draft.Source(declaration.Path);
        var routine = draft.Syntax(declarationSource).PrivateRoutine(declaration.Offset);
        if (routine is null)
            throw Refuse("Add Parameter requires a private FPAS routine without attributes or an external contract.");
        if (routine.HasVariadicParameter) throw Refuse("A routine with a params parameter requires a manual signature change.");
        if (routine.Parameters.Contains(request.Name, StringComparer.Ordinal)) throw Refuse("The routine already declares that parameter.");

        var targets = new HashSet<string>(StringComparer.Ordinal);
        foreach (var (project, snapshot) in draft.Owners(declaration.Path))
        {
            var owner = snapshot.Symbols.SingleOrDefault(symbol => symbol.Kind == "routine" &&
                symbol.Declaration is { } at && Paths.Equals(at.Path, declaration.Path) && at.Offset == declaration.Offset);
            if (owner is null || owner.Context?.Accessibility != "private")
                throw Refuse("The routine is not private in every linked owner.");
            targets.Add(draft.Before.Key(project, owner.Id));
        }
        if (targets.Count == 0 || draft.Before.Relations(draft.Token).Any(relation =>
            targets.Contains(relation.Implementation) || targets.Contains(relation.Contract)))
            throw Refuse("The selected signature participates in an interface or external contract.");

        var parameter = name + ": " + type;
        var insertion = routine.FirstOptional < routine.Parameters.Count ? parameter + "; " :
            (routine.Parameters.Count == 0 ? "" : "; ") + parameter;
        draft.Edit(declarationSource, routine.Insertion, routine.Insertion, insertion);
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
                var caller = Paths.Equals(path, source.Path) ? source : draft.Source(path);
                var call = draft.Syntax(caller).Call(operation.Span, offset)
                    ?? throw Refuse("A call could not be mapped back to its original argument list.");
                for (var index = routine.FirstOptional; index < call.PositionalArguments.Count; index++)
                {
                    var at = call.PositionalArguments[index];
                    draft.Edit(caller, at, at, IdentifierFacts.Spell(routine.Parameters[index]) + " := ");
                }
                draft.Edit(caller, call.Close, call.Close, (call.HasArguments ? ", " : "") + name + " := " + argument);
                draft.Calls.Add(new(project, snapshot, operation, routine.FirstOptional));
            }
        }
        return ["The new value argument is evaluated after the existing explicit arguments. The routine body is preserved."];
    }
}
