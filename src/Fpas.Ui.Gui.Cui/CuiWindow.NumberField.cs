using System.Globalization;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a bounded native number field with step buttons.</summary>
    /// <param name="id">Stable editor identity.</param>
    /// <param name="value">Current finite model value.</param>
    /// <param name="minimum">Inclusive finite lower bound.</param>
    /// <param name="maximum">Inclusive finite upper bound.</param>
    /// <param name="label">Optional caption above the editor.</param>
    /// <param name="suffix">Optional unit text before the buttons.</param>
    /// <param name="step">Positive finite increment.</param>
    /// <param name="decimalPlaces">Display precision between zero and fifteen.</param>
    /// <param name="enabled">Whether the field accepts input.</param>
    /// <param name="focus">Whether to request editor focus.</param>
    /// <param name="decreaseControl">Decrease button's tracking ID, or null to skip tracking.</param>
    /// <param name="increaseControl">Increase button's tracking ID when tracking is enabled.</param>
    /// <returns>The resulting bounded value.</returns>
    public unsafe double NumberField(string id, double value, double minimum, double maximum, string label,
        string suffix, double step, int decimalPlaces, bool enabled, bool focus, string? decreaseControl, string increaseControl)
    {
        CheckFrame();
        string stable = Stable(id);
        Utf8.Validate(label, nameof(label));
        Utf8.Validate(suffix, nameof(suffix));
        if (!double.IsFinite(minimum) || !double.IsFinite(maximum) || !double.IsFinite(value)
            || minimum > maximum || value < minimum || value > maximum)
            throw new ArgumentException("Expected a finite value within finite ordered bounds.", nameof(value));
        if (!double.IsFinite(step) || step <= 0) throw new ArgumentOutOfRangeException(nameof(step));
        if (decimalPlaces is < 0 or > 15) throw new ArgumentOutOfRangeException(nameof(decimalPlaces));

        string format = "F" + decimalPlaces.ToString(CultureInfo.InvariantCulture);
        Span<char> capacity = stackalloc char[63];
        if (!minimum.TryFormat(capacity, out _, format, CultureInfo.InvariantCulture) ||
            !maximum.TryFormat(capacity, out _, format, CultureInfo.InvariantCulture))
            throw new ArgumentException("The bounds exceed the native number display capacity.");

        int* nodes = stackalloc int[3];
        NativeMethods.NumberField(context, stable, ref value, minimum, maximum, Optional(label), Optional(suffix),
            step, decimalPlaces, enabled ? 1 : 0, focus ? 1 : 0, nodes);
        if (decreaseControl is not null)
        {
            TrackControl(id, nodes[0]);
            TrackControl(decreaseControl, nodes[1]);
            TrackControl(increaseControl, nodes[2]);
        }
        return value;
    }
}
