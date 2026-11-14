using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native busy ring or its success and error transition.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="status">Idle, busy, success or error.</param>
    /// <param name="size">Logical diameter, or zero for the default.</param>
    /// <param name="color">Busy ring color.</param>
    /// <param name="keep">Whether to retain the result marker.</param>
    /// <returns>Whether the indicator is visible.</returns>
    public bool StatusSpinner(string id, int status, float size, CuiChartColor color, bool keep)
    {
        CheckFrame();
        if (status is < 0 or > 3) throw new ArgumentOutOfRangeException(nameof(status));
        CheckFeedbackNumber(size, nameof(size));
        CheckChartColor(color);
        return NativeMethods.StatusSpinner(context, Stable(id), status, size, color.Palette,
            color.Rgba, keep ? 1 : 0) != 0;
    }
}
