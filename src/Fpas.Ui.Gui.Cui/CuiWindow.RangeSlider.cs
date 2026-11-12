using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws a native CUI range slider with two controlled endpoints.</summary>
    /// <param name="id">Stable control identity.</param>
    /// <param name="label">Literal caption.</param>
    /// <param name="lower">Model-owned lower selected value.</param>
    /// <param name="upper">Model-owned upper selected value.</param>
    /// <param name="minimum">Finite inclusive lower bound.</param>
    /// <param name="maximum">Finite inclusive upper bound.</param>
    /// <param name="enabled">Whether the track accepts input.</param>
    /// <param name="focus">Whether to request focus on the track.</param>
    /// <param name="trackingId">Optional identity for inspecting the track.</param>
    /// <param name="proposedLower">Lower endpoint proposed by the user, or the original value.</param>
    /// <param name="proposedUpper">Upper endpoint proposed by the user, or the original value.</param>
    /// <returns>Whether pointer or keyboard input proposed different endpoints.</returns>
    public bool RangeSlider(string id, string label, float lower, float upper, float minimum, float maximum,
        bool enabled, bool focus, string? trackingId, out float proposedLower, out float proposedUpper)
    {
        CheckFrame();
        ValidateSlider(label, lower, upper, minimum, maximum);
        proposedLower = lower;
        proposedUpper = upper;
        bool changed = NativeMethods.RangeSlider(context, Stable(id), label, ref proposedLower,
            ref proposedUpper, minimum, maximum, enabled ? 1 : 0, focus ? 1 : 0, out int track) != 0;
        if (trackingId is not null) TrackControl(trackingId, track);
        return changed;
    }
}
