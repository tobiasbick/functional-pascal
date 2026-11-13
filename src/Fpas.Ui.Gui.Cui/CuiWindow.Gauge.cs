using Fpas.Ui.Gui.Cui.Interop;
namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's animated gauge with an identity independent of its literal caption.</summary>
    /// <param name="id">Stable inspection identity.</param>
    /// <param name="label">Literal caption.</param>
    /// <param name="value">Fraction from zero through one.</param>
    /// <param name="size">Logical diameter; zero selects the native default.</param>
    /// <param name="track">Retain the public inspection identity.</param>
    public void Gauge(string id, string label, float value, float size = 0, bool track = true)
    {
        CheckFrame(); string stable = Stable(id); Utf8.Validate(label, nameof(label));
        if (!float.IsFinite(value) || value < 0 || value > 1 || !float.IsFinite(size) || size < 0)
            throw new ArgumentOutOfRangeException(nameof(value));
        int node = NativeMethods.Gauge(context, stable, label, value, size);
        if (track) TrackControl(id, node);
    }
}
