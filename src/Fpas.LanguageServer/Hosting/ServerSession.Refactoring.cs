// [Documentation](../../../docs/pascal/tools/language-server.md#checked-refactoring-previews)
using Fpas.Compiler.Analysis;
using Fpas.LanguageServer.Protocol;
using Fpas.Workspaces.Live;
using Fpas.Workspaces.Refactoring;
using Newtonsoft.Json.Linq;

namespace Fpas.LanguageServer.Hosting;

internal sealed partial class ServerSession
{
    private sealed record RetainedRefactoring(PreparedRefactoring Plan, long Revision, long Expires);
    private readonly Dictionary<string, RetainedRefactoring> refactorings = new(StringComparer.Ordinal);

    private object Refactor(string method, JObject parameters, LiveSnapshot snapshot, CancellationToken token)
    {
        if (!renameSupported) throw Invalid("This client does not support versioned document edits.", -32601);
        if (RequiredInteger(parameters, "schemaVersion") != 1) throw Invalid("Use FPAS refactoring schema 1.");
        if (method == "fpas/prepareRefactoring")
        {
            var selection = EditorSelection(parameters, snapshot, position: true);
            var end = Object(parameters["end"], "end");
            var kind = RequiredString(parameters, "kind");
            var request = new RefactoringRequest(kind, selection,
                checked(RequiredInteger(end, "line") + 1), checked(RequiredInteger(end, "character") + 1),
                Optional("name"), Optional("type"), Optional("argument"),
                kind == "change-signature" ? new(Optional("type"), Optional("mode"), SignatureArguments(parameters["arguments"]),
                    ParameterOrder(parameters["order"])) : null,
                parameters["closedContext"] is not null && Boolean(parameters, "closedContext"),
                kind == "implement-protocol" ? new(RequiredString(parameters, "protocol"), ProtocolMembers(parameters["members"])) : null);
            var plan = CheckedRefactoring.Prepare(snapshot.Analysis, request, token);
            var id = Guid.NewGuid().ToString("N");
            var now = Environment.TickCount64;
            lock (gate)
            {
                foreach (var key in refactorings.Where(item => item.Value.Expires <= now).Select(item => item.Key).ToArray())
                    refactorings.Remove(key);
                // Dictionary order is not insertion order after removals; evict the oldest preview explicitly.
                if (refactorings.Count == 8) refactorings.Remove(refactorings.MinBy(item => item.Value.Expires).Key);
                refactorings.Add(id, new(plan, snapshot.Revision, now + 300_000));
            }
            return new
            {
                schemaVersion = 1, id, kind = request.Kind, project = FileUris.Uri(plan.Project), snapshotId = plan.SnapshotId,
                selection = plan.Selection, notes = plan.Notes, binaryBoundaries = plan.BinaryBoundaries,
                canApply = plan.Edits.EditCount > 0 && plan.Edits.Documents.All(document => document.Original.Version is not null),
                files = plan.Edits.Documents.Select(document =>
                {
                    var original = new SourceDocument(document.Original.Path, document.Original.Text, token);
                    return new
                    {
                        uri = FileUris.Uri(original.Path), version = document.Original.Version,
                        edits = document.Edits.Select(edit => new
                        {
                            range = FeatureProjection.Range(original, original.Span(edit.Location.Offset!.Value, edit.Location.Length)),
                            expectedText = edit.ExpectedText, newText = edit.NewText
                        }).ToArray()
                    };
                }).ToArray()
            };
        }

        var selected = RequiredString(parameters, "id");
        RetainedRefactoring? retained;
        lock (gate) refactorings.Remove(selected, out retained);
        if (method == "fpas/discardRefactoring") return new { discarded = retained is not null };
        if (retained is null || retained.Expires <= Environment.TickCount64 || retained.Revision != snapshot.Revision ||
            retained.Plan.SnapshotId != snapshot.Analysis.Context.SnapshotId)
            throw Invalid("The refactoring preview is stale. Prepare it again.", -32801);
        if (retained.Plan.Edits.Documents.Any(document => document.Original.Version is null))
            throw Invalid("Open all affected source files and prepare the preview again before applying it.", -32803);
        var refreshed = CheckedRefactoring.Prepare(snapshot.Analysis, retained.Plan.Request, token);
        if (!SameEdits(retained.Plan, refreshed)) throw Invalid("The checked edit changed after preview.", -32801);
        return new { schemaVersion = 1, id = selected, edit = FeatureProjection.WorkspaceEdits(refreshed.Edits) };

        string? Optional(string property) => parameters[property] is null ? null : RequiredString(parameters, property);
    }

    private static IReadOnlyList<string>? ParameterOrder(JToken? token)
    {
        if (token is null) return null;
        if (token is not JArray { Count: > 0 and <= 1000 } names || names.Any(name => name.Type != JTokenType.String))
            throw Invalid("order must be a non-empty array of parameter names.");
        return names.Values<string>().Select(name => name!).ToArray();
    }

    private static IReadOnlyList<RefactorProtocolMember> ProtocolMembers(JToken? token)
    {
        if (token is null) return [];
        if (token is not JArray { Count: <= 1000 } members)
            throw Invalid("members must be an array of at most 1000 member implementations.");
        return members.Select(member =>
        {
            var item = Object(member, "member");
            return new RefactorProtocolMember(RequiredString(item, "key"), RequiredString(item, "value"),
                item["forward"] is not null && Boolean(item, "forward"));
        }).ToArray();
    }

    private static IReadOnlyList<RefactorArgumentOverride> SignatureArguments(JToken? token)
    {
        if (token is null) return [];
        if (token is not JArray { Count: <= 1000 } arguments)
            throw Invalid("arguments must be an array of at most 1000 call replacements.");
        return arguments.Select(argument =>
        {
            var item = Object(argument, "argument");
            var position = Object(item["position"], "position");
            var line = RequiredInteger(position, "line");
            var column = RequiredInteger(position, "character");
            if (line is < 0 or int.MaxValue || column is < 0 or int.MaxValue)
                throw Invalid("Invalid call position.");
            var file = item["uri"] is null ? null : FileUris.Path(new Uri(RequiredString(item, "uri"), UriKind.RelativeOrAbsolute));
            return new RefactorArgumentOverride(new(line + 1, column + 1), RequiredString(item, "expression"), file);
        }).ToArray();
    }

    private static bool SameEdits(PreparedRefactoring before, PreparedRefactoring after)
        => before.Project == after.Project && before.Edits.Documents.Count == after.Edits.Documents.Count &&
            before.Edits.Documents.Zip(after.Edits.Documents).All(pair =>
                pair.First.Original.Path == pair.Second.Original.Path && pair.First.Original.Version == pair.Second.Original.Version &&
                pair.First.OriginalHash == pair.Second.OriginalHash && pair.First.ReplacementHash == pair.Second.ReplacementHash &&
                pair.First.Edits.SequenceEqual(pair.Second.Edits));
}
