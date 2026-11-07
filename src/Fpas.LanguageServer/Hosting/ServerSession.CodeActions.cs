using Fpas.Compiler;
using Fpas.Compiler.Analysis;
using Fpas.LanguageServer.Protocol;
using Fpas.Workspaces.Editing;
using Fpas.Workspaces.Live;
using Newtonsoft.Json.Linq;

namespace Fpas.LanguageServer.Hosting;

internal sealed partial class ServerSession
{
    private sealed record EditorFix(DiagnosticFix Fix, SourceSpan? Trigger);
    private bool correctionCommand;

    private object CodeActions(string method, JObject parameters, LiveSnapshot snapshot, CancellationToken token)
    {
        if (!renameSupported) throw Invalid("This client does not support versioned document edits.", -32601);
        if (method == "codeAction/resolve")
        {
            var data = Object(parameters["data"], "data");
            CheckItem(Object(data["stamp"], "stamp"), snapshot);
            var path = FileUris.Path(new Uri(RequiredString(data, "uri")));
            var index = RequiredInteger(data, "index");
            var fixes = EditorFixes(snapshot, path, token);
            if (index < 0 || index >= fixes.Length) throw Invalid("Unknown correction.");
            var edit = TextEditSet.Create(snapshot.Documents, fixes[index].Fix.Edits, token);
            return new { title = fixes[index].Fix.Title, kind = "quickfix", data, edit = FeatureProjection.WorkspaceEdits(edit) };
        }

        var documentPath = DocumentPath(parameters);
        var context = Object(parameters["context"], "context");
        if (context["only"] is JArray only && !only.Values<string>().Contains("quickfix")) return Array.Empty<object>();
        var open = snapshot.Documents.Document(documentPath);
        if (open.Version is null) return Array.Empty<object>();
        var document = new SourceDocument(documentPath, open.Text, token);
        var range = Object(parameters["range"], "range");
        int Offset(string side)
        {
            var position = Object(range[side], side);
            return document.Offset(checked(RequiredInteger(position, "line") + 1), checked(RequiredInteger(position, "character") + 1));
        }
        var start = Offset("start"); var end = Offset("end");
        if (end < start) throw Invalid("The range end precedes its start.");
        var suggestions = EditorFixes(snapshot, documentPath, token);
        return suggestions.Select((suggestion, index) => (suggestion, index)).Where(item => item.suggestion.Trigger is { } trigger
            ? trigger.Offset <= end && trigger.Offset + trigger.Length >= start
            : item.suggestion.Fix.Edits.Any(edit => edit.Location.Offset is { } offset && offset <= end && offset + edit.Location.Length >= start))
            .Select(item =>
            {
                var title = item.suggestion.Fix.Title;
                var data = new { stamp = ItemStamp(snapshot), uri = FileUris.Uri(documentPath), index = item.index };
                var command = correctionCommand ? new
                    { title, command = "functionalPascalClr.applyCorrection", arguments = new[] { new { data } } } : null;
                return new { title, kind = "quickfix", data, command };
            }).ToArray();
    }

    private static EditorFix[] EditorFixes(LiveSnapshot snapshot, string path, CancellationToken token)
    {
        var owner = snapshot.Queries().EditorOwner(new(path, 1, 1));
        return snapshot.Corrections(token).Suggestions.Where(item => PathComparer.Equals(item.Project, owner) &&
            item.Fix.Applicability == DiagnosticApplicability.Automatic &&
            item.Fix.Edits.All(edit => PathComparer.Equals(edit.Location.Path, path))).Select(item => new EditorFix(item.Fix, null))
            .Concat(CheckedImports.Find(snapshot.Analysis, owner, path, token).Select(item => new EditorFix(item.Fix, item.Trigger))).Distinct().ToArray();
    }
}
