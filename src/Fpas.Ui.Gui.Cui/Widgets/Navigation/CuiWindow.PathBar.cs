using Fpas.Ui.Gui.Cui.Widgets.Navigation;
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a path bar and its native child menus.</summary>
    /// <param name="id">Stable identity.</param>
    /// <param name="segments">Visible path components.</param>
    /// <param name="children">Menu choices per parent segment.</param>
    /// <param name="text">Optional editable path.</param>
    /// <param name="icon">Leading glyph.</param>
    /// <param name="placeholder">Optional edit placeholder.</param>
    /// <param name="size">Native control size.</param>
    /// <param name="widthKind">Native width kind.</param>
    /// <param name="width">Preferred width.</param>
    /// <param name="flags">Input, edit and menu options.</param>
    /// <param name="focus">Requested segment, or minus one.</param>
    /// <param name="part">Maps visible native parts to tracking names.</param>
    /// <returns>Native input proposal.</returns>
    public unsafe CuiPathResult PathBar(string id, string[] segments, string[][] children, string text,
        string icon, string placeholder, int size, int widthKind, float width, int flags, int focus, Func<int, string>? part)
    {
        CheckFrame();
        if (segments.Length > 64 || children.Length > segments.Length || size is < 0 or > 2)
            throw new ArgumentOutOfRangeException(nameof(segments));
        CheckExtent(widthKind, width, nameof(width));
        foreach (string segment in segments) Utf8.Validate(segment, nameof(segments));
        foreach (var menu in children)
            foreach (string item in menu) Utf8.Validate(item, nameof(children));
        Utf8.Validate(text, nameof(text));
        Utf8.Validate(icon, nameof(icon));
        Utf8.Validate(placeholder, nameof(placeholder));
        string stable = Stable(id);
        int count = segments.Length * 2 + 1;
        int* nodes = stackalloc int[count];
        int segmentIndex = NativeMethods.PathBar(context, stable, segments, segments.Length, Optional(text),
            Optional(icon), Optional(placeholder), size, widthKind, width, flags, focus,
            out nint submitted, nodes, out int root, out uint retainedKey);
        if (part is not null) TrackControl(id, root, retainedKey);
        var path = Marshal.PtrToStringUTF8(submitted);
        var parts = new bool[count];
        for (int i = 0; i < count; ++i)
        {
            parts[i] = nodes[i] >= 0;
            if (parts[i] && part is not null) TrackControl(part(i), nodes[i]);
        }
        int parent = -1, chosen = -1;
        if ((flags & 5) == 5) parent = NativeMethods.PathMenuBegin(context, stable);
        if (parent >= 0)
        {
            try
            {
                if (parent < children.Length)
                    for (int i = 0; i < children[parent].Length; ++i)
                        if (NativeMethods.PathMenuItem(context, children[parent][i], i) != 0) chosen = i;
            }
            finally { NativeMethods.PathMenuEnd(context); }
        }
        return new(segmentIndex, path, chosen >= 0 ? parent : -1, chosen, parts);
    }
}
