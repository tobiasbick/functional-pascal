using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's audio knob.</summary>
    /// <param name="id">Stable control identity.</param>
    /// <param name="label">Literal caption.</param>
    /// <param name="value">Model-owned value.</param>
    /// <param name="minimum">Finite inclusive lower bound.</param>
    /// <param name="maximum">Finite inclusive upper bound.</param>
    /// <param name="step">Increment, or zero for continuous motion.</param>
    /// <param name="reset">Value to restore, clamped to the bounds.</param>
    /// <param name="decimalPlaces">Precision from zero through six.</param>
    /// <param name="unit">Optional unit caption.</param>
    /// <param name="hideValue">Whether to hide the numeric value.</param>
    /// <param name="size">Logical diameter, or zero for the native default.</param>
    /// <param name="bipolar">Whether to draw the arc from the middle.</param>
    /// <param name="logScale">Whether to use logarithmic rotation.</param>
    /// <param name="showSign">Whether to show the leading plus sign.</param>
    /// <param name="rgba">Packed RGBA arc color, or zero for the default.</param>
    /// <param name="enabled">Whether the control accepts input.</param>
    /// <param name="focus">Whether to request native focus.</param>
    /// <param name="track">Whether to publish the control bounds.</param>
    /// <returns>The input proposal, or the model value if unchanged.</returns>
    public float Knob(string id, string label, float value, float minimum, float maximum,
        float step, float reset, int decimalPlaces, string unit, bool hideValue,
        float size, bool bipolar, bool logScale, bool showSign, uint rgba, bool enabled, bool focus, bool track)
    {
        CheckFrame();
        ValidateSlider(label, value, value, minimum, maximum);
        ValidateAudioDial(step, reset, unit, maximum - minimum);
        if (!ValidAudioSize(size)) throw new ArgumentOutOfRangeException(nameof(size));
        if (logScale && (minimum <= 0 || !float.IsFinite(maximum / minimum)))
            throw new ArgumentOutOfRangeException(nameof(minimum));

        NativeMethods.Knob(context, Stable(id), label, ref value, minimum, maximum, step, reset,
            SliderNumberFormat(decimalPlaces, showSign: showSign), Optional(unit), hideValue ? 1 : 0,
            size, bipolar ? 1 : 0, logScale ? 1 : 0, rgba, enabled ? 1 : 0, focus ? 1 : 0, out int node);
        if (track) TrackControl(id, node);
        return value;
    }
}
