// [Documentation](../../../../docs/ui/gui/motion/tab-bar.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native compact tab bar.</summary>
    /// <param name="id">Stable tab bar identity.</param>
    /// <param name="icons">Glyphs in tab order. Null uses a circle and an empty string omits the glyph.</param>
    /// <param name="labels">Literal captions. Null omits the caption row.</param>
    /// <param name="badges">Badge counts in tab order.</param>
    /// <param name="selected">Current selected index, or minus one for no selection.</param>
    /// <param name="widthKind">Encoded width extent.</param>
    /// <param name="width">Logical width or percentage.</param>
    /// <param name="flags">Label mode and enabled bits.</param>
    /// <param name="focus">Tab to focus or minus one.</param>
    /// <param name="part">Optional snapshot identity by tab index.</param>
    /// <returns>The native selection proposal.</returns>
    public unsafe int TabBar(string id, string?[] icons, string?[] labels, int[] badges, int selected,
        int widthKind, float width, int flags, int focus, Func<int, string>? part)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        if (icons.Length != labels.Length || labels.Length != badges.Length || selected < -1 || selected >= labels.Length)
            throw new ArgumentException("Expected aligned tab data and a valid selection.");
        for (int i = 0; i < labels.Length; ++i)
        {
            if (badges[i] < 0) throw new ArgumentOutOfRangeException(nameof(badges));
            if (labels[i] is { } label) Utf8.Validate(label, nameof(labels));
            if (icons[i] is { } icon) Utf8.Validate(icon, nameof(icons));
        }
        var nodes = new int[labels.Length];
        fixed (int* b = badges, n = nodes)
            NativeMethods.TabBar(context, Stable(id), icons, labels, b, labels.Length,
                ref selected, widthKind, width, flags, focus, n);
        if (part is not null)
            for (int i = 0; i < nodes.Length; ++i) TrackControl(part(i), nodes[i]);
        return selected;
    }
}
