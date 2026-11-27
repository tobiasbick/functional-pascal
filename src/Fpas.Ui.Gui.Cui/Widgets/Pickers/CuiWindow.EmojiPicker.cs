// [Documentation](../../../../docs/ui/gui/smart-inputs/emoji-picker.md)
using Fpas.Ui.Gui.Cui.Widgets.Pickers;
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's searchable emoji picker and copies its result for managed callers.</summary>
    /// <param name="id">Stable picker identity.</param>
    /// <param name="recent">Recent emoji, newest first, separated by spaces. Null uses native history.</param>
    /// <param name="recentCapacity">Recent-history buffer capacity, including its terminator.</param>
    /// <param name="tone">Skin tone from zero through five, or null for native tone state.</param>
    /// <param name="widthKind">Encoded width extent kind.</param>
    /// <param name="width">Logical width or percentage for the selected extent.</param>
    /// <param name="height">Logical grid height, or zero for the native default.</param>
    /// <param name="size">Logical glyph size, or zero for the native default.</param>
    /// <param name="placeholder">Optional search placeholder.</param>
    /// <param name="emptyText">Optional empty results label.</param>
    /// <param name="hint">Optional preview hint.</param>
    /// <param name="groupNames">Ten captions in native group order, or null for default captions.</param>
    /// <param name="search">Whether to show the search field.</param>
    /// <param name="preview">Whether to show the preview.</param>
    /// <param name="toneSelector">Whether to show skin tone controls.</param>
    /// <param name="enabled">Whether input is accepted.</param>
    /// <param name="focus">Zero for no request, one for search or two for the grid.</param>
    /// <param name="trackPart">Maps native part indices to tracking IDs, or null.</param>
    /// <returns>The native selection and updated history and tone.</returns>
    public unsafe CuiEmojiPickerResult EmojiPicker(string id, string? recent, int recentCapacity, int? tone,
        int widthKind, float width, float height, float size, string? placeholder, string? emptyText,
        string? hint, string[]? groupNames, bool search, bool preview,
        bool toneSelector, bool enabled, int focus, Func<int, string>? trackPart)
    {
        CheckFrame();
        if (placeholder is not null) Utf8.Validate(placeholder, nameof(placeholder));
        if (emptyText is not null) Utf8.Validate(emptyText, nameof(emptyText));
        if (hint is not null) Utf8.Validate(hint, nameof(hint));
        if (groupNames is not null)
        {
            if (groupNames.Length != 10) throw new ArgumentException("Supply ten group captions.", nameof(groupNames));
            foreach (string name in groupNames) Utf8.Validate(name, nameof(groupNames));
        }
        if (tone is < 0 or > 5) throw new ArgumentOutOfRangeException(nameof(tone));
        CheckExtent(widthKind, width, nameof(width));
        if (!float.IsFinite(height) || height < 0) throw new ArgumentOutOfRangeException(nameof(height));
        if (!float.IsFinite(size) || size < 0) throw new ArgumentOutOfRangeException(nameof(size));
        if (focus is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(focus));
        if (recent is not null)
        {
            Utf8.Validate(recent, nameof(recent));
            ArgumentOutOfRangeException.ThrowIfNegativeOrZero(recentCapacity);
            if (Utf8.Encoding.GetByteCount(recent) >= recentCapacity) throw new ArgumentOutOfRangeException(nameof(recent));
        }
        Span<byte> history = recent is null ? [] : recentCapacity <= 256 ? stackalloc byte[recentCapacity] : new byte[recentCapacity];
        if (recent is not null) history[Utf8.Encoding.GetBytes(recent, history)] = 0;
        int skinTone = tone ?? 0;
        int* nodes = stackalloc int[20];
        nint selection;
        fixed (byte* bytes = history)
            selection = NativeMethods.EmojiPicker(context, Stable(id), bytes, (nuint)history.Length,
                tone.HasValue ? &skinTone : null, widthKind, width, height, size,
                placeholder, emptyText, hint, groupNames, search ? 1 : 0,
                preview ? 1 : 0, toneSelector ? 1 : 0, enabled ? 1 : 0, focus, trackPart is null ? null : nodes);
        string? emoji = Marshal.PtrToStringUTF8(selection);
        if (emoji is not null && recent is not null)
        {
            var updated = history[..history.IndexOf((byte)0)];
            if (System.Text.Unicode.Utf8.IsValid(updated)) recent = Utf8.Encoding.GetString(updated);
        }
        if (tone.HasValue) tone = skinTone;
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
