using System.Globalization;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's ruler slider and exposes its native tape.</summary>
    /// <param name="id">Stable control identity.</param>
    /// <param name="label">Literal caption.</param>
    /// <param name="value">Model-owned value.</param>
    /// <param name="minimum">Finite inclusive lower bound.</param>
    /// <param name="maximum">Finite inclusive upper bound.</param>
    /// <param name="step">Positive value between ticks.</param>
    /// <param name="majorEvery">Ticks between labels and steps per Shift+arrow.</param>
    /// <param name="spacing">Logical points between ticks.</param>
    /// <param name="width">Logical width, or zero to fill.</param>
    /// <param name="decimalPlaces">Precision from zero through six.</param>
    /// <param name="unit">Optional unit caption.</param>
    /// <param name="enabled">Whether the tape accepts input.</param>
    /// <param name="focus">Whether to request tape focus.</param>
    /// <param name="track">Whether to publish the native tape's bounds.</param>
    /// <returns>The proposed value, or the model value if unchanged.</returns>
    public float RulerSlider(string id, string label, float value, float minimum, float maximum,
        float step, int majorEvery, float spacing, float width, int decimalPlaces, string unit,
        bool enabled, bool focus, bool track)
    {
        CheckFrame();
        ValidateSlider(label, value, value, minimum, maximum);
        Utf8.Validate(unit, nameof(unit));
        // Above this range a float tick index can no longer advance by one during paint.
        if (!float.IsFinite(step * 60) || step <= 0 || (maximum - minimum) / step > 8_388_608
            || !float.IsFinite(minimum - step * 3) || !float.IsFinite(maximum + step * 3))
            throw new ArgumentOutOfRangeException(nameof(step));
        if (majorEvery < 1 || !float.IsFinite(majorEvery * step))
            throw new ArgumentOutOfRangeException(nameof(majorEvery));
        if (!float.IsFinite(spacing) || spacing < 1) throw new ArgumentOutOfRangeException(nameof(spacing));
        if (!float.IsFinite(width) || width < 0 || width / spacing > 8_388_608)
            throw new ArgumentOutOfRangeException(nameof(width));

        NativeMethods.RulerSlider(context, Stable(id), label, ref value, minimum, maximum, step, majorEvery,
            spacing, width, SliderNumberFormat(decimalPlaces), Optional(unit), enabled ? 1 : 0,
            focus ? 1 : 0, out int node);
        if (track) TrackControl(id, node);
        return value;
    }

    private static string SliderNumberFormat(int decimalPlaces, string prefix = "", string suffix = "", bool showSign = false)
    {
        if (decimalPlaces is < 0 or > 6) throw new ArgumentOutOfRangeException(nameof(decimalPlaces));
        return prefix.Replace("%", "%%", StringComparison.Ordinal)
            + (showSign ? "%+." : "%.") + decimalPlaces.ToString(CultureInfo.InvariantCulture) + "f"
            + suffix.Replace("%", "%%", StringComparison.Ordinal);
    }
}
