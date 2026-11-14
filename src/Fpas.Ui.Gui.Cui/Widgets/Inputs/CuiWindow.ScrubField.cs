using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's draggable numeric field and native typing mode.</summary>
    /// <param name="id">Stable identity independent of the caption.</param>
    /// <param name="label">Literal caption.</param>
    /// <param name="value">Finite model value within the bounds.</param>
    /// <param name="minimum">Finite lower bound.</param>
    /// <param name="maximum">Finite upper bound greater than the minimum.</param>
    /// <param name="step">Positive increment or zero for one percent of the range.</param>
    /// <param name="decimalPlaces">Precision from zero through six.</param>
    /// <param name="suffix">Literal unit suffix.</param>
    /// <param name="icon">Native glyph replacing the caption, or empty.</param>
    /// <param name="widthKind">Content zero, fill one, fixed two or percentage three.</param>
    /// <param name="width">Logical extent or percentage.</param>
    /// <param name="enabled">Whether input is accepted.</param>
    /// <param name="focus">Whether to request native focus.</param>
    /// <param name="track">Whether to retain and inspect the control.</param>
    /// <returns>The proposed model value.</returns>
    public float ScrubField(string id, string label, float value, float minimum, float maximum,
        float step, int decimalPlaces, string suffix, string icon, int widthKind, float width,
        bool enabled, bool focus, bool track)
    {
        CheckFrame(); string identity = Stable(id);
        Utf8.Validate(label, nameof(label));
        if (!float.IsFinite(value) || !float.IsFinite(minimum) || !float.IsFinite(maximum) ||
            minimum >= maximum || value < minimum || value > maximum)
            throw new ArgumentException("A scrub needs a finite value within increasing finite bounds.");
        Utf8.Validate(suffix, nameof(suffix)); Utf8.Validate(icon, nameof(icon));
        CheckSmartWidth(widthKind, width);
        if (!float.IsFinite(maximum - minimum) || !float.IsFinite(step) || step < 0 ||
            (step > 0 && !float.IsFinite((maximum - minimum) / step)))
            throw new ArgumentOutOfRangeException(nameof(step));
        NativeMethods.ScrubField(context, identity, label, ref value, minimum, maximum, step,
            SliderNumberFormat(decimalPlaces), Optional(suffix), Optional(icon), widthKind, width,
            enabled ? 1 : 0, focus ? 1 : 0, out int node, out uint key);
        if (track) TrackControl(id, node, key);
        return value;
    }
}
