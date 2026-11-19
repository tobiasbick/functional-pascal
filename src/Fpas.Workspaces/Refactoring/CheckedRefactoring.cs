// [Documentation](../../../docs/pascal/tools/refactoring.md#validation-and-plans)
using Fpas.Compiler;
using Fpas.Compiler.Analysis;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Queries;
using Fpas.Workspaces.Snapshots;

namespace Fpas.Workspaces.Refactoring;

/// <summary>Requests a checked refactoring over an exact source selection.</summary>
/// <param name="Kind">Name of the checked refactoring operation.</param>
/// <param name="Selection">One-based UTF-16 selection start and optional project owner.</param>
/// <param name="EndLine">One-based exclusive end line.</param>
/// <param name="EndColumn">One-based UTF-16 exclusive end column.</param>
/// <param name="Name">New unescaped identifier for extraction or parameter addition.</param>
/// <param name="Type">Explicit FPAS type for parameter addition.</param>
/// <param name="Argument">FPAS value expression appended at each affected call.</param>
/// <param name="SignatureChange">Requested type or mode and explicit call replacements.</param>
/// <param name="ClosedContext">Whether the caller confirms all consumers of an edited public signature belong to this context.</param>
/// <param name="Protocol">Requested protocol type and explicit member implementations.</param>
public sealed record RefactoringRequest(string Kind, SourceSelection Selection, int EndLine, int EndColumn,
    string? Name = null, string? Type = null, string? Argument = null, RefactorSignatureChange? SignatureChange = null,
    bool ClosedContext = false, RefactorProtocol? Protocol = null);

/// <summary>Contains a proposed edit batch checked in a private semantic overlay.</summary>
/// <param name="Request">Normalized originating request.</param>
/// <param name="Project">Selected source owner.</param>
/// <param name="Selection">Original selection span.</param>
/// <param name="SnapshotId">Input snapshot identity.</param>
/// <param name="Edits">Detached validated edits.</param>
/// <param name="Notes">Behavior and limitation explanations.</param>
/// <param name="BinaryBoundaries">Reference assemblies outside editable source ownership.</param>
public sealed record PreparedRefactoring(RefactoringRequest Request, string Project, SourceSpan Selection, string SnapshotId,
    TextEditSet Edits, IReadOnlyList<string> Notes, IReadOnlyList<string> BinaryBoundaries)
{
    internal IReadOnlyList<RefactorTarget> Targets { get; init; } = [];
}

