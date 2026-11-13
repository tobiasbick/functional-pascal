using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native instrument readout.</summary>
    /// <param name="id">Stable inspection identity.</param>
    /// <param name="label">Label above the value.</param>
    /// <param name="value">Literal value.</param>
    /// <param name="unit">Optional unit.</param>
    /// <param name="detail">Optional context.</param>
    /// <param name="size">Native control size.</param>
    /// <param name="tone">Native semantic value tone.</param>
    /// <param name="accent">Use the active theme accent.</param>
    /// <param name="align">Native text alignment.</param>
    /// <param name="kind">Encoded width extent.</param>
    /// <param name="width">Width or percentage.</param>
    public void Readout(string id, string label, string value, string unit, string detail, int size,
        int tone, bool accent, int align, int kind, float width)
    {
        CheckFrame();
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(value, nameof(value));
        Utf8.Validate(unit, nameof(unit));
        Utf8.Validate(detail, nameof(detail));
        CheckExtent(kind, width, nameof(width));
        CheckTone(tone);
        if (size is < 0 or > 2 || align is < 0 or > 2) throw new ArgumentOutOfRangeException(nameof(size));
        NativeMethods.Readout(context, Stable(id), label, value, Optional(unit), Optional(detail),
            size, tone, accent ? 1 : 0, align, kind, width);
    }
}
