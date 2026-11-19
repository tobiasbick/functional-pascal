// [Documentation](../../../docs/pascal/tools/refactoring.md#change-a-parameter-type-or-mode)
using Fpas.Compiler.Analysis;
using Fpas.Compiler.Syntax;
using Fpas.Workspaces.Queries;

namespace Fpas.Workspaces.Refactoring;

public static partial class CheckedRefactoring
{
    private static IReadOnlyList<string> ChangeSignature(Draft draft, RefactorSignatureChange? change, SourceSelection selection)
    {
        if (change is not { IsValid: true })
            throw Refuse("Supply a type or mode change with any call replacements, or a complete parameter order by itself.");
        if (change.Order is { } order) return ReorderParameters(draft, selection, order);
        var type = change.Type?.Trim();
        if (type is not null && !RefactoringSyntax.ParameterFragments(type, "0", draft.Token))
            throw Refuse("Supply a complete FPAS type for the selected parameter.");

        var definition = draft.Before.Query.Execute("definition", selection, cancellationToken: draft.Token).Definition;
        if (definition is not { Origin: "source", Symbol: { Kind: "parameter", Declaration: { } declaration } })
            throw Refuse("Select a parameter declaration, reference or argument label.");
        var source = draft.Source(declaration.Path);
        if (draft.Syntax(source).EditableParameter(declaration.Offset) is not { } parameter)
            throw Refuse("Change Signature requires an editable FPAS routine without attributes or params.");

        var mode = change.Mode switch { null => parameter.Mode, "value" => "", var requested => requested };
        var modeChanged = mode != parameter.Mode;
        var prefix = mode.Length == 0 ? "" : mode + " ";
        var replacements = new Dictionary<string, Dictionary<RefactorPosition, RefactorArgumentOverride>>(Paths);
        foreach (var replacement in change.Arguments)
        {
            var path = replacement.File is null ? source.Path : Path.GetFullPath(replacement.File);
            if (!replacements.TryGetValue(path, out var byPosition)) replacements.Add(path, byPosition = []);
            if (!byPosition.TryAdd(replacement.Call, replacement))
                throw Refuse("Supply one replacement per direct call.");
            if (!RefactoringSyntax.ParameterFragments("int", replacement.Expression.Trim(), draft.Token))
                throw Refuse("Supply a complete FPAS expression without a label or mode for each replacement.");
        }

        var (sites, owners) = DirectCalls(draft, source, parameter.Owner);
        draft.Targets.AddRange(owners);
        if (type is not null) Replace(source, parameter.Type, type);
        if (modeChanged) EditMode(source, parameter.Modifier, declaration.Offset);
        if (draft.Edits.Count == 0 && replacements.Count != 0)
            throw Refuse("Argument replacements require a change to the selected parameter.");

        draft.CheckOverloads = true;
        var used = new HashSet<RefactorArgumentOverride>();
        foreach (var site in sites.OrderByDescending(site => site.Operation.Span.Length))
        {
            var span = site.Operation.Span;
            if (draft.Removed.Any(removed => In(span, removed.Path, removed.Offset, removed.Offset + removed.Length)))
                continue;
            var anchor = site.Operation.Anchor!;
            var position = new RefactorPosition(anchor.Line, anchor.Column);
            string? replacement = null;
            if (replacements.TryGetValue(site.Source.Path, out var byPosition) && byPosition.TryGetValue(position, out var specified))
            {
                replacement = specified.Expression.Trim();
                used.Add(specified);
            }

            var argument = site.Syntax.Arguments.Where((item, index) => item.Name is null ? index == parameter.Index : item.Name == definition.Symbol.Name)
                .SingleOrDefault();
            if (argument is not null)
                EditArgument(site.Source, argument, replacement);
            else if (replacement is not null)
                draft.Edit(site.Source, site.Syntax.Close, site.Syntax.Close,
                    (site.Syntax.Arguments.Count == 0 ? "" : ", ") + IdentifierFacts.Spell(definition.Symbol.Name) + " := " + prefix + replacement);
            draft.Calls.Add(new(site.Project, site.Snapshot, site.Operation, 0, parameter.Index, mode));
        }
        if (used.Count != change.Arguments.Count)
            throw Refuse("Every replacement must identify an existing direct call by its called name and source file.");

        return ["The routine body is checked with the new signature. Direct calls retain the selected overload.",
            "Unchanged arguments keep their written evaluation order. Each supplied replacement is evaluated at its selected call."];

        void EditArgument(SourceDocument document, RefactoringArgument argument, string? replacement)
        {
            if (replacement is not null && !document.Text.AsSpan(argument.Value.Offset, argument.Value.Length).SequenceEqual(replacement))
            {
                if (argument.Modifier is null && modeChanged)
                    replacement = prefix + replacement;
                else if (modeChanged)
                    EditMode(document, argument.Modifier, argument.Value.Offset);
                Replace(document, argument.Value, replacement);
                return;
            }
            if (!modeChanged) return;
            EditMode(document, argument.Modifier, argument.Value.Offset);
            if (argument.Storage is int offset)
                draft.ChangedStorageAccess.Add((document.Path, offset));
        }

        void EditMode(SourceDocument document, SourceSpan? old, int start)
        {
            if (old is null)
                draft.Edit(document, start, start, prefix);
            else
            {
                var end = old.Offset + old.Length;
                if (mode.Length == 0)
                    while (end < start && document.Text[end] is ' ' or '\t') end++;
                draft.Edit(document, old.Offset, end, mode);
            }
        }

        void Replace(SourceDocument document, SourceSpan span, string replacement)
        {
            if (document.Text.AsSpan(span.Offset, span.Length).SequenceEqual(replacement)) return;
            if (draft.Syntax(document).ContainsComment(span))
                throw Refuse("The replacement would erase a source comment.");
            draft.Removed.Add(span);
            draft.Edit(document, span.Offset, span.Offset + span.Length, replacement);
        }
    }
}