/// <summary>Proposes FPAS source edits and checks them in an isolated overlay.</summary>
public static partial class CheckedRefactoring
{
    private static StringComparer Paths => WorkspaceTextSnapshot.Paths;
    /// <summary>Prepares source edits and proves existing bindings and declaration relationships are retained.</summary>
    /// <param name="analysis">Successful complete context analysis.</param>
    /// <param name="request">Requested refactoring and source selection.</param>
    /// <param name="token">Cancels syntax work and overlay analysis.</param>
    /// <returns>A detached checked proposal without publishing files.</returns>
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
        var draft = new Draft(analysis, before, token, request.ClosedContext);
        var details = request.Kind switch
        {
            "add-parameter" => AddParameter(draft, request, selection, syntax),
            "remove-parameter" => RemoveParameter(draft, selection),
            "implement-protocol" => ImplementProtocol(draft, request.Protocol, selection),
            "change-signature" => ChangeSignature(draft, request.SignatureChange, selection),
            "name-arguments" => NameArguments(draft, syntax, start),
            "inline-variable" => InlineVariable(draft, selection),
            "extract-routine" => ExtractRoutine(draft, request, syntax, start, end),
            "extract-variable" => ExtractVariable(draft, request, syntax, start, end),
            "organize-imports" => OrganizeImports(draft, syntax),
            _ => throw Refuse("Unknown FPAS refactoring.")
        };
        var edits = TextEditSet.Create(analysis.Context.Texts, draft.Edits, token);
        if (edits.EditCount != 0) draft.Verify(edits);
        if (request.Kind != "implement-protocol" && draft.Targets.Any(target => target.PublicApi))
            details = [.. details, "Public signature change. Every consumer must be editable in the confirmed context."];
        return new(request with { Selection = selection }, project, syntax.Span(start, end - start),
            analysis.Context.SnapshotId, edits, details, before.Query.Boundaries()) { Targets = draft.Targets };
    }

    private static ContextFailure Refuse(string message) => new("refactor_not_supported", message);
    private static ContextFailure Conflict(string message) => new("refactor_conflict", message);
    private static bool In(SourceSpan span, string file, int start, int end)
        => Paths.Equals(span.Path, file) && start <= span.Offset && span.Offset < end;

    private sealed record ParameterCall(string Project, SemanticSnapshot Snapshot, SourceOperation Call, int ParameterCountChange,
        int ParameterIndex = -1, string? ChangedMode = null, IReadOnlyList<int>? NewSlots = null);

    private sealed record SourceMove(SourceSpan Original, int Destination, int Relative = 0);

    private sealed class Draft(WorkspaceAnalysis analysis, RenameBindings before, CancellationToken token, bool closedContext)
    {
        private readonly Dictionary<string, RefactoringSyntax> syntax = new(Paths);
        internal WorkspaceAnalysis Analysis => analysis;
        internal RenameBindings Before => before;
        internal CancellationToken Token => token;
        internal bool ClosedContext => closedContext;
        internal List<DiagnosticEdit> Edits { get; } = [];
        internal List<SourceSpan> Removed { get; } = [];
        internal List<RefactorTarget> Targets { get; } = [];
        internal List<ParameterCall> Calls { get; } = [];
        internal bool CheckOverloads { get; set; }
        internal SourceSpan? ImplementedRecord { get; set; }
        internal List<(string Name, int Insertion, int Relative)> ForwardedMethods { get; } = [];
        internal HashSet<(string Path, int Offset)> ChangedStorageAccess { get; } = [];
        internal List<SourceMove> Moves { get; } = [];
        internal (SourceSpan Span, int Insertion, string Name)? Extracted { get; set; }
        internal InlineVariableEdit? Inlined { get; set; }
        internal RoutineExtractionEdit? RoutineExtraction { get; set; }
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
            var moves = Moves.GroupBy(move => move.Original.Path, Paths).ToDictionary(group => group.Key,
                group => group.OrderBy(move => move.Original.Offset).Select(move =>
                    (move.Original, Offset: offsetMap.Before(move.Original.Path, move.Destination) + move.Relative)).ToArray(), Paths);
            int Map(string path, int offset)
            {
                if (moves.TryGetValue(path, out var ranges))
                {
                    var first = 0;
                    var last = ranges.Length - 1;
                    while (first <= last)
                    {
                        var index = first + (last - first) / 2;
                        var (original, destination) = ranges[index];
                        if (original.Contains(offset)) return destination + offset - original.Offset;
                        if (offset < original.Offset) last = index - 1;
                        else first = index + 1;
                    }
                }
                return offsetMap.After(path, offset);
            }
            var updated = RenameOverlay.Analyze(analysis.Context, files.ToDictionary(pair => pair.Key, pair => pair.Value.NewText, Paths),
                token, "refactor_conflict");
            var oldBindings = new RenameBindings(analysis, Map);
            var newBindings = new RenameBindings(updated);
            var newStorageAccess = ChangedStorageAccess.Select(site => (site.Path, Map(site.Path, site.Offset))).ToHashSet();
            var expected = oldBindings.Occurrences(token, span => !Removed.Any(removed =>
                In(span, removed.Path, removed.Offset, removed.Offset + removed.Length)),
                (span, role) => StorageRole(span, role, ChangedStorageAccess));
            var actual = newBindings.Occurrences(token, roleAt: (span, role) => StorageRole(span, role, newStorageAccess));
            if (expected.Any(item => !actual.TryGetValue(item.Key, out var value) || value != item.Value) ||
                actual.Keys.Any(key => !expected.ContainsKey(key) && !offsetMap.Generated(key.Path, key.Offset)) ||
                !PreservedRelations(newBindings, oldBindings.Relations(token), offsetMap))
                throw Conflict("The edit changes a source binding, overload selection or declaration relationship.");

            if (Extracted is { } extraction)
                VerifyExtraction(updated, extraction, offsetMap.Before(extraction.Span.Path, extraction.Insertion), Moves.Single().Relative);
            if (RoutineExtraction is { } routine)
                VerifyRoutineExtraction(updated, oldBindings, newBindings, offsetMap, Map, routine);
            if (Inlined is { } inlined) VerifyInlining(updated, oldBindings, newBindings, offsetMap, inlined);
            foreach (var call in Calls) VerifyCall(updated, call, offsetMap, oldBindings, newBindings);
            if (CheckOverloads) VerifyOverloads(updated, offsetMap, Map);
            if (ForwardedMethods.Count != 0) VerifyForwarding(updated, offsetMap, newBindings);

            static string StorageRole(SourceSpan span, string role, HashSet<(string Path, int Offset)> changed)
                => role is "read" or "write" or "read_write" && changed.Contains((span.Path, span.Offset)) ? "storage" : role;
        }

        private bool PreservedRelations(RenameBindings updated, HashSet<(string Project, string Implementation, string Contract)> old,
            OffsetMap offsets)
        {
            var current = updated.Relations(token);
            if (ImplementedRecord is not { } record) return old.SetEquals(current);
            if (!old.IsSubsetOf(current)) return false;
            current.ExceptWith(old);
            foreach (var project in updated.Analysis.Projects)
            {
                var snapshot = project.Result!.Snapshot!;
                foreach (var relation in snapshot.Relations)
                {
                    var key = (project.Project, updated.Key(project.Project, relation.Implementation), updated.Key(project.Project, relation.Contract));
                    if (current.Contains(key) && !RecordMember(updated.Resolve(project.Project, relation.Implementation), snapshot, record, offsets))
                        return false;
                }
            }
            return true;
        }

        private void VerifyForwarding(WorkspaceAnalysis updated, OffsetMap offsets, RenameBindings bindings)
        {
            var record = ImplementedRecord!;
            foreach (var project in updated.Projects)
            {
                var snapshot = project.Result!.Snapshot!;
                if (!snapshot.Documents.Any(document => Paths.Equals(document.Path, record.Path))) continue;
                foreach (var (name, insertion, relative) in ForwardedMethods)
                {
                    var calls = snapshot.Operations.Where(operation => operation.Kind == "invoke" &&
                        operation.Anchor is { } anchor && Paths.Equals(anchor.Path, record.Path) &&
                        anchor.Offset == offsets.Before(record.Path, insertion) + relative &&
                        operation.Target is { } target && snapshot.Symbol(target).Name == name).ToArray();
                    if (calls.Length != 1 || calls[0].Target is not { } called ||
                        !RecordMember(bindings.Resolve(project.Project, called), snapshot, record, offsets) ||
                        bindings.Resolve(project.Project, called).Symbol.Declaration is not { } declaration ||
                        offsets.Generated(declaration.Path, declaration.Offset))
                        throw Conflict($"Forwarding must call an existing instance method named '{name}' on the selected record.");
                }
            }
        }

        private static bool RecordMember(DefinitionResult definition, SemanticSnapshot snapshot, SourceSpan record, OffsetMap offsets)
            => definition.Origin == "source" && definition.Symbol.Context?.Owner is { } owner &&
                snapshot.Symbols.FirstOrDefault(symbol => symbol.Id == owner) is { Kind: "record", Declaration: { } declaration } &&
                Paths.Equals(declaration.Path, record.Path) && declaration.Offset == offsets.After(record.Path, record.Offset);

        private void VerifyOverloads(WorkspaceAnalysis updated, OffsetMap offsets, Func<string, int, int> map)
        {
            foreach (var (project, snapshot) in Projects)
            {
                token.ThrowIfCancellationRequested();
                var after = updated.Projects.Single(item => Paths.Equals(item.Project, project)).Result!.Snapshot!;
                var expected = Overloads(snapshot)
                    .Where(item => !Removed.Any(range => In(item.Anchor, range.Path, range.Offset, range.Offset + range.Length)))
                    .ToLookup(item => (item.Anchor.Path, Offset: map(item.Anchor.Path, item.Anchor.Offset), item.Call.Kind));
                var actual = Overloads(after).Where(item => !offsets.Generated(item.Anchor.Path, item.Anchor.Offset) ||
                    expected.Contains((item.Anchor.Path, item.Anchor.Offset, item.Call.Kind)))
                    .ToLookup(item => (item.Anchor.Path, item.Anchor.Offset, item.Call.Kind));
                if (expected.Count != actual.Count || expected.Any(group =>
                {
                    var matches = actual[group.Key].ToArray();
                    return group.Count() != matches.Length ||
                        !group.Zip(matches).All(pair => ImplementedRecord is null
                            ? pair.First.Call.HasSameDispatch(pair.Second.Call)
                            : pair.First.Call.Dispatch == pair.Second.Call.Dispatch &&
                                pair.First.Call.Operation?.GetKey(includeAssemblyHash: false) == pair.Second.Call.Operation?.GetKey(includeAssemblyHash: false));
                }))
                    throw Conflict("The refactoring alters an existing constructor or operator overload.");
            }

            static IEnumerable<(SourceSpan Anchor, SourceOperation Call)> Overloads(SemanticSnapshot snapshot)
                => snapshot.Operations.Where(call => call.Kind is "construct" or "operator")
                    .Select(call => (call.Anchor ?? call.Span, call));
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

        private void VerifyCall(WorkspaceAnalysis updated, ParameterCall original, OffsetMap offsets, RenameBindings oldBindings, RenameBindings newBindings)
        {
            token.ThrowIfCancellationRequested();
            var snapshot = updated.Projects.Single(item => Paths.Equals(item.Project, original.Project)).Result!.Snapshot!;
            var span = original.Call.Span;
            var start = offsets.After(span.Path, span.Offset);
            var end = offsets.Before(span.Path, span.Offset + span.Length);
            var call = snapshot.Operations.SingleOrDefault(operation => operation.Kind == original.Call.Kind &&
                Paths.Equals(operation.Span.Path, span.Path) && operation.Span.Offset == start && start + operation.Span.Length == end);
            if (original.Call.Signature is not { } before || call?.Signature is not { } after ||
                after.Parameters.Count != before.Parameters.Count + original.ParameterCountChange ||
                before.TypeParameters.Count != after.TypeParameters.Count ||
                !TypesMatch(before.Result, after.Result) ||
                !before.TypeParameters.Zip(after.TypeParameters).All(pair => TypesMatch(pair.First, pair.Second)))
                throw Conflict("The proposed argument changes the selected call's types or generic substitution.");
            var changed = original.ChangedMode is not null;
            var signatureChanged = changed || original.ParameterCountChange != 0 || original.NewSlots is not null;
            if (!(signatureChanged ? original.Call.Dispatch == call.Dispatch : original.Call.HasSameDispatch(call)) ||
                (original.Call.Target is { } oldTarget ? oldBindings.Key(original.Project, oldTarget) : null) !=
                (call.Target is { } newTarget ? newBindings.Key(original.Project, newTarget) : null) ||
                !TypesMatch(original.Call.OwnerType, call.OwnerType))
                throw Conflict("The edit changes the selected call target.");
            if (changed && (after.Parameters[original.ParameterIndex].Name != before.Parameters[original.ParameterIndex].Name ||
                after.Parameters[original.ParameterIndex].Mode != original.ChangedMode))
                throw Conflict("The changed parameter does not retain its slot and requested mode.");
            var retainedBefore = before.Parameters.Where((_, index) => index != original.ParameterIndex || !changed && original.ParameterCountChange >= 0);
            var retainedAfter = original.NewSlots is { } slots
                ? slots.Select(slot => after.Parameters[slot])
                : after.Parameters.Where((_, index) => index != original.ParameterIndex || !changed && original.ParameterCountChange <= 0);
            if (!retainedBefore.Zip(retainedAfter).All(pair =>
                pair.First.Name == pair.Second.Name && pair.First.Mode == pair.Second.Mode && TypesMatch(pair.First.Type, pair.Second.Type)))
                throw Conflict("The proposed argument changes an existing parameter binding or mode.");

            bool TypesMatch(string? left, string? right) => left is null || right is null ? left == right : SameType(original.Snapshot.Types.Single(type => type.Id == left),
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
