using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Draws native scatter or bubble series.</summary>
    /// <param name="id">Stable identity.</param>
    /// <param name="series">Named finite points with optional bubble sizes.</param>
    /// <param name="options">Native axes, format and dimensions.</param>
    /// <param name="enabled">Allow pointer input.</param>
    /// <param name="track">Track inspection identity.</param>
    /// <returns>Clicked point index flattened in input-series order, otherwise minus one.</returns>
    public unsafe int ScatterChart(string id, CuiScatterSeries[] series, CuiScatterOptions options, bool enabled = true, bool track = true)
    {
        CheckFrame(); string stable = Stable(id); ArgumentNullException.ThrowIfNull(options); CheckScatter(series, options);
        using var memory = new ChartMemory();
        int clicked = NativeMethods.ScatterChart(context, stable, memory.Scatter(series), series.Length,
            options.WidthKind, options.Width, options.Height, memory.Text(options.XLabel), memory.Text(options.YLabel),
            memory.Format(options.XFormat), memory.Format(options.YFormat), options.Legend ? 1 : 0, enabled ? 1 : 0, out int node);
        if (track) TrackControl(id, node); return clicked;
    }
}
