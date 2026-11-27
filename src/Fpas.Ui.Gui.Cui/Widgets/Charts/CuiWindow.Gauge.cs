// [Documentation](../../../../docs/ui/gui/charts/gauge.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's animated gauge with an identity independent of its literal caption.</summary>
    /// <param name="id">Stable inspection identity.</param>
    /// <param name="label">Literal caption, or null to omit it.</param>
    /// <param name="value">Finite reading clamped to zero through one.</param>
    /// <param name="size">Finite logical diameter. Nonpositive values select the native default.</param>
    /// <param name="track">Retain the public inspection identity.</param>
    public void Gauge(string id, string? label, float value, float size = 0, bool track = true)
    {
        CheckFrame();
        string stable = Stable(id);
        if (label is not null) Utf8.Validate(label, nameof(label));
        if (!float.IsFinite(value)) throw new ArgumentOutOfRangeException(nameof(value));
        if (!float.IsFinite(size)) throw new ArgumentOutOfRangeException(nameof(size));
        int node = NativeMethods.Gauge(context, stable, label, value, size);
        if (track) TrackControl(id, node);
    }
}
