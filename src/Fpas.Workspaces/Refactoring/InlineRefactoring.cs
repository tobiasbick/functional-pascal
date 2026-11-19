// [Documentation](../../../docs/pascal/tools/refactoring.md#inline-a-local-variable)
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Queries;

namespace Fpas.Workspaces.Refactoring;

public static partial class CheckedRefactoring
{
    private sealed record InlineVariableEdit(SourceSpan Initializer, IReadOnlyList<SourceSpan> Reads);

    private static IReadOnlyList<string> InlineVariable(Draft draft, SourceSelection selection)
    {
        var definition = draft.Before.Query.Execute("definition", selection, cancellationToken: draft.Token).Definition;
        if (definition is not { Origin: "source", Symbol: { Kind: "local", Declaration: { } declaration } })
            throw Refuse("Select a local declaration or a read of that local.");
        var source = draft.Source(declaration.Path);
        var syntax = draft.Syntax(source);
        if (syntax.InlineInitializer(declaration.Offset) is not { HasSuspensionOrRoutine: false } binding)
            throw Refuse("Select an ordinary local const or var with an initializer that neither suspends nor defines a routine.");

        var initializer = binding.Initializer;
        var removal = StatementRemoval(source, binding.Statement);
        if (definition.Symbol.Context?.Documentation is not null ||
            syntax.ContainsComment(source.Span(removal.Offset, initializer.Offset - removal.Offset)) ||
            syntax.ContainsComment(source.Span(End(initializer), End(removal) - End(initializer))))
            throw Refuse("Removing the declaration would discard a comment.");

        SourceSpan[]? reads = null;
        foreach (var (owner, snapshot) in draft.Owners(source.Path).OrderBy(owner => owner.Project, StringComparer.Ordinal))
        {
            var occurrence = snapshot.OccurrenceAt(source.Path, declaration.Offset);
            var symbol = occurrence is null ? null : snapshot.Symbol(occurrence.Symbol);
            if (symbol is not { Kind: "local", Type: { } type, Declaration: { } name } ||
                !Paths.Equals(name.Path, declaration.Path) || name.Offset != declaration.Offset)
                throw Refuse("Every linked owner must provide the same local declaration.");
            var mentions = snapshot.Occurrences.Where(site => site.Symbol == symbol.Id && site.Role != "declaration").ToArray();
            if (mentions.Length == 0)
                throw Refuse("The local has no reads to inline.");
            if (mentions.Any(site => site.Role != "read" || !Paths.Equals(site.Span.Path, source.Path)))
                throw Refuse("Inlining requires a local with no writes or mutable storage uses.");
            var locations = mentions.Select(site => source.Span(site.Span.Offset, site.Span.Length)).OrderBy(span => span.Offset).ToArray();
            if (reads is not null && !reads.SequenceEqual(locations))
                throw Refuse("Linked owners resolve different reads of the selected local.");
            reads = locations;
            var declared = snapshot.Types.Single(item => item.Id == type);
            if (declared.Callable is not null || ExactType(snapshot, initializer) is not { } original || !SameType(original, declared) ||
                reads.Any(site => ExactType(snapshot, site) is not { } effective || !SameType(effective, declared)))
                throw Refuse("The initializer must already have the local's type at every read, without a storage conversion or flow refinement.");
            if (!binding.Literal && (reads is not [var read] || syntax.FirstReadAfter(declaration.Offset, read) is not { } calls ||
                    calls.Any(call => !snapshot.Operations.Any(operation => operation.Kind == "invoke" &&
                        (!call.RequiresStatic || operation.IsStaticInvocation()) && operation.Anchor is { } anchor &&
                        Paths.Equals(anchor.Path, source.Path) && anchor.Offset == call.Offset))))
                throw Refuse("An initializer that needs evaluation must have one read evaluated first in the immediately following statement in the same block.");
            draft.Targets.Add(new(owner, symbol.Kind, symbol.Name, name, false));
        }
        if (reads is null) throw Refuse("The local has no checked source owner.");
        if (reads.Length != 1 && syntax.ContainsComment(initializer))
            throw Refuse("Inlining a repeated literal would duplicate its comment.");
        var replacement = "(" + source.Text.Substring(initializer.Offset, initializer.Length) + ")";
        foreach (var read in reads)
        {
            draft.Edit(source, read.Offset, End(read), replacement);
            draft.Removed.Add(read);
        }
        draft.Edit(source, removal.Offset, End(removal), "");
        if (reads.Length == 1)
        {
            draft.Moves.Add(new(initializer, reads[0].Offset, Relative: 1));
            draft.Removed.Add(source.Span(removal.Offset, initializer.Offset - removal.Offset));
            draft.Removed.Add(source.Span(End(initializer), End(removal) - End(initializer)));
        }
        else
        {
            draft.Removed.Add(removal);
        }
        draft.Inlined = new(initializer, reads);
        return binding.Literal
            ? ["Replace the local's reads with its literal value."]
            : ["Move the initializer to the first evaluated value of the next statement in the same block."];

        static SourceSpan StatementRemoval(SourceDocument document, SourceSpan statement)
        {
            var text = document.Text;
            var lineStart = document.Offset(statement.Line, 1);
            var end = End(statement);
            var after = end;
            while (after < text.Length && text[after] is ' ' or '\t') after++;
            if (after < text.Length && text.AsSpan(after).StartsWith("//", StringComparison.Ordinal))
                throw Refuse("Removing the declaration would detach its trailing comment.");
            if (text.AsSpan(lineStart, statement.Offset - lineStart).Trim().IsEmpty &&
                (after == text.Length || text[after] is '\r' or '\n'))
            {
                after += after < text.Length && text[after] == '\r' ? 1 : 0;
                after += after < text.Length && text[after] == '\n' ? 1 : 0;
                return document.Span(lineStart, after - lineStart);
            }
            return document.Span(statement.Offset, after - statement.Offset);
        }
    }

