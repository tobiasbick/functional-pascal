using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native CUI slider with a controlled value.</summary>
    /// <param name="id">Stable control identity.</param>
    /// <param name="label">Literal caption.</param>
    /// <param name="value">Model-owned value.</param>
    /// <param name="minimum">Finite inclusive lower bound.</param>
    /// <param name="maximum">Finite inclusive upper bound.</param>
    /// <param name="step">Positive step, or zero for continuous adjustment.</param>
    /// <param name="showValue">Whether to show the numeric value.</param>
    /// <param name="decimalPlaces">Precision from zero through six.</param>
    /// <param name="percent">Whether to append a percent sign.</param>
    /// <param name="enabled">Whether the track accepts input.</param>
    /// <param name="focus">Whether to request focus on the track.</param>
    /// <param name="trackingId">Optional identity for inspecting the track.</param>
    /// <param name="proposal">The proposed value, or the original value if unchanged.</param>
    /// <returns>Whether input proposed a different value.</returns>
    public bool Slider(string id, string label, float value, float minimum, float maximum, float step,
        bool showValue, int decimalPlaces, bool percent, bool enabled, bool focus, string? trackingId,
        out float proposal)
    {
        CheckFrame();
        ValidateSlider(label, value, value, minimum, maximum);
        if (!float.IsFinite(step) || step < 0 || (step > 0 && !float.IsFinite((maximum - minimum) / step)))
            throw new ArgumentOutOfRangeException(nameof(step));
        if (decimalPlaces is < 0 or > 6) throw new ArgumentOutOfRangeException(nameof(decimalPlaces));
        proposal = value;
        bool changed = NativeMethods.Slider(context, Stable(id), label, ref proposal, minimum, maximum,
            step, showValue ? 1 : 0, decimalPlaces, percent ? 1 : 0, enabled ? 1 : 0, focus ? 1 : 0,
            out int track) != 0;
        if (trackingId is not null) TrackControl(trackingId, track);
        return changed;
    }

    private static void ValidateSlider(string label, float lower, float upper, float minimum, float maximum)
    {
        Utf8.Validate(label, nameof(label));
        if (string.IsNullOrWhiteSpace(label)) throw new ArgumentException("A slider needs a caption.", nameof(label));
        if (!float.IsFinite(minimum) || !float.IsFinite(maximum) ||
            !float.IsFinite(maximum - minimum) || minimum >= maximum)
            throw new ArgumentException("Slider bounds must be finite and increasing.");
        if (!float.IsFinite(lower) || !float.IsFinite(upper) || lower < minimum || upper > maximum || lower > upper)
            throw new ArgumentException("Slider values must be finite and ordered within their bounds.");
    }
}
