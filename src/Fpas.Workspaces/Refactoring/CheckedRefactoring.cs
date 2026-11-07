using Fpas.Compiler;
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;
using Fpas.Workspaces.Snapshots;

namespace Fpas.Workspaces.Refactoring;

public sealed record RefactoringRequest(string Kind, SourceSelection Selection, int EndLine, int EndColumn,
    string? Name = null, string? Type = null, string? Argument = null);

public sealed record PreparedRefactoring(RefactoringRequest Request, string Project, SourceSpan Selection, string SnapshotId,
    TextEditSet Edits, IReadOnlyList<string> Notes, IReadOnlyList<string> BinaryBoundaries);

/// <summary>Proposes FPAS source edits and checks them in an isolated overlay.</summary>
public static partial class CheckedRefactoring
{
    private static StringComparer Paths => WorkspaceTextSnapshot.Paths;
    public static PreparedRefactoring Prepare(WorkspaceAnalysis analysis, RefactoringRequest request, CancellationToken token = default)
    {
        if (!analysis.Success || !analysis.IsComplete) throw Refuse("Refactoring requires successful and complete analysis of the context.");
        var before = new RenameBindings(analysis);
        var file = Path.GetFullPath(request.Selection.File);
        var selection = request.Selection with { File = file };
        var project = before.Query.EditorOwner(selection);
        var syntax = new SourceDocument(file, analysis.Context.Texts.Document(file).Text, token);
        var start = syntax.Offset(selection.Line, selection.Column);
        var end = syntax.Offset(request.EndLine, request.EndColumn);
        if (end < start) throw Refuse("The selection end precedes its start.");
        var draft = new Draft(analysis, before, token);
        var details = request.Kind switch
        {
            "add-parameter" => AddParameter(draft, request, selection, syntax),
            "extract-variable" => ExtractVariable(draft, request, syntax, start, end),
            "organize-imports" => OrganizeImports(draft, syntax),
            _ => throw Refuse("Unknown FPAS refactoring.")
        };
        var edits = TextEditSet.Create(analysis.Context.Texts, draft.Edits, token);
        if (edits.EditCount != 0) draft.Verify(edits);
        return new(request with { Selection = selection }, project, syntax.Span(start, end - start),
            analysis.Context.SnapshotId, edits, details, before.Query.Boundaries());
    }

    private static ContextFailure Refuse(string message) => new("refactor_not_supported", message);
    private static ContextFailure Conflict(string message) => new("refactor_conflict", message);
    private static bool In(SourceSpan span, string file, int start, int end)
        => Paths.Equals(span.Path, file) && start <= span.Offset && span.Offset < end;

    private sealed record ParameterCall(string Project, SemanticSnapshot Snapshot, SourceOperation Call, int ParameterIndex);

    private sealed class Draft(WorkspaceAnalysis analysis, RenameBindings before, CancellationToken token)
    {
        private readonly Dictionary<string, RefactoringSyntax> syntax = new(Paths);
        internal WorkspaceAnalysis Analysis => analysis;
        internal RenameBindings Before => before;
        internal CancellationToken Token => token;
        internal List<DiagnosticEdit> Edits { get; } = [];
        internal List<SourceSpan> Removed { get; } = [];
        internal List<ParameterCall> Calls { get; } = [];
        internal (SourceSpan Original, int Insertion, int Relative)? Move { get; set; }
        internal (SourceSpan Span, int Insertion, string Name)? Extracted { get; set; }
        internal IEnumerable<(string Project, SemanticSnapshot Snapshot)> Projects => analysis.Projects.Select(project => (project.Project, project.Result!.Snapshot!));

        internal RefactoringSyntax Syntax(SourceDocument document)
        {
            if (!syntax.TryGetValue(document.Path, out var parsed)) syntax.Add(document.Path, parsed = new(document, token));
            return parsed;
        }

        internal SourceDocument Source(string file) => new(file, analysis.Context.Texts.Document(file).Text, token);

        internal IEnumerable<(string Project, SemanticSnapshot Snapshot)> Owners(string file)
            => Projects.Where(project => project.Snapshot.Documents.Any(document => Paths.Equals(document.Path, file)));

        internal void Edit(SourceDocument document, int start, int end, string replacement)
        {
            var span = document.Span(start, end - start);
            Edits.Add(new(new(document.Path, span.Line, span.Column, span.Length, Offset: start), document.Text[start..end], replacement));
            if (Edits.Count > 1000) throw Refuse("A refactoring cannot exceed 1,000 source edits.");
        }

