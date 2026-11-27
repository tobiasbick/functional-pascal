// [Documentation](../../../../docs/ui/gui/feedback/status-spinner.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws the native busy ring or its success and error transition.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="status">Idle, busy, success or error.</param>
    /// <param name="size">Logical diameter, or zero for the default.</param>
    /// <param name="color">Optional packed straight-alpha busy ring color.</param>
    /// <param name="keep">Whether to retain the result marker.</param>
    /// <param name="themeColor">Optional one-based busy ring theme token. Use zero for the default.</param>
    /// <returns>Whether the indicator is visible.</returns>
    public bool StatusSpinner(string id, int status, float size, uint? color, bool keep, int themeColor = 0)
    {
        CheckFrame();
        if (status is < 0 or > 3) throw new ArgumentOutOfRangeException(nameof(status));
        CheckFeedbackNumber(size, nameof(size));
        return NativeMethods.StatusSpinner(context, Stable(id), status, size,
            SpinnerColor(color, themeColor, nameof(themeColor)), color.GetValueOrDefault(), keep ? 1 : 0) != 0;
    }
}
