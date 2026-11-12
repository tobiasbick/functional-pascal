using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's histogram slider with two controlled endpoints.</summary>
    /// <param name="id">Stable control identity.</param>
    /// <param name="label">Literal caption.</param>
    /// <param name="bins">Nonempty series of finite nonnegative bin heights.</param>
    /// <param name="lower">Model-owned lower selected value.</param>
    /// <param name="upper">Model-owned upper selected value.</param>
    /// <param name="minimum">Finite inclusive lower bound.</param>
    /// <param name="maximum">Finite inclusive upper bound.</param>
    /// <param name="height">Logical bar height, or zero for CUI's default.</param>
    /// <param name="decimalPlaces">Precision from zero through six.</param>
    /// <param name="prefix">Literal text before each endpoint.</param>
    /// <param name="suffix">Literal text after each endpoint.</param>
    /// <param name="enabled">Whether the track accepts input.</param>
    /// <param name="focus">Whether to request track focus.</param>
    /// <param name="track">Whether to publish the track's native bounds.</param>
    /// <param name="proposedLower">Lower proposal, or the model value if unchanged.</param>
    /// <param name="proposedUpper">Upper proposal, or the model value if unchanged.</param>
    /// <returns>Whether input proposed a different interval.</returns>
    public unsafe bool HistogramSlider(string id, string label, float[] bins, float lower, float upper,
        float minimum, float maximum, float height, int decimalPlaces, string prefix, string suffix,
        bool enabled, bool focus, bool track, out float proposedLower, out float proposedUpper)
    {
        CheckFrame();
        ValidateSlider(label, lower, upper, minimum, maximum);
        ArgumentNullException.ThrowIfNull(bins);
        if (bins.Length == 0) throw new ArgumentException("A histogram needs at least one bin.", nameof(bins));
        foreach (float bin in bins)
            if (!float.IsFinite(bin) || bin < 0) throw new ArgumentOutOfRangeException(nameof(bins));
        if (!float.IsFinite(height) || height < 0) throw new ArgumentOutOfRangeException(nameof(height));
        Utf8.Validate(prefix, nameof(prefix));
        Utf8.Validate(suffix, nameof(suffix));
        string format = SliderNumberFormat(decimalPlaces, prefix, suffix);

        proposedLower = lower;
        proposedUpper = upper;
        int node;
        bool changed;
        fixed (float* heights = bins)
            changed = NativeMethods.HistogramSlider(context, Stable(id), label, heights, bins.Length,
                ref proposedLower, ref proposedUpper, minimum, maximum, height, format, enabled ? 1 : 0,
                focus ? 1 : 0, out node) != 0;
        if (track) TrackControl(id, node);
        return changed;
    }
}
