using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native loader.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="style">Native loader style.</param>
    /// <param name="size">Logical size, or zero.</param>
    /// <param name="color">Semantic or explicit color.</param>
    public void Loader(string id, int style, float size, CuiChartColor color)
    {
        CheckFrame();
        CheckFeedbackNumber(size, nameof(size));
        if (style is < 0 or > 4) throw new ArgumentOutOfRangeException(nameof(style));
        NativeMethods.Loader(context, Stable(id), style, size, color.Palette, color.Rgba);
    }
}
