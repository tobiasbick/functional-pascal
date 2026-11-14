using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native compact tab bar.</summary>
    /// <param name="id">Stable tab bar identity.</param>
    /// <param name="icons">Glyphs in tab order.</param>
    /// <param name="labels">Literal captions in tab order.</param>
    /// <param name="badges">Badge counts in tab order.</param>
    /// <param name="selected">Current selected index.</param>
    /// <param name="widthKind">Encoded width extent.</param>
    /// <param name="width">Logical width or percentage.</param>
    /// <param name="flags">Label mode and enabled bits.</param>
    /// <param name="focus">Tab to focus or minus one.</param>
    /// <param name="part">Optional snapshot identity by tab index.</param>
    /// <returns>The native selection proposal.</returns>
    public unsafe int TabBar(string id, string[] icons, string[] labels, int[] badges, int selected,
        int widthKind, float width, int flags, int focus, Func<int, string>? part)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        if (icons.Length != labels.Length || labels.Length != badges.Length || selected < 0 || selected >= labels.Length)
            throw new ArgumentException("Expected aligned tab data and a valid selection.");
        for (int i = 0; i < labels.Length; ++i)
        {
            Utf8.Validate(labels[i], nameof(labels));
            Utf8.Validate(icons[i], nameof(icons));
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
