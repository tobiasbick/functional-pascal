using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's 5×7 dot-matrix text.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="text">Literal display text.</param>
    /// <param name="pitch">Distance between dots, or zero.</param>
    /// <param name="color">Native theme or explicit color.</param>
    /// <param name="align">Native text alignment.</param>
    /// <param name="kind">Encoded width extent.</param>
    /// <param name="width">Width or percentage.</param>
    /// <param name="flags">Accent, no dim dots and marquee flags.</param>
    public void DotDisplay(string id, string text, float pitch, CuiChartColor color, int align,
        int kind, float width, int flags)
    {
        CheckFrame();
        Utf8.Validate(text, nameof(text));
        CheckFeedbackNumber(pitch, nameof(pitch));
        CheckExtent(kind, width, nameof(width));
        if (align is < 0 or > 2 || color.Palette is < -1 or > 6) throw new ArgumentOutOfRangeException(nameof(align));
        NativeMethods.DotDisplay(context, Stable(id), text, pitch, color.Palette, color.Rgba, align, kind, width, flags);
    }
}
