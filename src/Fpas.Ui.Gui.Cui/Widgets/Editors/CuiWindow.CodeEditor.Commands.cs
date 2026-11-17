using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private enum EditorOperation { GoTo, Select, Find }
    private sealed record EditorRequest(EditorOperation Operation, int First = 0, int Second = 0, string? Query = null);
    private readonly Dictionary<string, EditorRequest> editorRequests = new(StringComparer.Ordinal);

    /// <summary>Queues a one-based line/visual-column navigation for the next enabled editor view.</summary>
    /// <param name="id">The editor's stable tracking identity.</param>
    /// <param name="line">Positive line; CUI clamps it to the document.</param>
    /// <param name="column">Positive visual column, including native tab stops.</param>
    public void CodeEditorGoTo(string id, int line, int column = 1)
    {
        CheckEditorRequest(id);
        ArgumentOutOfRangeException.ThrowIfLessThan(line, 1);
        ArgumentOutOfRangeException.ThrowIfLessThan(column, 1);
        editorRequests[id] = new(EditorOperation.GoTo, line, column);
    }

    /// <summary>Queues selection at UTF-16 scalar boundaries in the next view's model text.</summary>
    /// <param name="id">The editor's stable tracking identity.</param>
    /// <param name="anchor">Nonnegative UTF-16 anchor offset; may follow the cursor.</param>
    /// <param name="cursor">Nonnegative UTF-16 caret offset.</param>
    public void CodeEditorSelect(string id, int anchor, int cursor)
    {
        CheckEditorRequest(id);
        ArgumentOutOfRangeException.ThrowIfNegative(anchor);
        ArgumentOutOfRangeException.ThrowIfNegative(cursor);
        editorRequests[id] = new(EditorOperation.Select, anchor, cursor);
    }

    /// <summary>Queues opening the native find bar, or closing it with a null query.</summary>
    /// <param name="id">The editor's stable tracking identity.</param>
    /// <param name="query">Valid Unicode of at most 255 UTF-8 bytes, or null to close.</param>
    public void CodeEditorFind(string id, string? query)
    {
        CheckEditorRequest(id);
        if (query is not null)
        {
            Utf8.Validate(query, nameof(query));
            if (Utf8.Encoding.GetByteCount(query) > 255)
                throw new ArgumentException("The native find query holds at most 255 UTF-8 bytes.", nameof(query));
        }
        editorRequests[id] = new(EditorOperation.Find, Query: query);
    }

    private void CheckEditorRequest(string id)
    {
        CheckOpen();
        if (inFrame) throw new InvalidOperationException("Queue editor requests between frames.");
        ArgumentException.ThrowIfNullOrEmpty(id);
        Utf8.Validate(id, nameof(id));
    }

    private int ApplyEditorRequest(string id, string text, int flags)
    {
        if (renderingResize || !editorRequests.Remove(id, out var request) || (flags & 1) == 0) return flags;
        string key = Stable(id);
        switch (request.Operation)
        {
            case EditorOperation.GoTo: NativeMethods.CodeEditorGoTo(context, key, request.First, request.Second); break;
            case EditorOperation.Select:
                NativeMethods.CodeEditorSelect(context, key, TextOffset(text, request.First), TextOffset(text, request.Second));
                break;
            case EditorOperation.Find:
                NativeMethods.CodeEditorFind(context, key, request.Query);
                if (request.Query is null) flags |= 16; // Return focus to the editor after removing its find field.
                break;
        }
        return flags;
    }

    private static int TextOffset(string text, int offset)
    {
        if (offset < 0 || offset > text.Length || offset > 0 && offset < text.Length &&
            char.IsHighSurrogate(text[offset - 1]) && char.IsLowSurrogate(text[offset]))
            throw new ArgumentOutOfRangeException(nameof(offset), "Use a UTF-16 Unicode scalar boundary in the current text.");
        return Utf8.Encoding.GetByteCount(text.AsSpan(0, offset));
    }
}