    private static void VerifyInlining(WorkspaceAnalysis updated, RenameBindings oldBindings, RenameBindings newBindings,
        OffsetMap offsets, InlineVariableEdit inline)
    {
        var path = inline.Initializer.Path;
        foreach (var project in oldBindings.Analysis.Projects)
        {
            var before = project.Result!.Snapshot!;
            var after = updated.Projects.Single(item => Paths.Equals(item.Project, project.Project)).Result!.Snapshot!;
            if (!before.Documents.Any(document => Paths.Equals(document.Path, path))) continue;
            var original = ExactType(before, inline.Initializer)!;
            foreach (var read in inline.Reads)
            {
                var inserted = ExactType(after, inline.Initializer with { Offset = offsets.Before(path, read.Offset) + 1 });
                if (inserted is null || !SameType(original, inserted))
                    throw Conflict("The initializer changes type when inserted at a read.");
            }

            var previous = before.Operations.Where(InFile).ToLookup(operation => (Map(operation.Span), operation.Kind));
            var current = after.Operations.Where(InFile).ToLookup(operation => ((operation.Span.Offset, End(operation.Span)), operation.Kind));
            if (previous.Count != current.Count || previous.Any(group =>
            {
                var matches = current[group.Key].ToArray();
                return group.Count() != matches.Length || !group.Zip(matches).All(pair => SameOperation(pair.First, pair.Second));
            }))
                throw Conflict("Inlining changes an operation's binding, dispatch or generic substitution.");

            bool InFile(SourceOperation operation) => Paths.Equals(operation.Span.Path, path);
            (int Start, int End) Map(SourceSpan span)
            {
                if (inline.Initializer.Contains(span.Offset) && End(span) <= End(inline.Initializer) && inline.Reads is [var read])
                {
                    var start = offsets.Before(path, read.Offset) + 1 + span.Offset - inline.Initializer.Offset;
                    return (start, start + span.Length);
                }
                return (offsets.After(path, span.Offset), offsets.Before(path, End(span)));
            }

            bool SameOperation(SourceOperation left, SourceOperation right)
            {
                if (!left.HasSameDispatch(right) || !TypesMatch(left.OwnerType, right.OwnerType) ||
                    !TypesMatch(left.ResultType, right.ResultType) ||
                    (left.Target is { } old ? oldBindings.Key(project.Project, old) : null) !=
                    (right.Target is { } edited ? newBindings.Key(project.Project, edited) : null))
                    return false;
                if (left.Signature is not { } prior || right.Signature is not { } next)
                    return left.Signature is null && right.Signature is null;
                return TypesMatch(prior.Result, next.Result) && prior.TypeParameters.Count == next.TypeParameters.Count &&
                    prior.TypeParameters.Zip(next.TypeParameters).All(pair => TypesMatch(pair.First, pair.Second)) &&
                    prior.Parameters.Count == next.Parameters.Count && prior.Parameters.Zip(next.Parameters).All(pair =>
                        pair.First.Name == pair.Second.Name && pair.First.Mode == pair.Second.Mode && TypesMatch(pair.First.Type, pair.Second.Type));
            }

            bool TypesMatch(string? left, string? right) => left is null || right is null ? left == right :
                SameType(before.Types.Single(type => type.Id == left), after.Types.Single(type => type.Id == right));
        }
    }

    private static int End(SourceSpan span) => span.Offset + span.Length;
}