        internal void Verify(TextEditSet edits)
        {
            var files = edits.Documents.ToDictionary(document => document.Original.Path, Paths);
            var offsetMap = new OffsetMap(files);
            int Map(string path, int offset)
            {
                if (Move is { } move && Paths.Equals(path, move.Original.Path) && move.Original.Contains(offset))
                    return offsetMap.Before(path, move.Insertion) + move.Relative + offset - move.Original.Offset;
                return offsetMap.After(path, offset);
            }
            var updated = RenameOverlay.Analyze(analysis.Context, files.ToDictionary(pair => pair.Key, pair => pair.Value.NewText, Paths),
                token, "refactor_conflict");
            var oldBindings = new RenameBindings(analysis, Map);
            var newBindings = new RenameBindings(updated);
            var expected = oldBindings.Occurrences(token, span => !Removed.Any(removed =>
                In(span, removed.Path, removed.Offset, removed.Offset + removed.Length)));
            var actual = newBindings.Occurrences(token);
            if (expected.Any(item => !actual.TryGetValue(item.Key, out var value) || value != item.Value) ||
                actual.Keys.Any(key => !expected.ContainsKey(key) && !offsetMap.Generated(key.Path, key.Offset)) ||
                !oldBindings.Relations(token).SetEquals(newBindings.Relations(token)))
                throw Conflict("The edit changes a source binding, overload selection or declaration relationship.");

            if (Extracted is { } extraction)
                VerifyExtraction(updated, extraction, offsetMap.Before(extraction.Span.Path, extraction.Insertion), Move!.Value.Relative);
            foreach (var call in Calls) VerifyCall(updated, call, offsetMap);
        }

        private void VerifyExtraction(WorkspaceAnalysis updated, (SourceSpan Span, int Insertion, string Name) extraction, int insertion, int expressionOffset)
        {
            var span = extraction.Span;
            foreach (var (project, original) in Owners(span.Path))
            {
                var snapshot = updated.Projects.Single(item => Paths.Equals(item.Project, project)).Result!.Snapshot!;
                var local = snapshot.Symbols.SingleOrDefault(symbol => symbol.Name == extraction.Name && symbol.Kind == "local" &&
                    symbol.Declaration is { } declaration && In(declaration, span.Path, insertion, insertion + "const ".Length + extraction.Name.Length + 1));
                var expected = ExactType(original, span);
                var moved = ExactType(snapshot, span with { Offset = insertion + expressionOffset });
                if (local?.Type is null || expected is null || moved is null || !SameType(expected, moved) ||
                    !SameType(expected, snapshot.Types.Single(type => type.Id == local.Type)))
                    throw Conflict("The extracted binding does not have the selected expression's original type.");
            }
        }

        private void VerifyCall(WorkspaceAnalysis updated, ParameterCall original, OffsetMap offsets)
        {
            token.ThrowIfCancellationRequested();
            var snapshot = updated.Projects.Single(item => Paths.Equals(item.Project, original.Project)).Result!.Snapshot!;
            var anchor = original.Call.Anchor!;
            var position = offsets.After(anchor.Path, anchor.Offset);
            var call = snapshot.Operations.SingleOrDefault(operation => operation.Kind == "invoke" &&
                operation.Anchor is { } at && Paths.Equals(at.Path, anchor.Path) && at.Offset == position);
            if (original.Call.Signature is not { } before || call?.Signature is not { } after ||
                after.Parameters.Count != before.Parameters.Count + 1 ||
                before.TypeParameters.Count != after.TypeParameters.Count ||
                !TypesMatch(before.Result, after.Result) ||
                !before.TypeParameters.Zip(after.TypeParameters).All(pair => TypesMatch(pair.First, pair.Second)))
                throw Conflict("The proposed argument changes the selected call's types or generic substitution.");
            var retained = after.Parameters.Where((_, index) => index != original.ParameterIndex);
            if (!before.Parameters.Zip(retained).All(pair =>
                pair.First.Name == pair.Second.Name && pair.First.Mode == pair.Second.Mode && TypesMatch(pair.First.Type, pair.Second.Type)))
                throw Conflict("The proposed argument changes an existing parameter binding or mode.");

            bool TypesMatch(string left, string right) => SameType(original.Snapshot.Types.Single(type => type.Id == left),
                snapshot.Types.Single(type => type.Id == right));
        }
    }

    private sealed class OffsetMap(Dictionary<string, DocumentEdits> files)
    {
        internal int Before(string path, int offset) => Position(path, offset, insertion: false);
        internal int After(string path, int offset) => Position(path, offset, insertion: true);

        private int Position(string path, int offset, bool insertion)
            => files.TryGetValue(path, out var file) ? offset + file.Edits.Where(edit => edit.Location.Offset < offset &&
                edit.Location.Offset + edit.Location.Length <= offset || insertion && edit.Location.Offset == offset && edit.Location.Length == 0)
                .Sum(edit => edit.NewText.Length - edit.Location.Length) : offset;

        internal bool Generated(string path, int offset)
        {
            if (!files.TryGetValue(path, out var file)) return false;
            var delta = 0;
            foreach (var edit in file.Edits)
            {
                var start = edit.Location.Offset!.Value + delta;
                if (start <= offset && offset < start + edit.NewText.Length) return true;
                delta += edit.NewText.Length - edit.Location.Length;
            }
            return false;
        }
    }
}
