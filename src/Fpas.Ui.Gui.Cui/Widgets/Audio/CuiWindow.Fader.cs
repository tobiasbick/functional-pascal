using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's audio fader.</summary>
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
    /// <param name="length">Logical track length, or zero for the native default.</param>
    /// <param name="horizontal">Whether the track runs from left to right.</param>
    /// <param name="ticks">Up to 32 tick values within the range.</param>
    /// <param name="levels">Up to eight optional linear channel peaks.</param>
    /// <param name="enabled">Whether the control accepts input.</param>
    /// <param name="focus">Whether to request native focus.</param>
    /// <param name="track">Whether to publish the control bounds.</param>
    /// <returns>The input proposal, or the model value if unchanged.</returns>
    public unsafe float Fader(string id, string label, float value, float minimum, float maximum,
        float step, float reset, int decimalPlaces, string unit, bool hideValue,
        float length, bool horizontal, float[] ticks, float[] levels, bool enabled, bool focus, bool track)
    {
        CheckFrame();
        ValidateSlider(label, value, value, minimum, maximum);
        ValidateAudioDial(step, reset, unit, maximum - minimum);
        if (!ValidAudioSize(length)) throw new ArgumentOutOfRangeException(nameof(length));
        ArgumentNullException.ThrowIfNull(ticks);
        if (ticks.Length > 32) throw new ArgumentOutOfRangeException(nameof(ticks));
        foreach (float tick in ticks)
            if (!float.IsFinite(tick) || tick < minimum || tick > maximum)
                throw new ArgumentOutOfRangeException(nameof(ticks));
        ValidateAudioLevels(levels, minimumChannels: 0);

        fixed (float* marks = ticks, peaks = levels)
        {
            NativeMethods.Fader(context, Stable(id), label, ref value, minimum, maximum, step, reset,
                SliderNumberFormat(decimalPlaces), Optional(unit), hideValue ? 1 : 0, length,
                horizontal ? 1 : 0, marks, ticks.Length, peaks, levels.Length, enabled ? 1 : 0,
                focus ? 1 : 0, out int node);
            if (track) TrackControl(id, node);
        }
        return value;
    }
}
