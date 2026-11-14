using Fpas.Ui.Gui.Cui.Widgets.Pickers;
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's searchable emoji picker and copies its result for managed callers.</summary>
    /// <param name="id">Stable picker identity.</param>
    /// <param name="recent">Recent emoji, newest first, separated by spaces. Up to 255 UTF-8 bytes.</param>
    /// <param name="tone">Skin tone from zero through five.</param>
    /// <param name="width">Logical width, or zero for the native default.</param>
    /// <param name="height">Logical grid height, or zero for the native default.</param>
    /// <param name="size">Logical glyph size, or zero for the native default.</param>
    /// <param name="placeholder">Optional search placeholder.</param>
    /// <param name="emptyText">Optional empty results label.</param>
    /// <param name="hint">Optional preview hint.</param>
    /// <param name="search">Whether to show the search field.</param>
    /// <param name="preview">Whether to show the preview.</param>
    /// <param name="toneSelector">Whether to show skin tone controls.</param>
    /// <param name="enabled">Whether input is accepted.</param>
    /// <param name="focus">Zero for no request, one for search or two for the grid.</param>
    /// <param name="trackPart">Maps native part indices to tracking IDs, or null.</param>
    /// <returns>The native selection and updated history and tone.</returns>
    public unsafe CuiEmojiPickerResult EmojiPicker(string id, string recent, int tone, float width, float height,
        float size, string placeholder, string emptyText, string hint, bool search, bool preview,
        bool toneSelector, bool enabled, int focus, Func<int, string>? trackPart)
    {
        CheckFrame();
        Utf8.Validate(recent, nameof(recent));
        Utf8.Validate(placeholder, nameof(placeholder));
        Utf8.Validate(emptyText, nameof(emptyText));
        Utf8.Validate(hint, nameof(hint));
        if (tone is < 0 or > 5) throw new ArgumentOutOfRangeException(nameof(tone));
        if (!float.IsFinite(width) || width < 0) throw new ArgumentOutOfRangeException(nameof(width));
        if (!float.IsFinite(height) || height < 0) throw new ArgumentOutOfRangeException(nameof(height));
        if (!float.IsFinite(size) || size < 0) throw new ArgumentOutOfRangeException(nameof(size));
        if (focus is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(focus));
        Span<byte> history = stackalloc byte[256];
        if (Utf8.Encoding.GetByteCount(recent) >= history.Length) throw new ArgumentOutOfRangeException(nameof(recent));
        int length = Utf8.Encoding.GetBytes(recent, history);
        history[length] = 0;
        int* nodes = stackalloc int[20];
        nint selection;
        fixed (byte* bytes = history)
            selection = NativeMethods.EmojiPicker(context, Stable(id), bytes, (nuint)history.Length, ref tone,
                width, height, size, Optional(placeholder), Optional(emptyText), Optional(hint), search ? 1 : 0,
                preview ? 1 : 0, toneSelector ? 1 : 0, enabled ? 1 : 0, focus, trackPart is null ? null : nodes);
        string? emoji = Marshal.PtrToStringUTF8(selection);
        if (selection != 0) recent = Utf8.Encoding.GetString(history[..history.IndexOf((byte)0)]);
        if (trackPart is null) return new(recent, tone, emoji, []);

        TrackControl(id, nodes[0]);
        List<int> visible = [];
        for (int i = 1; i < 20; ++i)
        {
            if (nodes[i] < 0) continue;
            TrackControl(trackPart(i - 1), nodes[i]);
            visible.Add(i - 1);
        }
        return new(recent, tone, emoji, visible.ToArray());
    }
}
