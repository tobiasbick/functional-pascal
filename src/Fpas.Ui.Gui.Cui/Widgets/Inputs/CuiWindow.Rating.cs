using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native CUI whole-star rating with a model-owned value.</summary>
    /// <param name="id">Stable rating identity.</param>
    /// <param name="value">Current rating, or zero for no selection.</param>
    /// <param name="maximum">Number of stars from one through sixteen.</param>
    /// <param name="enabled">Whether the rating accepts pointer input.</param>
    /// <returns>A proposed rating, or minus one when unchanged.</returns>
    public int Rating(string id, int value, int maximum, bool enabled)
    {
        CheckFrame();
        ArgumentOutOfRangeException.ThrowIfLessThan(maximum, 1);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(maximum, 16);
        ArgumentOutOfRangeException.ThrowIfNegative(value);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(value, maximum);
        return NativeMethods.Rating(context, Stable(id), value, maximum, enabled ? 1 : 0);
    }
}
