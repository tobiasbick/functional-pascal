using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws CUI's native radar rings, series, labels and value chip.</summary>
    /// <param name="id">Stable inspection identity.</param>
    /// <param name="series">Nonnegative samples, one per axis in each series.</param>
    /// <param name="axes">At least three literal axis captions.</param>
    /// <param name="options">Native scale and presentation.</param>
    /// <param name="enabled">Allow native pointer input.</param>
    /// <param name="track">Retain inspection identity.</param>
    /// <returns>Clicked axis index, otherwise minus one.</returns>
    public unsafe int RadarChart(string id, CuiChartSeries[] series, string[] axes, CuiRadarOptions options,
        bool enabled = true, bool track = true)
    {
        CheckFrame(); string stable = Stable(id); ArgumentNullException.ThrowIfNull(options);
        CheckRadar(series, axes, options);
        using var memory = new ChartMemory();
        int clicked = NativeMethods.RadarChart(context, stable, memory.Series(series), series.Length,
            memory.Labels(axes), axes.Length, options.Size, options.Maximum, memory.Format(options.Format),
            options.Legend ? 1 : 0, enabled ? 1 : 0, out int node);
        if (track) TrackControl(id, node);
        return clicked;
    }
}
