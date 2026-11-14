using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native toggle tile.</summary>
    /// <param name="id">Stable tile identity.</param>
    /// <param name="label">Literal title.</param>
    /// <param name="icon">Optional glyph.</param>
    /// <param name="detail">Optional state description.</param>
    /// <param name="on">Current toggle value.</param>
    /// <param name="widthKind">Encoded width extent.</param>
    /// <param name="width">Logical width or percentage.</param>
    /// <param name="color">Active color.</param>
    /// <param name="flags">Wide, enabled and focus bits.</param>
    /// <returns>The native toggle proposal.</returns>
    public bool ControlTile(string id, string label, string icon, string detail, bool on,
        int widthKind, float width, CuiChartColor color, int flags)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(icon, nameof(icon));
        Utf8.Validate(detail, nameof(detail));
        CheckExtent(widthKind, width, nameof(width));
        int active = on ? 1 : 0;
        NativeMethods.ControlTile(context, Stable(id), label, Optional(icon), Optional(detail), ref active,
            widthKind, width, color.Palette, color.Rgba, flags);
        return active != 0;
    }
}
