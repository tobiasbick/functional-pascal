// [Documentation](../../../../docs/ui/gui/social-and-media/counter.md)
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native rolling counter.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="value">Current finite value.</param>
    /// <param name="format">Optional safe numeric format.</param>
    /// <param name="size">Text size, or zero for the default.</param>
    /// <param name="color">Native text color.</param>
    /// <param name="group">Whether to group thousands.</param>
    /// <param name="font">Native CUI font role. CUI uses Display for zero.</param>
    public unsafe void Counter(string id, double value, CuiChartFormat? format, float size, CuiChartColor color,
        bool group, int font = 2)
    {
        CheckFrame();
        if (!double.IsFinite(value)) throw new ArgumentOutOfRangeException(nameof(value));
        CheckFeedbackNumber(size, nameof(size));
        if (font is < 0 or > 4) throw new ArgumentOutOfRangeException(nameof(font));
        CheckChartColor(color);
        CheckChartFormat(format);
        using var memory = new ChartMemory();
        NativeMethods.Counter(context, Stable(id), value, memory.Format(format),
            size, color.Palette, color.Rgba, group ? 1 : 0, font);
    }
}
