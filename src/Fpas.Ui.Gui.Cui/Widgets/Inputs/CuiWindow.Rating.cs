// [Documentation](../../../../docs/ui/gui/selection/rating.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native CUI whole-star rating with a model-owned value.</summary>
    /// <param name="id">Stable rating identity.</param>
    /// <param name="value">Current rating, or zero for no selection.</param>
    /// <param name="maximum">Number of stars. CUI uses five for nonpositive values and caps the count at sixteen.</param>
    /// <param name="enabled">Whether the rating accepts pointer input.</param>
    /// <param name="starSize">The positive logical edge length of each star's hit area.</param>
    /// <returns>A proposed rating, or minus one when unchanged.</returns>
    public int Rating(string id, int value, int maximum, bool enabled, float starSize = 22f)
    {
        CheckFrame();
        int count = maximum <= 0 ? 5 : Math.Min(maximum, 16);
        ArgumentOutOfRangeException.ThrowIfNegative(value);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(value, count);
        if (!float.IsFinite(starSize) || starSize <= 0)
            throw new ArgumentOutOfRangeException(nameof(starSize));
        return NativeMethods.Rating(context, Stable(id), value, maximum, enabled ? 1 : 0, starSize);
    }
}
