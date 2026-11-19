// [Documentation](../../../docs/pascal/tools/refactoring.md#extract-a-routine)
using System.Text;
using Fpas.Compiler.Analysis;
using Fpas.Compiler.Syntax;
using Fpas.Workspaces.Analysis;

namespace Fpas.Workspaces.Refactoring;

public static partial class CheckedRefactoring
{
    private sealed record RoutineExtractionEdit(SourceSpan Region, int Insertion, int NameOffset, int CallOffset);

    private static IReadOnlyList<string> ExtractRoutine(Draft draft, RefactoringRequest request, SourceDocument source, int start, int end)
    {
        var name = IdentifierFacts.Spell(request.Name ?? "");
        while (start < end && char.IsWhiteSpace(source.Text[start])) start++;
        while (end > start && char.IsWhiteSpace(source.Text[end - 1])) end--;
        RefactoringRoutineRegion? region;
        try { region = draft.Syntax(source).RoutineRegion(start, end); }
        catch (ArgumentException error) { throw Refuse(error.Message); }
        if (region is null) throw Refuse("Select complete, consecutive statements from one executable block, including the final semicolon.");

        var output = AnalyzeRoutineData(draft, region, out var notes);
        var resultType = output?.Type ?? region.ReturnType;
        var kind = resultType is null ? "procedure" : "function";
        var newline = LineEnding(source.Text, start);
        var callerIndent = LeadingIndent(source, start);
        var insertion = start;
        var written = new StringBuilder();
        var helperIndent = callerIndent;
        if (region.BeforeBegin is { } begin)
        {
            var beginIndent = LeadingIndent(source, begin);
            helperIndent = beginIndent + "  ";
            var line = source.Offset(source.Span(begin, 0).Line, 1);
            insertion = line + beginIndent.Length == begin ? line : begin;
            if (insertion == begin && line != begin) written.Append(newline);
            written.Append(helperIndent);
        }

        written.Append(kind).Append(' ');
        var nameOffset = written.Length;
        written.Append(name).Append("()");
        if (resultType is not null)
            written.Append(": ").Append(source.Text.AsSpan(resultType.Offset, resultType.Length));
        written.Append(';').Append(newline).Append(helperIndent).Append("begin").Append(newline);

        if (output is { } binding)
        {
            var cutoff = binding.PrefixStart;
            while (cutoff > start && char.IsWhiteSpace(source.Text[cutoff - 1])) cutoff--;
            if (cutoff > start)
            {
                AppendBody(start, cutoff, indentFirst: true);
                written.Append(newline);
            }
            written.Append(helperIndent).Append("  return ");
            AppendBody(binding.Initializer.Offset, End(binding.Initializer), indentFirst: false);
            written.Append(';');
        }
        else AppendBody(start, end, indentFirst: true);
        written.Append(newline).Append(helperIndent).Append("end ").Append(kind).Append(';').Append(newline);

        if (region.BeforeBegin is { } originalBegin)
        {
            if (insertion == originalBegin && source.Offset(source.Span(originalBegin, 0).Line, 1) != originalBegin)
                written.Append(LeadingIndent(source, originalBegin));
            draft.Edit(source, insertion, insertion, written.ToString());
            written.Clear();
        }
        else written.Append(newline).Append(callerIndent);

        if (output is { } declared)
            AppendOriginal(declared.PrefixStart, declared.Initializer.Offset, start);
        else if (region.TailReturn && resultType is not null)
            written.Append("return ");
        var callOffset = written.Length;
        written.Append(name).Append("()");
        if (output is { } initialized)
            AppendOriginal(End(initialized.Initializer), end, start);
        else
        {
            written.Append(';');
            if (region.TailReturn && resultType is null) written.Append(newline).Append(callerIndent).Append("return;");
        }

        draft.Edit(source, start, end, written.ToString());
        draft.RoutineExtraction = new(region.Span, insertion, nameOffset, callOffset);
        return notes;

        void AppendOriginal(int left, int right, int destination)
        {
            if (left == right) return;
            draft.Moves.Add(new(source.Span(left, right - left), destination, written.Length));
            written.Append(source.Text.AsSpan(left, right - left));
        }

        void AppendBody(int left, int right, bool indentFirst)
        {
            var lineStart = left;
            var indent = indentFirst;
            while (lineStart < right)
            {
                var lineEnd = source.Text.IndexOfAny(['\r', '\n'], lineStart, right - lineStart);
                if (lineEnd < 0) lineEnd = right;
                var content = lineStart;
                if (indent && !region.LiteralContinuations.Contains(lineStart))
                {
                    if (lineStart != left && callerIndent.Length <= lineEnd - lineStart &&
                        source.Text.AsSpan(lineStart, callerIndent.Length).SequenceEqual(callerIndent))
                        content += callerIndent.Length;
                    if (content < lineEnd) written.Append(helperIndent).Append("  ");
                }
                var next = lineEnd;
                if (next < right && source.Text[next] == '\r') next++;
                if (next < right && source.Text[next] == '\n') next++;
                AppendOriginal(content, next, insertion);
                lineStart = next;
                indent = true;
            }
        }
    }

