using Fpas.Ui.Gui.Cui.Widgets.Inputs;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's inline editor with a retained draft and model-owned commits.</summary>
    /// <param name="id">Stable application identity.</param>
    /// <param name="value">Confirmed model text.</param>
    /// <param name="placeholder">Literal empty-state text.</param>
    /// <param name="font">Medium one, display two or mono three.</param>
    /// <param name="size">Logical font size, zero for the native body size.</param>
    /// <param name="capacity">Maximum UTF-8 bytes, excluding terminator, at most 511.</param>
    /// <param name="widthKind">Content zero, fill one, fixed two or percentage three.</param>
    /// <param name="width">Logical extent or percentage.</param>
    /// <param name="enabled">Whether input is accepted.</param>
    /// <param name="focus">Whether to focus the current native interaction mode.</param>
    /// <param name="track">Whether to retain and inspect the control.</param>
    /// <returns>The commit proposal and native draft status.</returns>
    public unsafe CuiInlineEditResult InlineEdit(string id, string value, string placeholder,
        int font, float size, int capacity, int widthKind, float width, bool enabled, bool focus, bool track)
    {
        CheckFrame();
        string identity = Stable(id);
        Utf8.Validate(value, nameof(value)); Utf8.Validate(placeholder, nameof(placeholder));
        CheckSmartWidth(widthKind, width);
        if (font is < 1 or > 3 || !float.IsFinite(size) || size < 0)
            throw new ArgumentOutOfRangeException(nameof(font));
        if (capacity is < 1 or > 511 || Utf8.Encoding.GetByteCount(value) > capacity ||
            value.Contains('\r') || value.Contains('\n'))
            throw new ArgumentOutOfRangeException(nameof(capacity));
        Span<byte> text = stackalloc byte[capacity + 1];
        int count = Utf8.Encoding.GetBytes(value, text); text[count] = 0;
        fixed (byte* buffer = text)
        {
            int flags = NativeMethods.InlineEdit(context, identity, buffer, (nuint)text.Length,
                Optional(placeholder), font, size, widthKind, width, enabled ? 1 : 0, focus ? 1 : 0,
                out int editing, out int node, out uint key);
            if (track) TrackControl(id, node, key);
            string draft = Utf8.Encoding.GetString(text[..text.IndexOf((byte)0)]);
            return new((flags & 1) != 0 ? draft : value, draft, editing != 0);
        }
    }
}
