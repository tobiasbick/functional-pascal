using Fpas.Ui.Gui.Cui.Interop;
namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's native segmented, arc or ring instrument.</summary>
    /// <param name="id">Stable identity.</param>
    /// <param name="value">Finite reading; native fill clamps it to the scale.</param>
    /// <param name="minimum">Inclusive lower scale limit.</param>
    /// <param name="maximum">Increasing upper scale limit.</param>
    /// <param name="options">Native bands, target, style and captions.</param>
    /// <param name="track">Track inspection identity.</param>
    public unsafe void RadialGauge(string id, float value, float minimum, float maximum, CuiRadialGaugeOptions options, bool track = true)
    {
        CheckFrame(); string stable = Stable(id); ArgumentNullException.ThrowIfNull(options); CheckRadialGauge(value, minimum, maximum, options);
        using var memory = new ChartMemory();
        var native = memory.Options(options);
        int node = NativeMethods.RadialGauge(context, stable, value, minimum, maximum, in native);
        if (track) TrackControl(id, node);
    }
}