    private static RefactoringRoutineOutput? AnalyzeRoutineData(Draft draft, RefactoringRoutineRegion region, out IReadOnlyList<string> notes)
    {
        var path = region.Span.Path;
        RefactoringRoutineOutput? output = null;
        string[]? agreedNotes = null;
        foreach (var (_, snapshot) in draft.Owners(path))
        {
            if (snapshot.Bodies.Any(body => body.Kind == "lambda" && Inside(body.Span)))
                throw Refuse("A selected anonymous routine must stay in its declaring scope.");
            var declarations = snapshot.Symbols.Where(symbol => symbol.Kind == "local" &&
                symbol.Declaration is { } declaration && Inside(declaration)).ToArray();
            var groups = snapshot.Occurrences.ToLookup(occurrence => occurrence.Symbol);
            var outgoing = declarations.Where(symbol => groups[symbol.Id].Any(occurrence => !Inside(occurrence.Span))).ToArray();
            if (outgoing.Length != 0 && (outgoing is not [{ Declaration: { } name }] || region.LastBinding is not { } last ||
                    name.Offset != last.Name.Span.Offset))
                throw Refuse("A local used after the selection must be the final selected binding and the sole new result.");
            var writes = snapshot.Occurrences.Where(occurrence => Inside(occurrence.Span) && occurrence.Role is "write" or "read_write")
                .Select(occurrence => occurrence.Symbol).ToHashSet(StringComparer.Ordinal);
            var external = snapshot.Occurrences.Where(occurrence => Inside(occurrence.Span) && occurrence.Role is "read" or "write" or "read_write")
                .Select(occurrence => snapshot.Symbol(occurrence.Symbol))
                .Where(symbol => symbol.Kind is "local" or "parameter" && symbol.Declaration is { } declaration && !Inside(declaration))
                .DistinctBy(symbol => symbol.Id).OrderBy(symbol => symbol.Declaration!.Offset).ToArray();
            var reads = external.Where(symbol => !writes.Contains(symbol.Id)).Select(symbol => symbol.Name).ToArray();
            var changed = external.Where(symbol => writes.Contains(symbol.Id)).Select(symbol => symbol.Name).ToArray();
            var decisions = new List<string> { "Move the selected statements to a nested helper called at their original location." };
            if (reads.Length != 0) decisions.Add("Capture outer bindings " + string.Join(", ", reads) + ".");
            if (changed.Length != 0) decisions.Add("Keep writes to outer storage for " + string.Join(", ", changed) + ".");
            if (outgoing is [var result]) decisions.Add("Return the initializer for " + result.Name + " to its original binding.");
            if (region.TailReturn) decisions.Add("Return from the original routine at the helper call site.");
            if (agreedNotes is not null && !agreedNotes.SequenceEqual(decisions))
                throw Refuse("Linked owners require different inputs or outputs for this selection.");
            agreedNotes = decisions.ToArray();
            output = outgoing.Length == 0 ? null : region.LastBinding;
        }
        notes = agreedNotes ?? throw Refuse("The selection has no checked source owner.");
        return output;

        bool Inside(SourceSpan span) => In(span, path, region.Span.Offset, End(region.Span));
    }

    private static string LeadingIndent(SourceDocument source, int offset)
    {
        var line = source.Offset(source.Span(offset, 0).Line, 1);
        var end = line;
        while (end < offset && source.Text[end] is ' ' or '\t') end++;
        return source.Text[line..end];
    }

    private static void VerifyRoutineExtraction(WorkspaceAnalysis updated, RenameBindings oldBindings, RenameBindings newBindings,
        OffsetMap offsets, Func<string, int, int> map, RoutineExtractionEdit extraction)
    {
        var region = extraction.Region;
        var path = region.Path;
        var name = offsets.Before(path, extraction.Insertion) + extraction.NameOffset;
        var call = offsets.Before(path, region.Offset) + extraction.CallOffset;
        foreach (var project in oldBindings.Analysis.Projects)
        {
            var before = project.Result!.Snapshot!;
            if (!before.Documents.Any(document => Paths.Equals(document.Path, path))) continue;
            var after = updated.Projects.Single(item => Paths.Equals(item.Project, project.Project)).Result!.Snapshot!;
            var declaration = after.OccurrenceAt(path, name);
            var invocation = after.Operations.SingleOrDefault(operation => operation.Kind == "invoke" &&
                operation.Anchor is { } anchor && Paths.Equals(anchor.Path, path) && anchor.Offset == call);
            if (declaration is not { Role: "declaration" } || invocation?.Target != declaration.Symbol)
                throw Conflict("The replacement call does not bind to the extracted helper.");
            VerifyMovedOperations(project.Project, path, before, after, oldBindings, newBindings, MapRange, invocation);
            var afterTypes = after.Expressions.Where(item => Paths.Equals(item.Span.Path, path))
                .ToDictionary(item => (item.Span.Offset, End(item.Span)), item => item.Type);
            var originalTypes = before.Types.ToDictionary(type => type.Id);
            var editedTypes = after.Types.ToDictionary(type => type.Id);
            foreach (var expression in before.Expressions.Where(item => In(item.Span, path, region.Offset, End(region))))
                if (!afterTypes.TryGetValue(MapRange(expression.Span), out var type) || !SameType(originalTypes[expression.Type], editedTypes[type]))
                    throw Conflict("An extracted expression changes type or loses a proven flow refinement.");
        }

        (int Start, int End) MapRange(SourceSpan span)
            => In(span, path, region.Offset, End(region))
                ? (map(path, span.Offset), map(path, End(span) - 1) + 1)
                : (offsets.After(path, span.Offset), offsets.Before(path, End(span)));
    }
}
