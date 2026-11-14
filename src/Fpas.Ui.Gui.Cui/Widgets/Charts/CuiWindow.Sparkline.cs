using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's compact line chart.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="values">Finite samples in display order.</param>
    /// <param name="widthKind">Encoded width extent.</param>
    /// <param name="width">Logical width or percentage.</param>
    /// <param name="height">Height in logical points.</param>
    /// <param name="color">Stroke color.</param>
    public unsafe void Sparkline(string id, float[] values, int widthKind, float width, float height, CuiChartColor color)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        CheckFeedbackNumber(height, nameof(height));
        CheckChartColor(color);
        foreach (float value in values)
            if (!float.IsFinite(value)) throw new ArgumentOutOfRangeException(nameof(values));
        fixed (float* samples = values)
            NativeMethods.Sparkline(context, Stable(id), samples, values.Length,
                widthKind, width, height, color.Palette, color.Rgba);
    }
}
