using Fpas.Compiler;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Snapshots;
using System.Security.Cryptography;
using System.Text;

namespace Fpas.Workspaces.Editing;

public sealed class DocumentEdits
{
    public TextDocumentSnapshot Original { get; }
    public string NewText { get; }
    public IReadOnlyList<DiagnosticEdit> Edits { get; }
    public string OriginalHash { get; }
    public string ReplacementHash { get; }
    internal DocumentEdits(TextDocumentSnapshot original, string text, DiagnosticEdit[] edits)
    {
        Original = original; NewText = text; Edits = Array.AsReadOnly(edits);
        OriginalHash = Convert.ToHexString(SHA256.HashData(original.EncodeUtf8()));
        ReplacementHash = Convert.ToHexString(SHA256.HashData(TextDocumentSnapshot.Utf8.GetBytes(text)));
    }
}

/// <summary>Validated original-offset edits over captured text. Construction performs no file I/O.</summary>
public sealed class TextEditSet
{
    public IReadOnlyList<DocumentEdits> Documents { get; }
    public int EditCount => Documents.Sum(document => document.Edits.Count);
    private TextEditSet(List<DocumentEdits> documents) => Documents = documents.AsReadOnly();

    public static TextEditSet Create(WorkspaceTextSnapshot snapshot, IEnumerable<DiagnosticEdit> edits, CancellationToken token = default)
    {
        token.ThrowIfCancellationRequested();
        var groups = new Dictionary<string, List<DiagnosticEdit>>(WorkspaceTextSnapshot.Paths); var count = 0;
        foreach (var edit in edits)
        {
            token.ThrowIfCancellationRequested();
            if (++count > 1000) throw new ContextFailure("fix_limit", "A correction batch exceeds 1,000 proposed edits.");
            var location = edit.Location; var document = snapshot.Document(location.Path);
            if (!Path.GetExtension(document.Path).Equals(".fpas", StringComparison.OrdinalIgnoreCase))
                throw new ContextFailure("invalid_fix", "A correction is outside the captured FPAS sources: " + document.Path);
            if (location.Offset is not { } offset || offset < 0 || location.Length < 0 || (long)offset + location.Length > document.Text.Length ||
                location.Length != edit.ExpectedText.Length || !document.Text.AsSpan(offset, location.Length).SequenceEqual(edit.ExpectedText))
                throw new ContextFailure("invalid_fix", "A correction does not match its captured source range: " + document.Path);
            if (!groups.TryGetValue(document.Path, out var group)) groups.Add(document.Path, group = []);
            if (!group.Any(previous => previous.Location.Offset == offset && previous.Location.Length == location.Length &&
                previous.ExpectedText == edit.ExpectedText && previous.NewText == edit.NewText)) group.Add(edit);
        }
        var documents = new List<DocumentEdits>(); long bytes = 0;
        foreach (var (path, group) in groups.OrderBy(pair => pair.Key, StringComparer.Ordinal))
        {
            token.ThrowIfCancellationRequested(); var original = snapshot.Document(path);
            var ordered = group.OrderBy(edit => edit.Location.Offset).ThenBy(edit => edit.Location.Length).ToArray();
            for (var index = 1; index < ordered.Length; index++)
            {
                var previous = ordered[index - 1].Location; var current = ordered[index].Location;
                if (current.Offset == previous.Offset || current.Offset < previous.Offset + previous.Length)
                    throw new ContextFailure("edit_conflict", "Automatic corrections overlap or compete at the same position: " + path);
            }
            var size = TextDocumentSnapshot.Utf8.GetByteCount(original.Text);
            if ((bytes += size) > 67_108_864) throw new ContextFailure("fix_limit", "Correction inputs exceed 64 MiB.");
            var replacementSize = size + ordered.Sum(edit => (long)TextDocumentSnapshot.Utf8.GetByteCount(edit.NewText) - TextDocumentSnapshot.Utf8.GetByteCount(edit.ExpectedText));
            if (replacementSize > 67_108_864 - bytes) throw new ContextFailure("fix_limit", "Correction inputs and replacements exceed 64 MiB.");
            var updated = new StringBuilder(original.Text);
            foreach (var edit in ordered.Reverse()) updated.Remove(edit.Location.Offset!.Value, edit.Location.Length).Insert(edit.Location.Offset.Value, edit.NewText);
            var text = updated.ToString(); bytes += replacementSize;
            if (text != original.Text) documents.Add(new(original, text, ordered));
        }
        return new(documents);
    }

    /// <summary>Creates independent byte buffers for an explicit filesystem publication.</summary>
    public IReadOnlyList<SourceChange> ToSourceChanges() => Array.AsReadOnly(Documents.Select(document => new SourceChange(document.Original.Path,
        document.Original.EncodeUtf8(), TextDocumentSnapshot.Utf8.GetBytes(document.NewText))).ToArray());
}
